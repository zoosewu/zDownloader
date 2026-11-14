//! The native messaging host. The browser starts it for each message the extension sends;
//! it forwards the JSON to the app as one line and passes the one-line answer back, starting
//! the app first when it is not running.

use crate::{socket_name, ErrorCode, Response, EXTENSION_ID, MAX_LINE};
use interprocess::local_socket::{prelude::*, Name, Stream};
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const APP_EXE: &str = if cfg!(windows) { "zoo-downloader.exe" } else { "zoo-downloader" };

/// How long a freshly started app gets to open its socket.
pub const APP_START_WAIT: Duration = Duration::from_secs(10);

/// One native messaging message: a 32-bit length in native byte order, then UTF-8 JSON.
/// `None` when the browser closed stdin between messages.
pub fn read_message(input: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len = [0u8; 4];
    match input.read_exact(&mut len) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_ne_bytes(len);
    if u64::from(len) > MAX_LINE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("message of {len} bytes is too long")));
    }
    let mut message = vec![0; len as usize];
    input.read_exact(&mut message)?;
    Ok(Some(message))
}

pub fn write_message(output: &mut impl Write, json: &[u8]) -> io::Result<()> {
    let len = u32::try_from(json.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "message too long"))?;
    output.write_all(&len.to_ne_bytes())?;
    output.write_all(json)?;
    output.flush()
}

/// The browser passes the calling extension's origin as an argument. The host manifest already
/// allows only ours; this is the second lock.
pub fn is_our_extension(args: impl IntoIterator<Item = String>) -> bool {
    let ours = format!("chrome-extension://{EXTENSION_ID}/");
    args.into_iter().any(|arg| arg == ours)
}

pub struct Bridge {
    pub socket: Name<'static>,
    /// Started when the socket is missing; on Windows also the only process allowed to own it.
    pub app: PathBuf,
    pub wait: Duration,
}

impl Bridge {
    /// The app installed next to this executable.
    pub fn next_to_this_exe() -> io::Result<Self> {
        let exe = std::env::current_exe()?;
        let dir = exe.parent().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "executable has no folder"))?;
        Ok(Self { socket: socket_name()?, app: dir.join(APP_EXE), wait: APP_START_WAIT })
    }

    /// Forward one request and return the JSON answer for the extension. Always answers.
    pub fn forward(&self, request: &[u8]) -> String {
        // Re-encoding guarantees a single line.
        let line = match serde_json::from_slice::<Value>(request) {
            Ok(value) => value.to_string(),
            Err(e) => return error(ErrorCode::Internal, format!("bad request: {e}")),
        };
        let stream = match self.connect() {
            Ok(stream) => stream,
            Err(answer) => return answer,
        };
        exchange(&stream, &line).unwrap_or_else(|e| error(ErrorCode::Internal, format!("no answer from Zoo Downloader: {e}")))
    }

    fn connect(&self) -> Result<Stream, String> {
        if let Ok(stream) = Stream::connect(self.socket.clone()) {
            return self.verified(stream);
        }
        launch(&self.app).map_err(|e| error(ErrorCode::AppNotRunning, format!("cannot start {}: {e}", self.app.display())))?;
        let deadline = Instant::now() + self.wait;
        loop {
            std::thread::sleep(Duration::from_millis(200));
            if let Ok(stream) = Stream::connect(self.socket.clone()) {
                return self.verified(stream);
            }
            if Instant::now() >= deadline {
                return Err(error(ErrorCode::AppNotRunning, "Zoo Downloader did not start in time"));
            }
        }
    }

    /// Another program could create the pipe name first and collect cookies; on Windows the
    /// pipe must belong to the app next to us. (Unix sockets live in the user's own folder.)
    fn verified(&self, stream: Stream) -> Result<Stream, String> {
        #[cfg(windows)]
        {
            let owner = stream.peer_creds().ok().and_then(|c| c.pid()).and_then(|pid| process_exe(pid).ok());
            if !owner.is_some_and(|exe| same_file(&exe, &self.app)) {
                return Err(error(ErrorCode::Internal, "the local socket does not belong to Zoo Downloader"));
            }
        }
        Ok(stream)
    }
}

fn error(code: ErrorCode, message: impl Into<String>) -> String {
    let mut line = Response::error(code, message).to_line();
    line.pop();
    line
}

fn exchange(stream: &Stream, line: &str) -> io::Result<String> {
    let mut sender = stream;
    sender.write_all(format!("{line}\n").as_bytes())?;
    let mut answer = String::new();
    BufReader::new(stream).take(MAX_LINE).read_line(&mut answer)?;
    if !answer.ends_with('\n') {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "connection closed"));
    }
    answer.pop();
    Ok(answer)
}

/// Start the app detached from us: our stdout is the browser's channel, and the browser ends
/// our process (on Windows: our job) as soon as we answer.
fn launch(app: &Path) -> io::Result<()> {
    if !app.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "not installed"));
    }
    let mut cmd = Command::new(app);
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    if let Some(dir) = app.parent() {
        cmd.current_dir(dir);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        keep_std_handles_to_ourselves();
        const DETACHED_PROCESS: u32 = 0x8;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x200;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
        // Breaking away fails when the browser's job forbids it; the app may then live only as
        // long as the job, which is still better than not starting.
        if cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB).spawn().is_ok() {
            return Ok(());
        }
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    cmd.spawn().map(drop)
}

