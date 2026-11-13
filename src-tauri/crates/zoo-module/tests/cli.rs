//! The module author's CLI (`zoo-module new / check / test`), run as a real process.

mod common;

use std::path::Path;

use common::{Resp, Server};
use serde_json::{json, Value};
use tempfile::TempDir;

async fn zoo_module(dir: &Path, args: &[&str]) -> (bool, String, String) {
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_zoo-module"))
        .args(args)
        .current_dir(dir)
        .output()
        .await
        .unwrap();
    (output.status.success(), String::from_utf8_lossy(&output.stdout).into(), String::from_utf8_lossy(&output.stderr).into())
}

async fn json_of(dir: &Path, args: &[&str]) -> (bool, Value) {
    let (ok, stdout, stderr) = zoo_module(dir, args).await;
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}\n{stderr}"));
    (ok, value)
}

/// `zoo-module new` a module in `root/name`, pointed at `site`.
async fn new_module(root: &TempDir, site: &Server, code: &str) -> std::path::PathBuf {
    let (ok, stdout, stderr) = zoo_module(root.path(), &["new", "mod", "--id", "fixture", "--name", "Fixture"]).await;
    assert!(ok, "{stdout}{stderr}");
    let dir = root.path().join("mod");
    let mut manifest: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    manifest["urlPatterns"] = json!([format!("http://{}/page/*", site.addr)]);
    manifest["permissions"]["network"]["allowDomains"] = json!(["127.0.0.1"]);
    std::fs::write(dir.join("manifest.json"), manifest.to_string()).unwrap();
    std::fs::create_dir_all(dir.join("dist")).unwrap();
    std::fs::write(dir.join("dist/index.js"), code).unwrap();
    dir
}

fn write_cases(dir: &Path, cases: Value) {
    std::fs::write(dir.join("tests/cases.json"), cases.to_string()).unwrap();
}

const SITE_MODULE: &str = r#"
export default {
  async resolve(url, zoo) {
    await fetch(new URL('/login', url), {
      method: 'POST',
      body: new URLSearchParams({ pass: zoo.credentials.placeholder('password') }),
    });
    const page = await fetch(url);
    if (page.status === 404) throw zoo.fail('file not found', { hint: 'deleted' });
    const href = new DOMParser().parseFromString(await page.text(), 'text/html').querySelector('a').getAttribute('href');
    console.log('link', href);
    await zoo.wait(0.3, 'countdown');
    return { url: new URL(href, page.url).href, filename: 'movie.mp4' };
  },
};
"#;

async fn site() -> Server {
    Server::start(|req| match req.path.as_str() {
        "/login" => Resp::ok("welcome, s3cret-pass").header("Set-Cookie", "sid=7; Path=/"),
        "/page/ok" if req.headers.get("cookie").map(String::as_str) == Some("sid=7") => {
            Resp::ok(r#"<a href="/dl/movie?t=1">get</a>"#)
        }
        "/page/old" => Resp::redirect(302, "/page/ok"),
        _ => Resp { status: 404, headers: vec![], body: vec![] },
    })
    .await
}

#[tokio::test]
async fn new_creates_a_complete_module() {
    let root = TempDir::new().unwrap();
    let (ok, stdout, _) = zoo_module(root.path(), &["new", "My Host"]).await;
    assert!(ok, "{stdout}");
    let dir = root.path().join("My Host");
    for file in [
        "manifest.json", "AGENTS.md", "CLAUDE.md", ".gitignore", "package.json", "tsconfig.json", "src/index.ts",
        "tests/cases.json", "types/zoo-module.d.ts", "types/globals.d.ts", "types/manifest.schema.json",
    ] {
        assert!(dir.join(file).is_file(), "{file} missing");
    }
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["id"], "my-host");
    assert!(std::fs::read_to_string(dir.join(".gitignore")).unwrap().contains(".credentials.json"));

    let (ok, _, stderr) = zoo_module(root.path(), &["new", "My Host"]).await;
    assert!(!ok && stderr.contains("not empty"), "{stderr}");
}

