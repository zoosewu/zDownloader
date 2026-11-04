//! Web APIs as module code sees them, and fetch through the Policy against a fixture server.

mod common;

use std::time::Instant;

use common::{eval, host, loopback_host, Resp, Server};
use serde_json::json;

#[tokio::test]
async fn standard_globals_behave_like_the_browser() {
    let value = eval(
        &host(),
        &[],
        r#"
        const u = new URL('/a b?x=1#h', 'https://Example.com/base/');
        const q = new URLSearchParams({ a: '1', b: 'x y' });
        q.append('a', '2');
        const bytes = new TextEncoder().encode('héllo');
        const h = new Headers({ 'Content-Type': 'text/html' });
        h.append('Set-Cookie', 'a=1');
        h.append('set-cookie', 'b=2');
        const ac = new AbortController();
        ac.abort();
        const order = [];
        await new Promise((r) => { setTimeout(() => { order.push('timer'); r(); }, 5); order.push('sync'); });
        const doc = new DOMParser().parseFromString(
          '<html><head><title>T</title></head><body><a class="dl" href="/f.zip">x</a></body></html>', 'text/html');
        const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode('abc')));
        return {
          url: u.href,
          query: q.toString(),
          all: q.getAll('a'),
          utf8: [bytes.length, new TextDecoder().decode(bytes)],
          base64: [btoa('hi'), atob('aGk=')],
          clone: structuredClone({ d: [1, { e: 2 }] }),
          uuid: /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(crypto.randomUUID()),
          sha256: Array.from(digest.slice(0, 4)),
          headers: [h.get('content-type'), h.getSetCookie(), [...h.keys()]],
          aborted: [ac.signal.aborted, ac.signal.reason.name],
          order,
          dom: [doc.title, doc.querySelector('a.dl').getAttribute('href')],
          fatal: (() => { try { new TextDecoder('utf-8', { fatal: true }).decode(new Uint8Array([0xff])); return 'no'; } catch { return 'threw'; } })(),
        };
        "#,
    )
    .await;
    assert_eq!(
        value,
        json!({
            "url": "https://example.com/a%20b?x=1#h",
            "query": "a=1&b=x+y&a=2",
            "all": ["1", "2"],
            "utf8": [6, "héllo"],
            "base64": ["aGk=", "hi"],
            "clone": { "d": [1, { "e": 2 }] },
            "uuid": true,
            "sha256": [0xba, 0x78, 0x16, 0xbf],
            "headers": ["text/html", ["a=1", "b=2"], ["content-type", "set-cookie", "set-cookie"]],
            "aborted": [true, "AbortError"],
            "order": ["sync", "timer"],
            "dom": ["T", "/f.zip"],
            "fatal": "threw",
        })
    );
}

#[tokio::test]
async fn fetch_follows_redirects_keeps_cookies_and_posts_forms() {
    let server = Server::start(|req| match req.path.as_str() {
        "/login" => Resp::redirect(302, "/home").header("Set-Cookie", "sid=abc; Path=/"),
        "/home" => Resp::ok(format!("cookie={}", req.headers.get("cookie").cloned().unwrap_or_default())),
        "/form" => Resp::ok(String::from_utf8_lossy(&req.body).into_owned()).header("X-Echo", "1"),
        _ => Resp { status: 404, headers: vec![], body: vec![] },
    })
    .await;
    let base = server.url("");
    let value = eval(
        &loopback_host(),
        &["127.0.0.1"],
        &format!(
            r#"
            const home = await fetch('{base}/login');
            const form = await fetch('{base}/form', {{ method: 'POST', body: new URLSearchParams({{ user: 'u', pass: 'p w' }}) }});
            const missing = await fetch('{base}/nope');
            const manual = await fetch('{base}/login', {{ redirect: 'manual' }});
            return {{
              home: [home.status, home.ok, home.redirected, home.url.endsWith('/home'), await home.text()],
              form: [await form.text(), form.headers.get('x-echo')],
              missing: [missing.status, missing.ok],
              manual: [manual.status, manual.headers.get('location')],
            }};
            "#
        ),
    )
    .await;
    assert_eq!(
        value,
        json!({
            "home": [200, true, true, true, "cookie=sid=abc"],
            "form": ["user=u&pass=p+w", "1"],
            "missing": [404, false],
            "manual": [302, "/home"],
        })
    );
    let form = server.requests().into_iter().find(|r| r.path == "/form").unwrap();
    assert_eq!(form.method, "POST");
    assert!(form.headers["content-type"].starts_with("application/x-www-form-urlencoded"));
    assert!(form.headers["user-agent"].starts_with("Mozilla/5.0"), "browser-like default user agent");
}