/// Windows hands every inheritable handle to a child. Ours are the browser's pipes: an app
/// holding them would keep the browser's channel to us open for as long as the app runs.
#[cfg(windows)]
fn keep_std_handles_to_ourselves() {
    use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE_FLAG_INHERIT};
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
    for std in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: GetStdHandle returns a handle we own or null/invalid, which SetHandleInformation rejects.
        unsafe {
            SetHandleInformation(GetStdHandle(std), HANDLE_FLAG_INHERIT, 0);
        }
    }
}

#[cfg(windows)]
fn process_exe(pid: u32) -> io::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: the handle is checked and closed; the buffer outlives the call and `len` holds its size.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut buf = vec![0u16; 32 * 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
        let err = io::Error::last_os_error();
        CloseHandle(process);
        if ok == 0 {
            return Err(err);
        }
        Ok(PathBuf::from(std::ffi::OsString::from_wide(&buf[..len as usize])))
    }
}

#[cfg(windows)]
fn same_file(a: &Path, b: &Path) -> bool {
    matches!((a.canonicalize(), b.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn framed(json: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        write_message(&mut out, json).unwrap();
        out
    }

    #[test]
    fn frames_round_trip() {
        let bytes = [framed(b"{\"a\":1}"), framed(b"{}")].concat();
        let mut input = bytes.as_slice();
        assert_eq!(read_message(&mut input).unwrap().unwrap(), b"{\"a\":1}");
        assert_eq!(read_message(&mut input).unwrap().unwrap(), b"{}");
        assert_eq!(read_message(&mut input).unwrap(), None);
    }

    #[test]
    fn rejects_truncated_and_oversized_frames() {
        let truncated = &framed(b"{\"a\":1}")[..6];
        assert!(read_message(&mut &truncated[..]).is_err());
        let huge = ((MAX_LINE + 1) as u32).to_ne_bytes();
        assert_eq!(read_message(&mut &huge[..]).unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn only_our_extension_may_start_the_host() {
        let ours = format!("chrome-extension://{EXTENSION_ID}/");
        assert!(is_our_extension([ours, "--parent-window=0".into()]));
        assert!(!is_our_extension(["chrome-extension://abcdefghijklmnopabcdefghijklmnop/".to_string()]));
        assert!(!is_our_extension(Vec::<String>::new()));
    }

    fn unique_socket() -> Name<'static> {
        static N: AtomicUsize = AtomicUsize::new(0);
        format!("zoo-host-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed))
            .to_ns_name::<GenericNamespaced>()
            .unwrap()
            .into_owned()
    }

    /// A fake app that answers one request with `answer` and returns what it received.
    fn fake_app(socket: Name<'static>, answer: &'static str) -> std::thread::JoinHandle<String> {
        let listener = ListenerOptions::new().name(socket).create_sync().unwrap();
        std::thread::spawn(move || {
            let stream = listener.accept().unwrap();
            let mut request = String::new();
            BufReader::new(&stream).read_line(&mut request).unwrap();
            (&stream).write_all(answer.as_bytes()).unwrap();
            request
        })
    }

    fn bridge(socket: Name<'static>, app: PathBuf) -> Bridge {
        Bridge { socket, app, wait: Duration::from_millis(500) }
    }

    #[test]
    fn forwards_one_line_and_returns_the_answer() {
        let socket = unique_socket();
        let app = fake_app(socket.clone(), "{\"ok\":true,\"name\":\"a.zip\"}\n");
        // The fake app is this test process, so it passes the owner check.
        let answer = bridge(socket, std::env::current_exe().unwrap()).forward(b"{ \"v\": 1,\n \"type\": \"ping\" }");
        assert_eq!(answer, "{\"ok\":true,\"name\":\"a.zip\"}");
        assert_eq!(app.join().unwrap(), "{\"type\":\"ping\",\"v\":1}\n");
    }

    #[cfg(windows)]
    #[test]
    fn refuses_a_socket_owned_by_another_program() {
        let socket = unique_socket();
        let _app = fake_app(socket.clone(), "{\"ok\":true}\n");
        let other = PathBuf::from(r"C:\Windows\System32\whoami.exe");
        let answer: Value = serde_json::from_str(&bridge(socket, other).forward(b"{\"v\":1,\"type\":\"ping\"}")).unwrap();
        assert_eq!(answer["code"], "internal");
        assert!(answer["message"].as_str().unwrap().contains("does not belong"), "{answer}");
    }

    #[test]
    fn reports_a_missing_app() {
        let answer: Value =
            serde_json::from_str(&bridge(unique_socket(), PathBuf::from("no-such-dir/zoo-downloader.exe")).forward(b"{\"v\":1,\"type\":\"ping\"}"))
                .unwrap();
        assert_eq!(answer["code"], "appNotRunning");
    }

    #[cfg(windows)]
    #[test]
    fn gives_up_when_the_started_app_never_listens() {
        let started = Instant::now();
        let answer: Value =
            serde_json::from_str(&bridge(unique_socket(), PathBuf::from(r"C:\Windows\System32\whoami.exe")).forward(b"{\"v\":1,\"type\":\"ping\"}"))
                .unwrap();
        assert_eq!(answer["code"], "appNotRunning");
        assert!(started.elapsed() >= Duration::from_millis(500));
    }

    #[test]
    fn malformed_requests_are_answered_without_connecting() {
        let answer: Value = serde_json::from_str(&bridge(unique_socket(), PathBuf::from("missing")).forward(b"not json")).unwrap();
        assert_eq!(answer["code"], "internal");
    }
}
