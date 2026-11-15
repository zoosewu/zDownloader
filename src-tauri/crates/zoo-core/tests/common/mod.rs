//! Minimal HTTP/1.1 fixture server for integration tests.
//!
//! `GET /file/<size>` serves `size` deterministic bytes (see `expected`). Query flags shape
//! the server's behaviour:
//!   norange        ignore Range entirely, always 200 with the full body
//!   range200       honour the 0-0 probe, but answer every other Range with 200
//!   nolength       no Content-Length; body delimited by connection close
//!   drop=N&times=K the first K body responses close the connection after N bytes
//!   fail=CODE&times=K  the first K requests get status CODE
//!   status=CODE    always answer CODE with an empty body
//!   delay_ms=D     sleep D ms before each 16 KiB chunk
//!   alt            serve different bytes (see `alt_byte_at`): another file of the same size
//!   disposition=V  send `Content-Disposition: V` (V percent-encoded in the query)
//! Requests whose target is an absolute URL are recorded as proxied (the fixture doubles as
//! a plain HTTP proxy for its own URLs).
#![allow(dead_code)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone)]
pub struct Request {
    pub path: String,
    pub range: Option<String>,
    pub proxied: bool,
    /// Lowercased names.
    pub headers: HashMap<String, String>,
}

#[derive(Default)]
struct State {
    requests: Vec<Request>,
    /// Remaining forced failures per (path+query) key.
    remaining: HashMap<String, u32>,
    etag: String,
}

pub struct Fixture {
    pub addr: SocketAddr,
    state: Arc<Mutex<State>>,
}

impl Fixture {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = Arc::new(Mutex::new(State { etag: "\"v1\"".into(), ..Default::default() }));
        let shared = state.clone();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else { return };
                let state = shared.clone();
                tokio::spawn(async move {
                    let _ = handle(stream, state).await;
                });
            }
        });
        Self { addr, state }
    }

    pub fn url(&self, path_and_query: &str) -> String {
        format!("http://{}{}", self.addr, path_and_query)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn set_etag(&self, etag: &str) {
        self.state.lock().unwrap().etag = format!("\"{etag}\"");
    }
}

/// Byte `i` of every fixture file.
pub fn byte_at(i: u64) -> u8 {
    ((i * 31 + 7) % 251) as u8
}

pub fn expected(size: u64) -> Vec<u8> {
    (0..size).map(byte_at).collect()
}

/// Byte `i` of a fixture file served with `alt`.
pub fn alt_byte_at(i: u64) -> u8 {
    !byte_at(i)
}

