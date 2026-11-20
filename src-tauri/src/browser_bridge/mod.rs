//! The app's end of the browser extension bridge (docs/architecture/browser-extension.md):
//! answers `zoo-native-host` on a per-user local socket, and registers the host with the
//! browsers.

pub mod install;
mod register;

use crate::add_window::{self, AddItem};
use crate::commands::{downloads, error::CommandError, settings::SettingsState, urls};
use crate::site_modules::SiteModules;
use crate::downloads::Downloads;
use interprocess::local_socket::{tokio::Listener, traits::tokio::Listener as _, ListenerOptions, Name};
use std::future::Future;
use std::io;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use zoo_native_host::{parse_request, AddDownload, ErrorCode, Request, Response, MAX_LINE};

pub fn start(app: AppHandle) {
    register::register();
    // The listener needs the Tokio runtime, which Tauri's setup does not run in.
    tauri::async_runtime::spawn(async move {
        #[cfg(unix)]
        let ready = zoo_native_host::socket_dir().and_then(|dir| private_dir(&dir));
        #[cfg(windows)]
        let ready = Ok(());
        let listener = match ready.and_then(|()| zoo_native_host::socket_name()).and_then(listen) {
            Ok(l) => l,
            Err(e) => {
                // Most likely another instance already listens; that one serves the extension.
                log::warn!("browser extension bridge not started: {e}");
                return;
            }
        };
        loop {
            match listener.accept().await {
                Ok(conn) => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = answer(conn, |req| handle(&app, req)).await {
                            log::warn!("browser extension bridge: {e}");
                        }
                    });
                }
                Err(e) => log::warn!("browser extension bridge: accept failed: {e}"),
            }
        }
    });
}

/// Only the current user may connect. On Windows the pipe is also local-only and must be the
/// first instance of its name (interprocess defaults); the host checks who owns it.
fn listen(name: Name<'static>) -> io::Result<Listener> {
    match create(name.clone(), false) {
        // A socket file left by a crashed app would block the name forever; one that still
        // answers belongs to a running app and is left alone.
        #[cfg(unix)]
        Err(e) if e.kind() == io::ErrorKind::AddrInUse && !answers(name.clone()) => create(name, true),
        result => result,
    }
}

fn create(name: Name<'static>, overwrite: bool) -> io::Result<Listener> {
    let options = ListenerOptions::new().name(name);
    #[cfg(windows)]
    let options = {
        use interprocess::os::windows::{local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor};
        // Protected DACL: full access for the pipe's owner (this user) and SYSTEM, nobody else.
        options.security_descriptor(SecurityDescriptor::deserialize(widestring::u16cstr!("D:P(A;;GA;;;OW)(A;;GA;;;SY)"))?)
    };
    // Unix: the socket's folder is the user's alone (`private_dir`); a socket file's own mode
    // cannot be set on macOS (fchmod fails, and the bridge would never listen).
    let options = options.try_overwrite(overwrite);
    options.create_tokio()
}

#[cfg(unix)]
fn answers(name: Name<'static>) -> bool {
    use interprocess::local_socket::{traits::Stream as _, Stream};
    Stream::connect(name).is_ok()
}