#[tokio::test]
async fn check_reports_what_to_fix() {
    let root = TempDir::new().unwrap();
    zoo_module(root.path(), &["new", "m"]).await;
    let dir = root.path().join("m");

    let (ok, report) = json_of(&dir, &["check", "--json"]).await;
    assert!(!ok);
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|d| d["level"] == "error" && d["hint"].as_str().unwrap().contains("pnpm build")), "{report:#}");

    std::fs::create_dir_all(dir.join("dist")).unwrap();
    std::fs::write(dir.join("dist/index.js"), "export default { resolve(url, zoo) { zoo.credentials.placeholder('apiKey'); } };").unwrap();
    let (ok, report) = json_of(&dir, &["check", "--json"]).await;
    assert!(ok, "warnings only: {report:#}");
    let messages: Vec<&str> = report["diagnostics"].as_array().unwrap().iter().map(|d| d["message"].as_str().unwrap()).collect();
    assert!(messages.iter().any(|m| m.contains("\"apiKey\"")), "undeclared credential: {messages:?}");
    assert!(messages.iter().any(|m| m.contains("no recording")), "unrecorded case: {messages:?}");

    std::fs::write(dir.join("dist/index.js"), "import x from 'cheerio'; export default {};").unwrap();
    let (ok, stdout, _) = zoo_module(&dir, &["check"]).await;
    assert!(!ok && stdout.contains("single file"), "{stdout}");
}

#[tokio::test]
async fn record_once_then_replay_offline() {
    let root = TempDir::new().unwrap();
    let server = site().await;
    let dir = new_module(&root, &server, SITE_MODULE).await;
    std::fs::write(dir.join(".credentials.json"), r#"{ "password": "s3cret-pass" }"#).unwrap();
    write_cases(
        &dir,
        json!([
            { "name": "ok", "url": server.url("/page/ok"), "expect": { "url": server.url("/dl/*"), "filename": "*.mp4" } },
            { "name": "moved", "url": server.url("/page/old"), "expect": { "url": "*movie*" } },
            { "name": "gone", "url": server.url("/page/nope"), "expect": { "error": "file not found" } },
        ]),
    );

    let (ok, report) = json_of(&dir, &["test", "--record", "--json", "--allow-loopback"]).await;
    assert!(ok, "{report:#}");
    assert_eq!((report["passed"].as_u64(), report["failed"].as_u64()), (Some(3), Some(0)));
    let ok_case = &report["cases"][0];
    assert_eq!(ok_case["mode"], "record");
    let steps = ok_case["steps"].as_array().unwrap();
    assert!(steps.iter().any(|s| s["kind"] == "log" && s["message"] == "link /dl/movie?t=1"));
    assert!(steps.iter().any(|s| s["kind"] == "wait" && s["skipped"] == false));

    let cassette = std::fs::read_to_string(dir.join("tests/cassettes/ok.json")).unwrap();
    assert!(!cassette.contains("s3cret-pass"), "credential values never reach a recording");
    assert!(cassette.contains("{{credential:password}}"));

    // The site is gone; the recordings are enough, even without the account.
    drop(server);
    std::fs::remove_file(dir.join(".credentials.json")).unwrap();
    let (ok, report) = json_of(&dir, &["test", "--json", "--allow-loopback"]).await;
    assert!(ok, "{report:#}");
    let ok_case = &report["cases"][0];
    assert_eq!(ok_case["mode"], "replay");
    assert!(ok_case["steps"].as_array().unwrap().iter().any(|s| s["kind"] == "wait" && s["skipped"] == true));
    assert_eq!(ok_case["resolved"]["filename"], "movie.mp4");

    // A request the recording does not have says how to fix it.
    std::fs::write(dir.join("dist/index.js"), SITE_MODULE.replace("'/login'", "'/signin'")).unwrap();
    let (ok, stdout, _) = zoo_module(&dir, &["test", "--case", "ok", "--allow-loopback"]).await;
    assert!(!ok);
    assert!(stdout.contains("not in the cassette") && stdout.contains("--record"), "{stdout}");
}

#[tokio::test]
async fn blocked_requests_and_failed_expectations_are_explained() {
    let root = TempDir::new().unwrap();
    let server = site().await;
    let code = "export default { async resolve(url) { await fetch('https://elsewhere.example/'); return { url }; } };";
    let dir = new_module(&root, &server, code).await;
    write_cases(&dir, json!([{ "name": "a", "url": server.url("/page/ok"), "expect": { "url": "https://cdn/*" } }]));

    let (ok, stdout, _) = zoo_module(&dir, &["test", "--allow-loopback"]).await;
    assert!(!ok);
    assert!(stdout.contains("BLOCKED") && stdout.contains("elsewhere.example") && stdout.contains("allowDomains"), "{stdout}");
    assert!(stdout.contains("FAIL a"), "{stdout}");
}