async fn handle(stream: TcpStream, state: Arc<Mutex<State>>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    // A connection closed before its request line (a cancelled request) is not a request.
    if reader.read_line(&mut line).await? == 0 || line.trim().is_empty() {
        return Ok(());
    }
    let target = line.split_whitespace().nth(1).unwrap_or("/").to_string();
    let mut headers = HashMap::new();
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).await? == 0 || h == "\r\n" {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }

    let proxied = target.starts_with("http://");
    let path_and_query = if proxied {
        let after_scheme = &target["http://".len()..];
        after_scheme.find('/').map(|i| after_scheme[i..].to_string()).unwrap_or("/".into())
    } else {
        target
    };
    let (path, query) = path_and_query.split_once('?').unwrap_or((&path_and_query, ""));
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|kv| kv.split_once('=').unwrap_or((kv, "")))
        .collect();
    let range = headers.get("range").cloned();

    let etag = {
        let mut st = state.lock().unwrap();
        st.requests.push(Request { path: path.to_string(), range: range.clone(), proxied, headers: headers.clone() });
        st.etag.clone()
    };
    let mut stream = reader.into_inner();

    let take_failure = |key: &str| -> bool {
        let times: u32 = params.get("times").and_then(|t| t.parse().ok()).unwrap_or(1);
        let mut st = state.lock().unwrap();
        let left = st.remaining.entry(format!("{key}:{path_and_query}")).or_insert(times);
        if *left > 0 {
            *left -= 1;
            true
        } else {
            false
        }
    };

    if let Some(code) = params.get("status") {
        return respond(&mut stream, code, &[], &[]).await;
    }
    if let Some(code) = params.get("fail") {
        if take_failure("fail") {
            return respond(&mut stream, code, &[], &[]).await;
        }
    }

    let Some(size) = path.strip_prefix("/file/").and_then(|s| s.parse::<u64>().ok()) else {
        return respond(&mut stream, "404", &[], &[]).await;
    };

    let probe = range.as_deref() == Some("bytes=0-0");
    let honour_range = !params.contains_key("norange") && (!params.contains_key("range200") || probe);
    let requested = range.as_deref().and_then(parse_range).filter(|_| honour_range);

    let (status, start, end) = match requested {
        Some((s, _)) if s >= size => {
            let cr = format!("bytes */{size}");
            return respond(&mut stream, "416", &[("Content-Range", &cr)], &[]).await;
        }
        Some((s, e)) => ("206", s, e.min(size.saturating_sub(1)) + 1),
        None => ("200", 0, size),
    };

    let mut head: Vec<(&str, String)> = vec![("ETag", etag), ("Accept-Ranges", "bytes".into())];
    if let Some(value) = params.get("disposition") {
        head.push(("Content-Disposition", percent_encoding::percent_decode_str(value).decode_utf8_lossy().into_owned()));
    }
    if status == "206" {
        head.push(("Content-Range", format!("bytes {}-{}/{}", start, end - 1, size)));
    }
    let nolength = params.contains_key("nolength");
    if !nolength {
        head.push(("Content-Length", (end - start).to_string()));
    }
    let head_refs: Vec<(&str, &str)> = head.iter().map(|(k, v)| (*k, v.as_str())).collect();
    write_head(&mut stream, status, &head_refs).await?;

    let drop_after: Option<u64> = params
        .get("drop")
        .and_then(|d| d.parse().ok())
        .filter(|_| !probe && take_failure("drop"));
    let delay = params.get("delay_ms").and_then(|d| d.parse().ok()).map(std::time::Duration::from_millis);

    let mut sent = 0u64;
    let mut pos = start;
    while pos < end {
        if let Some(d) = delay {
            tokio::time::sleep(d).await;
        }
        let chunk_end = (pos + 16 * 1024).min(end);
        let byte = if params.contains_key("alt") { alt_byte_at } else { byte_at };
        let mut chunk: Vec<u8> = (pos..chunk_end).map(byte).collect();
        if let Some(limit) = drop_after {
            if sent + chunk.len() as u64 >= limit {
                chunk.truncate((limit - sent) as usize);
                stream.write_all(&chunk).await?;
                return stream.shutdown().await; // abrupt end mid-body
            }
        }
        stream.write_all(&chunk).await?;
        sent += chunk.len() as u64;
        pos = chunk_end;
    }
    stream.shutdown().await
}

fn parse_range(v: &str) -> Option<(u64, u64)> {
    let (s, e) = v.strip_prefix("bytes=")?.split_once('-')?;
    Some((s.parse().ok()?, e.parse().unwrap_or(u64::MAX)))
}

async fn write_head(stream: &mut TcpStream, status: &str, headers: &[(&str, &str)]) -> std::io::Result<()> {
    let mut head = format!("HTTP/1.1 {status} X\r\nConnection: close\r\n");
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await
}

async fn respond(stream: &mut TcpStream, status: &str, headers: &[(&str, &str)], body: &[u8]) -> std::io::Result<()> {
    let len = body.len().to_string();
    let mut all = headers.to_vec();
    all.push(("Content-Length", &len));
    write_head(stream, status, &all).await?;
    stream.write_all(body).await?;
    stream.shutdown().await
}