/// Make `dir` (created if missing) accessible to its owner only.
#[cfg(unix)]
fn private_dir(dir: &std::path::Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

/// Read one request line, write one response line.
async fn answer<S, F, Fut>(conn: S, handle: F) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite,
    F: FnOnce(Request) -> Fut,
    Fut: Future<Output = Response>,
{
    let (recv, mut send) = tokio::io::split(conn);
    let mut line = String::new();
    BufReader::new(recv.take(MAX_LINE)).read_line(&mut line).await?;
    let response = match parse_request(&line) {
        Ok(request) => handle(request).await,
        Err(response) => response,
    };
    send.write_all(response.to_line().as_bytes()).await?;
    send.flush().await
}

async fn handle(app: &AppHandle, request: Request) -> Response {
    match request {
        Request::Ping => Response::ok(None),
        Request::Add(add) => {
            // Not the URL or headers: they can carry tokens and cookies.
            log::info!("download added by the browser extension");
            let headers = add.headers();
            let AddDownload { url, filename, .. } = add;
            let (downloads, settings) = (app.state::<Downloads>(), app.state::<SettingsState>());
            let Some(file) = urls::starts_at_once(&url, &app.state::<SiteModules>(), &settings).await else {
                // The user checks it first, with the browser's headers kept.
                return match add_window::open(app, vec![AddItem { url, filename, headers }]).await {
                    Ok(()) => Response::ok(None),
                    Err(e) => error_response(e),
                };
            };
            // The module's link for the file, with what the user answered it while listing.
            let filename = filename.or(file.filename);
            match downloads::add_now(&downloads, &settings, file.url, filename, headers, file.answers).await {
                Ok(task) => Response::ok(Some(task.name)),
                Err(e) => error_response(e),
            }
        }
    }
}

fn error_response(e: CommandError) -> Response {
    let code = if e.code == "invalid" { ErrorCode::Invalid } else { ErrorCode::Internal };
    Response::error(code, e.message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::{tokio::Stream, traits::tokio::Stream as _, GenericNamespaced, ToNsName};
    use zoo_core::CoreError;

    async fn exchange(request: &str) -> (String, Option<Request>) {
        let (client, server) = tokio::io::duplex(1024);
        let seen = std::sync::Mutex::new(None);
        let serving = answer(server, |req| {
            *seen.lock().unwrap() = Some(req);
            async { Response::ok(Some("a.zip".into())) }
        });
        let asking = async {
            let (recv, mut send) = tokio::io::split(client);
            send.write_all(request.as_bytes()).await.unwrap();
            let mut line = String::new();
            BufReader::new(recv).read_line(&mut line).await.unwrap();
            line
        };
        let (served, line) = tokio::join!(serving, asking);
        served.unwrap();
        (line, seen.into_inner().unwrap())
    }

    #[tokio::test]
    async fn answers_one_request_with_one_line() {
        let (line, seen) = exchange("{\"v\":1,\"type\":\"ping\"}\n").await;
        assert_eq!(line, "{\"ok\":true,\"name\":\"a.zip\"}\n");
        assert_eq!(seen, Some(Request::Ping));
    }

    #[tokio::test]
    async fn bad_requests_never_reach_the_handler() {
        let (line, seen) = exchange("{\"v\":9,\"type\":\"ping\"}\n").await;
        assert!(line.contains("\"code\":\"unsupportedVersion\""), "{line}");
        assert_eq!(seen, None);
    }

    #[test]
    fn invalid_input_is_reported_as_invalid() {
        let e = CommandError::from(CoreError::Invalid("invalid URL: x".into()));
        assert_eq!(error_response(e), Response::error(ErrorCode::Invalid, "invalid URL: x"));
        let e = CommandError::new("io", "disk gone");
        assert_eq!(error_response(e).code, Some(ErrorCode::Internal));
    }

    #[cfg(unix)]
    #[test]
    fn the_socket_folder_is_the_users_alone() {
        use std::os::unix::fs::PermissionsExt;
        let parent = tempfile::tempdir().unwrap();
        let dir = parent.path().join("zoo-downloader");
        private_dir(&dir).unwrap();
        assert_eq!(std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777, 0o700);
    }

    /// A socket file nobody listens on (a crashed app's) is taken over; a live one is not.
    #[cfg(unix)]
    #[tokio::test]
    async fn takes_over_only_a_dead_socket_file() {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bridge.sock");
        drop(std::os::unix::net::UnixListener::bind(&path).unwrap());
        assert!(path.exists());
        let name = path.as_path().to_fs_name::<GenericFilePath>().unwrap().into_owned();
        let _live = listen(name.clone()).unwrap();
        assert!(listen(name).is_err());
    }

    #[tokio::test]
    async fn serves_over_the_local_socket() {
        let name = format!("zoo-bridge-test-{}", std::process::id()).to_ns_name::<GenericNamespaced>().unwrap().into_owned();
        let listener = listen(name.clone()).unwrap();
        // The name is taken: a second app instance must not listen on it.
        assert!(listen(name.clone()).is_err());
        let server = tokio::spawn(async move {
            let conn = listener.accept().await.unwrap();
            answer(conn, |_| async { Response::ok(None) }).await.unwrap();
        });
        let conn = Stream::connect(name).await.unwrap();
        let (recv, mut send) = tokio::io::split(conn);
        send.write_all(b"{\"v\":1,\"type\":\"ping\"}\n").await.unwrap();
        let mut line = String::new();
        BufReader::new(recv).read_line(&mut line).await.unwrap();
        assert_eq!(line, "{\"ok\":true}\n");
        server.await.unwrap();
    }
}