/// Runs `fetch(url)` in a module allowed `domains` and returns the error message (or "ok").
async fn fetch_error(host: &zoo_module_client::Supervisor, domains: &[&str], url: &str) -> String {
    eval(host, domains, &format!("try {{ await fetch({url:?}); return 'ok'; }} catch (e) {{ return `${{e.name}}: ${{e.message}}`; }}"))
        .await
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn policy_blocks_unlisted_hosts_private_addresses_and_bad_redirects() {
    let server = Server::start(|req| match req.path.as_str() {
        "/to-evil" => Resp::redirect(302, "http://evil.invalid/"),
        "/to-metadata" => Resp::redirect(302, "http://169.254.169.254/latest/meta-data"),
        _ => Resp::ok("fine"),
    })
    .await;
    let host = loopback_host();
    let allowed = ["127.0.0.1", "169.254.169.254"];

    let unlisted = fetch_error(&host, &allowed, "http://example.com/").await;
    assert!(unlisted.starts_with("TypeError") && unlisted.contains("allowDomains"), "{unlisted}");

    let redirected = fetch_error(&host, &allowed, &server.url("/to-evil")).await;
    assert!(redirected.contains("evil.invalid is not in"), "{redirected}");

    let metadata = fetch_error(&host, &allowed, &server.url("/to-metadata")).await;
    assert!(metadata.contains("private or reserved"), "{metadata}");

    assert_eq!(fetch_error(&host, &allowed, &server.url("/ok")).await, "ok");
    // Nothing reached the fixture except the allowed requests.
    assert_eq!(server.requests().len(), 3);
}

#[tokio::test]
async fn names_resolving_to_loopback_are_blocked_without_the_test_flag() {
    let server = Server::start(|_| Resp::ok("secret")).await;
    let url = format!("http://localhost:{}/", server.addr.port());
    let error = fetch_error(&host(), &["localhost"], &url).await;
    assert!(error.contains("private or reserved"), "{error}");
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn oversized_responses_are_refused() {
    let server = Server::start(|_| Resp::ok(vec![b'x'; 17 * 1024 * 1024])).await;
    let error = fetch_error(&loopback_host(), &["127.0.0.1"], &server.url("/big")).await;
    assert!(error.contains("larger than 16 MB"), "{error}");
}

#[tokio::test]
async fn module_host_internals_stay_hidden() {
    let value = eval(
        &host(),
        &[],
        "return [typeof Deno, typeof __bootstrap, typeof globalThis['__core-js_shared__'], Object.keys(globalThis).filter((k) => k.startsWith('op_'))];",
    )
    .await;
    assert_eq!(value, json!(["undefined", "undefined", "undefined", []]));
}

#[tokio::test]
async fn a_call_with_the_full_web_layer_starts_quickly() {
    let host = host();
    eval(&host, &[], "return 1;").await; // first call pays process start-up
    let started = Instant::now();
    eval(&host, &[], "return new DOMParser().parseFromString('<p>x</p>', 'text/html').body.textContent;").await;
    let elapsed = started.elapsed();
    eprintln!("call with DOMParser: {elapsed:?}");
    assert!(elapsed.as_millis() < 1500, "{elapsed:?}");
}
