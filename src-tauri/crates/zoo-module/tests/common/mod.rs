//! Shared helpers: a host behind a Supervisor, module files, and a tiny HTTP fixture server.
#![allow(dead_code)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use zoo_module_client::{NoInteraction, Grants, Limits, ModuleRef, Supervisor};

pub fn host() -> Supervisor {
    Supervisor::new(env!("CARGO_BIN_EXE_zoo-module"), ["serve"])
}

/// A host whose fetch may reach the loopback fixture server.
pub fn loopback_host() -> Supervisor {
    Supervisor::new(env!("CARGO_BIN_EXE_zoo-module"), ["serve", "--allow-loopback"])
}

pub fn module(dir: &TempDir, name: &str, code: &str) -> ModuleRef {
    module_allowing(dir, name, code, &[])
}

pub fn module_allowing(dir: &TempDir, name: &str, code: &str, domains: &[&str]) -> ModuleRef {
    let entry = dir.path().join(format!("{name}.js"));
    std::fs::write(&entry, code).unwrap();
    ModuleRef {
        id: format!("{name}@test"),
        entry,
        grants: Grants { allow_domains: domains.iter().map(|d| d.to_string()).collect(), ..Default::default() },
    }
}

pub fn limits(timeout_ms: u64) -> Limits {
    Limits { timeout_ms, ..Limits::default() }
}

/// Runs `body` (the inside of an async function) as a module and returns what it returned,
/// carried back as JSON in `filename`.
pub async fn eval(host: &Supervisor, domains: &[&str], body: &str) -> serde_json::Value {
    let dir = TempDir::new().unwrap();
    let code = format!(
        "export default {{ async resolve(url, zoo) {{\n\
           const value = await (async () => {{\n{body}\n}})();\n\
           return {{ url: 'https://result/', filename: JSON.stringify(value ?? null) }};\n\
         }} }};"
    );
    let resolved = host
        .resolve(module_allowing(&dir, "eval", &code, domains), "https://input/", Limits::default(), &NoInteraction)
        .await
        .unwrap_or_else(|e| panic!("module failed: {e:?}"));
    serde_json::from_str(&resolved.filename.unwrap()).unwrap()
}

#[derive(Debug, Clone)]
pub struct Req {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self { status: 200, headers: vec![], body: body.into() }
    }
    pub fn redirect(status: u16, location: &str) -> Self {
        Self { status, headers: vec![("Location".into(), location.into())], body: vec![] }
    }
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

type Handler = Arc<dyn Fn(&Req) -> Resp + Send + Sync>;

pub struct Server {
    pub addr: SocketAddr,
    requests: Arc<Mutex<Vec<Req>>>,
}

impl Server {
    /// One response per connection (`Connection: close`).
    pub async fn start(handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let handler: Handler = Arc::new(handler);
        let log = requests.clone();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let (handler, log) = (handler.clone(), log.clone());
                tokio::spawn(async move {
                    let mut stream = BufReader::new(stream);
                    let mut line = String::new();
                    if stream.read_line(&mut line).await.is_err() {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or_default().to_string();
                    let path = parts.next().unwrap_or_default().to_string();
                    let mut headers = HashMap::new();
                    loop {
                        let mut header = String::new();
                        if stream.read_line(&mut header).await.unwrap_or(0) == 0 || header.trim().is_empty() {
                            break;
                        }
                        if let Some((name, value)) = header.split_once(':') {
                            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
                        }
                    }
                    let length = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let mut body = vec![0; length];
                    let _ = stream.read_exact(&mut body).await;
                    let req = Req { method, path, headers, body };
                    let resp = handler(&req);
                    log.lock().unwrap().push(req);

                    let mut head = format!("HTTP/1.1 {} X\r\nContent-Length: {}\r\nConnection: close\r\n", resp.status, resp.body.len());
                    for (name, value) in &resp.headers {
                        head.push_str(&format!("{name}: {value}\r\n"));
                    }
                    head.push_str("\r\n");
                    let stream = stream.get_mut();
                    let _ = stream.write_all(head.as_bytes()).await;
                    let _ = stream.write_all(&resp.body).await;
                    let _ = stream.shutdown().await;
                });
            }
        });
        Self { addr, requests }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    pub fn requests(&self) -> Vec<Req> {
        self.requests.lock().unwrap().clone()
    }
}
