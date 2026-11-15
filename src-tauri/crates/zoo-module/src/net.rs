//! The only way a module reaches the network. Every hop of every request passes the Policy:
//! http(s) only, host on the module's allowlist, and (after DNS, so rebinding cannot dodge it)
//! a public address. Redirects are followed here, hop by hop, never by reqwest.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use deno_core::url::Url;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::cookie::{CookieStore, Jar};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use zoo_module_client::protocol::{Resolved, Secret};

use crate::cassette::{Cassette, Exchange};
use crate::hooks::{FetchOutcome, Step, Trace};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Method, StatusCode};

const MAX_REDIRECTS: usize = 20;
/// Largest response body a module may read.
pub const MAX_BODY: usize = 16 * 1024 * 1024;
/// Hosters routinely refuse non-browser user agents.
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";

#[derive(Debug, Clone)]
pub struct Policy {
    /// Lowercased; each entry also covers its subdomains. `*.` prefixes are accepted and ignored.
    allow_domains: Vec<String>,
    /// Test-only (`serve --allow-loopback`): lets fixtures on 127.0.0.1 through. Allowlist still applies.
    allow_loopback: bool,
}

impl Policy {
    pub fn new(allow_domains: &[String], allow_loopback: bool) -> Self {
        let allow_domains = allow_domains
            .iter()
            .map(|d| d.trim().trim_start_matches("*.").trim_end_matches('.').to_ascii_lowercase())
            .filter(|d| !d.is_empty())
            .collect();
        Self { allow_domains, allow_loopback }
    }

    /// Checks everything knowable before connecting. Hostnames are checked again after DNS.
    pub fn check_url(&self, url: &Url) -> Result<(), String> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(format!("fetch {url} blocked: only http and https are allowed"));
        }
        let host = match url.host() {
            Some(deno_core::url::Host::Domain(d)) => d.to_ascii_lowercase(),
            Some(deno_core::url::Host::Ipv4(ip)) => self.check_ip_literal(url, ip.into())?,
            Some(deno_core::url::Host::Ipv6(ip)) => self.check_ip_literal(url, ip.into())?,
            None => return Err(format!("fetch {url} blocked: no host")),
        };
        let host = host.trim_end_matches('.');
        let allowed = self
            .allow_domains
            .iter()
            .any(|d| host == d || host.strip_suffix(d.as_str()).is_some_and(|rest| rest.ends_with('.')));
        if allowed {
            Ok(())
        } else {
            Err(format!(
                "fetch {url} blocked: {host} is not in the module's permissions.network.allowDomains \
                 (add \"{host}\" to manifest.json if the module really needs it)"
            ))
        }
    }

    fn check_ip_literal(&self, url: &Url, ip: IpAddr) -> Result<String, String> {
        if self.ip_allowed(ip) {
            Ok(ip.to_string())
        } else {
            Err(format!("fetch {url} blocked: {ip} is a private or reserved address"))
        }
    }

    pub fn ip_allowed(&self, ip: IpAddr) -> bool {
        is_public(ip) || (self.allow_loopback && ip.is_loopback())
    }
}

/// Globally routable unicast. Anything that could reach the user's machine, LAN or cloud
/// metadata endpoints is not.
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_public_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_v4(v4);
            }
            let s = v6.segments();
            // NAT64 (64:ff9b::/96) carries an IPv4 address.
            if s[0..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
                return is_public_v4(Ipv4Addr::from(((s[6] as u32) << 16) | s[7] as u32));
            }
            !(v6.is_unspecified()
                || v6.is_loopback()
                || v6.is_multicast()
                || (s[0] & 0xfe00) == 0xfc00 // unique local
                || (s[0] & 0xffc0) == 0xfe80 // link-local
                || (s[0] & 0xffc0) == 0xfec0 // site-local (deprecated)
                || (s[0] == 0x2001 && s[1] == 0x0db8) // documentation
                || s[0] == 0x2002 // 6to4
                || (s[0] == 0x2001 && s[1] == 0) // Teredo
                || s[0..5] == [0, 0, 0, 0, 0]) // IPv4-compatible and other ::/80 forms
        }
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        || a == 0
        || (a == 100 && (64..128).contains(&b)) // CGNAT
        || (a == 192 && b == 0 && c == 0) // IETF protocol assignments
        || (a == 198 && (b == 18 || b == 19)) // benchmarking
        || a >= 240) // reserved
}

/// Resolves names and drops every address the Policy refuses, so a hostname that points
/// inside the network fails instead of connecting.
struct PolicyResolver(Arc<Policy>);

impl Resolve for PolicyResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let policy = self.0.clone();
        Box::pin(async move {
            let host = name.as_str().to_string();
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            let allowed: Vec<SocketAddr> = addrs.into_iter().filter(|a| policy.ip_allowed(a.ip())).collect();
            if allowed.is_empty() {
                return Err(format!("{host} resolves only to private or reserved addresses").into());
            }
            Ok(Box::new(allowed.into_iter()) as Addrs)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Redirect {
    Follow,
    Manual,
    Error,
}

pub struct FetchRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub redirect: Redirect,
}

pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub url: String,
    pub redirected: bool,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Credential values the module may use but never see. Module code gets placeholders
/// (`zoocred-<nonce>-<key>-`, unchanged by URL, form and JSON encoding); requests get the
/// value, encoded for where the placeholder sits; responses get the value replaced back.
pub struct Credentials {
    nonce: String,
    values: BTreeMap<String, String>,
}

/// Shorter values are too likely to occur in ordinary pages to scrub safely.
const MIN_SCRUB_LEN: usize = 6;
/// RFC 3986 unreserved characters stay; everything else is percent-encoded.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

impl Credentials {
    pub fn new(granted: &BTreeMap<String, Secret>) -> Self {
        let mut nonce = [0u8; 6];
        getrandom::fill(&mut nonce).expect("OS randomness");
        let values = granted
            .iter()
            .filter(|(key, _)| {
                let ok = !key.is_empty() && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
                if !ok {
                    eprintln!("ignoring credential with invalid key {key:?} (allowed: A-Z a-z 0-9 _)");
                }
                ok
            })
            .map(|(key, secret)| (key.clone(), secret.0.clone()))
            .collect();
        Self { nonce: nonce.iter().map(|b| format!("{b:02x}")).collect(), values }
    }

    pub fn placeholder(&self, key: &str) -> Option<String> {
        self.values.contains_key(key).then(|| format!("zoocred-{}-{key}-", self.nonce))
    }

    fn fill(&self, text: &str, encode: fn(&str) -> String) -> String {
        self.values.iter().fold(text.to_string(), |text, (key, value)| {
            text.replace(&format!("zoocred-{}-{key}-", self.nonce), &encode(value))
        })
    }

    fn fill_bytes(&self, bytes: &[u8], encode: fn(&str) -> String) -> Vec<u8> {
        self.values.iter().fold(bytes.to_vec(), |bytes, (key, value)| {
            replace_bytes(&bytes, format!("zoocred-{}-{key}-", self.nonce).as_bytes(), encode(value).as_bytes())
        })
    }

    fn scrub(&self, bytes: &[u8]) -> Vec<u8> {
        self.values.iter().filter(|(_, value)| value.len() >= MIN_SCRUB_LEN).fold(bytes.to_vec(), |bytes, (key, value)| {
            replace_bytes(&bytes, value.as_bytes(), format!("zoocred-{}-{key}-", self.nonce).as_bytes())
        })
    }

    fn scrub_text(&self, text: &str) -> String {
        String::from_utf8_lossy(&self.scrub(text.as_bytes())).into_owned()
    }

    /// This call's placeholders as `{{credential:<key>}}`, stable across calls (for cassettes).
    fn normalize(&self, text: &str) -> String {
        self.values.keys().fold(text.to_string(), |text, key| {
            text.replace(&format!("zoocred-{}-{key}-", self.nonce), &format!("{{{{credential:{key}}}}}"))
        })
    }

    /// The reverse of `normalize`, with this call's placeholders.
    fn denormalize(&self, text: &str) -> String {
        self.values.keys().fold(text.to_string(), |text, key| {
            text.replace(&format!("{{{{credential:{key}}}}}"), &format!("zoocred-{}-{key}-", self.nonce))
        })
    }
}

/// Where requests go.
pub enum Transport {
    Live,
    /// Live, keeping every exchange for a cassette.
    Record(RefCell<Vec<Exchange>>),
    /// From a cassette; an exchange is used once. Nothing reaches the network.
    Replay(RefCell<Vec<Option<Exchange>>>),
}

impl Transport {
    pub fn replay(cassette: &Cassette) -> Self {
        Transport::Replay(RefCell::new(cassette.exchanges.iter().cloned().map(Some).collect()))
    }
}

/// One HTTP hop's answer, credentials still in place.
struct Hop {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn raw(value: &str) -> String {
    value.to_string()
}

fn component(value: &str) -> String {
    utf8_percent_encode(value, COMPONENT).to_string()
}

fn form(value: &str) -> String {
    deno_core::url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn json(value: &str) -> String {
    let quoted = serde_json::to_string(value).expect("strings serialize");
    quoted[1..quoted.len() - 1].to_string()
}

fn replace_bytes(haystack: &[u8], needle: &[u8], with: &[u8]) -> Vec<u8> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return haystack.to_vec();
    }
    let mut out = Vec::with_capacity(haystack.len());
    let mut i = 0;
    while i < haystack.len() {
        if haystack[i..].starts_with(needle) {
            out.extend_from_slice(with);
            i += needle.len();
        } else {
            out.push(haystack[i]);
            i += 1;
        }
    }
    out
}

/// One per `resolve` call: its own cookie jar, like a private browser session.
pub struct Net {
    client: reqwest::Client,
    policy: Arc<Policy>,
    jar: Arc<Jar>,
    pub credentials: Credentials,
    transport: Transport,
    trace: Option<Rc<Trace>>,
}

impl Net {
    pub fn new(
        policy: Policy,
        credentials: Credentials,
        transport: Transport,
        trace: Option<Rc<Trace>>,
    ) -> Result<Self, String> {
        let policy = Arc::new(policy);
        let jar = Arc::new(Jar::default());
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_provider(jar.clone())
            // A proxy would resolve names itself, out of the Policy's sight.
            .no_proxy()
            // Like the download core: Google resets HTTP/2 streams from clients whose
            // User-Agent claims a browser (modules may set one). Explicit, because workspace
            // feature unification can turn HTTP/2 on in dev builds.
            .http1_only()
            .dns_resolver(Arc::new(PolicyResolver(policy.clone())))
            .connect_timeout(Duration::from_secs(15))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| format!("cannot build HTTP client: {e}"))?;
        Ok(Self { client, policy, jar, credentials, transport, trace })
    }

    /// Exchanges recorded so far (`Transport::Record`), ready for a cassette.
    pub fn recorded(&self) -> Vec<Exchange> {
        match &self.transport {
            Transport::Record(exchanges) => exchanges.borrow().clone(),
            _ => Vec::new(),
        }
    }

    fn trace(&self, method: &Method, url: &str, outcome: FetchOutcome, replayed: bool) {
        if let Some(trace) = &self.trace {
            trace.push(Step::Fetch { method: method.to_string(), url: url.to_string(), outcome, replayed });
        }
    }

    /// Send one request (no redirect following). `shown` is the URL as the module may see it
    /// (placeholders, not values): what traces and cassettes record.
    async fn hop(
        &self,
        method: &Method,
        url: &Url,
        shown: &str,
        headers: &HeaderMap,
        body: Option<&Vec<u8>>,
    ) -> Result<Hop, String> {
        let shown = self.credentials.normalize(shown);
        if let Transport::Replay(exchanges) = &self.transport {
            let mut exchanges = exchanges.borrow_mut();
            let found = exchanges
                .iter_mut()
                .find(|e| e.as_ref().is_some_and(|e| e.method == method.as_str() && e.url == shown))
                .and_then(Option::take);
            let Some(exchange) = found else {
                let message = format!(
                    "fetch {method} {shown} is not in the cassette; the module now makes a request it did not make \
                     when recording (run zoo-module test --record to record again)"
                );
                self.trace(method, &shown, FetchOutcome::Failed(message.clone()), true);
                return Err(message);
            };
            let headers: Vec<(String, String)> =
                exchange.headers.iter().map(|(k, v)| (k.clone(), self.credentials.denormalize(v))).collect();
            for (_, cookie) in headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie")) {
                self.jar.add_cookie_str(cookie, url);
            }
            let body = match &exchange.body {
                Some(text) => self.credentials.denormalize(text).into_bytes(),
                None => exchange.body_bytes(),
            };
            self.trace(method, &shown, FetchOutcome::Status(exchange.status), true);
            return Ok(Hop { status: exchange.status, headers, body });
        }

        let mut builder = self.client.request(method.clone(), url.clone()).headers(headers.clone());
        if let Some(body) = body {
            builder = builder.body(body.clone());
        }
        let mut response = match builder.send().await {
            Ok(response) => response,
            Err(e) => {
                let message = describe(url, &e);
                self.trace(method, &shown, FetchOutcome::Failed(message.clone()), false);
                return Err(message);
            }
        };
        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned()))
            .collect();
        let mut body = Vec::new();
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) if body.len() + chunk.len() > MAX_BODY => {
                    let message = format!("fetch {url}: response larger than {} MB", MAX_BODY / 1024 / 1024);
                    self.trace(method, &shown, FetchOutcome::Failed(message.clone()), false);
                    return Err(message);
                }
                Ok(Some(chunk)) => body.extend_from_slice(&chunk),
                Ok(None) => break,
                Err(e) => {
                    let message = describe(url, &e);
                    self.trace(method, &shown, FetchOutcome::Failed(message.clone()), false);
                    return Err(message);
                }
            }
        }
        self.trace(method, &shown, FetchOutcome::Status(status), false);
        if let Transport::Record(exchanges) = &self.transport {
            let credentials = &self.credentials;
            let mut exchange = Exchange {
                method: method.to_string(),
                url: shown.clone(),
                status,
                headers: headers.iter().map(|(k, v)| (k.clone(), credentials.normalize(&credentials.scrub_text(v)))).collect(),
                body: None,
                body_base64: None,
            };
            let scrubbed = credentials.scrub(&body);
            match std::str::from_utf8(&scrubbed) {
                Ok(text) => exchange.set_body(credentials.normalize(text).as_bytes()),
                Err(_) => exchange.set_body(&scrubbed),
            }
            exchanges.borrow_mut().push(exchange);
        }
        Ok(Hop { status, headers, body })
    }

    /// Fills credential placeholders in the result and carries the session's cookies over to
    /// the download, unless the module set its own Cookie header.
    pub fn finish(&self, resolved: &mut Resolved) {
        resolved.url = self.credentials.fill(&resolved.url, component);
        for value in resolved.headers.values_mut() {
            *value = self.credentials.fill(value, raw);
        }
        if resolved.headers.keys().any(|k| k.eq_ignore_ascii_case("cookie")) {
            return;
        }
        let cookies = Url::parse(&resolved.url).ok().and_then(|url| self.jar.cookies(&url));
        if let Some(cookies) = cookies.and_then(|v| v.to_str().ok().map(String::from)) {
            resolved.headers.insert("Cookie".into(), cookies);
        }
    }

    pub async fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, String> {
        let credentials = &self.credentials;
        let target = credentials.fill(&request.url, component);
        let mut url = Url::parse(&target).map_err(|e| format!("fetch: invalid url {:?}: {e}", request.url))?;
        let mut method =
            Method::from_bytes(request.method.as_bytes()).map_err(|_| format!("fetch: bad method {}", request.method))?;
        let mut headers = HeaderMap::new();
        for (name, value) in &request.headers {
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| format!("fetch: bad header name {name:?}"))?;
            let value = HeaderValue::from_str(&credentials.fill(value, raw))
                .map_err(|_| format!("fetch: bad value for header {name}"))?;
            headers.append(name, value);
        }
        let content_type = headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_ascii_lowercase();
        let encode: fn(&str) -> String = if content_type.starts_with("application/x-www-form-urlencoded") {
            form
        } else if content_type.contains("json") {
            json
        } else {
            raw
        };
        let mut body = request.body.map(|body| credentials.fill_bytes(&body, encode));
        let mut redirected = false;
        // The URL as the module wrote it (placeholders, not values), for traces and cassettes.
        let mut shown = request.url.clone();

        for _ in 0..=MAX_REDIRECTS {
            if let Err(blocked) = self.policy.check_url(&url) {
                self.trace(&method, &credentials.normalize(&shown), FetchOutcome::Blocked(blocked.clone()), false);
                return Err(blocked);
            }
            let hop = self.hop(&method, &url, &shown, &headers, body.as_ref()).await?;
            let status = StatusCode::from_u16(hop.status).map_err(|_| format!("fetch {url}: bad status {}", hop.status))?;
            let location =
                hop.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("location")).map(|(_, v)| v.as_str());

            if status.is_redirection() && request.redirect != Redirect::Manual {
                if let Some(location) = location {
                    if request.redirect == Redirect::Error {
                        return Err(format!("fetch {url}: redirected while redirect mode is \"error\""));
                    }
                    let next = url.join(location).map_err(|e| format!("fetch {url}: bad redirect location: {e}"))?;
                    if status == StatusCode::SEE_OTHER
                        || (matches!(status, StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND) && method == Method::POST)
                    {
                        method = Method::GET;
                        body = None;
                        for name in ["content-type", "content-length", "content-encoding", "content-language"] {
                            headers.remove(name);
                        }
                    }
                    if next.origin() != url.origin() {
                        headers.remove(reqwest::header::AUTHORIZATION);
                    }
                    shown = credentials.scrub_text(next.as_str());
                    url = next;
                    redirected = true;
                    continue;
                }
            }
            // Module code sees placeholders wherever a credential value comes back.
            return Ok(FetchResponse {
                status: hop.status,
                status_text: status.canonical_reason().unwrap_or("").to_string(),
                url: credentials.scrub_text(url.as_str()),
                redirected,
                headers: hop.headers.iter().map(|(k, v)| (k.clone(), credentials.scrub_text(v))).collect(),
                body: credentials.scrub(&hop.body),
            });
        }
        Err(format!("fetch {url}: more than {MAX_REDIRECTS} redirects"))
    }
}

/// reqwest hides the useful part (our resolver's refusal, the OS error) in the source chain.
fn describe(url: &Url, error: &reqwest::Error) -> String {
    let mut message = format!("fetch {url} failed");
    let mut source: Option<&dyn std::error::Error> = Some(error);
    while let Some(e) = source {
        message.push_str(&format!(": {e}"));
        source = e.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(domains: &[&str]) -> Policy {
        Policy::new(&domains.iter().map(|d| d.to_string()).collect::<Vec<_>>(), false)
    }

    fn check(p: &Policy, url: &str) -> Result<(), String> {
        p.check_url(&Url::parse(url).unwrap())
    }

    #[test]
    fn allowlist_covers_domain_and_subdomains_only() {
        let p = policy(&["K2S.cc", "*.cdn.example"]);
        assert!(check(&p, "https://k2s.cc/file").is_ok());
        assert!(check(&p, "https://api.k2s.cc/file").is_ok());
        assert!(check(&p, "https://K2S.CC./x").is_ok());
        assert!(check(&p, "https://a.cdn.example/x").is_ok());
        assert!(check(&p, "https://notk2s.cc/").is_err());
        assert!(check(&p, "https://k2s.cc.evil.com/").is_err());
        let err = check(&p, "https://evil.com/").unwrap_err();
        assert!(err.contains("allowDomains") && err.contains("\"evil.com\""), "{err}");
    }

    #[test]
    fn non_http_schemes_are_blocked() {
        let p = policy(&["k2s.cc"]);
        for url in ["file:///etc/passwd", "ftp://k2s.cc/", "data:text/plain,hi"] {
            assert!(check(&p, url).is_err(), "{url}");
        }
    }

    #[test]
    fn private_ip_literals_are_blocked_even_when_listed() {
        let p = policy(&["127.0.0.1", "10.0.0.1", "169.254.169.254", "[::1]", "8.8.8.8"]);
        for url in ["http://127.0.0.1/", "http://10.0.0.1/", "http://169.254.169.254/", "http://[::1]/", "http://0x7f000001/"] {
            assert!(check(&p, url).is_err(), "{url}");
        }
        assert!(check(&p, "http://8.8.8.8/").is_ok());
    }

    #[test]
    fn address_classification() {
        let blocked = [
            "0.0.0.0", "127.0.0.1", "10.1.2.3", "172.16.0.1", "192.168.1.1", "169.254.169.254", "100.64.0.1",
            "192.0.0.1", "198.18.0.1", "224.0.0.1", "255.255.255.255", "240.0.0.1", "::", "::1", "fc00::1",
            "fd12::1", "fe80::1", "ff02::1", "::ffff:127.0.0.1", "::ffff:10.0.0.1", "64:ff9b::a00:1", "2001:db8::1",
            "2002::1", "2001::1", "::127.0.0.1",
        ];
        for ip in blocked {
            assert!(!is_public(ip.parse().unwrap()), "{ip} should be blocked");
        }
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111", "::ffff:8.8.8.8", "64:ff9b::808:808"] {
            assert!(is_public(ip.parse().unwrap()), "{ip} should be allowed");
        }
    }

    #[test]
    fn loopback_escape_hatch_is_loopback_only() {
        let p = Policy::new(&["127.0.0.1".into()], true);
        assert!(check(&p, "http://127.0.0.1:8080/").is_ok());
        assert!(!p.ip_allowed("10.0.0.1".parse().unwrap()));
        assert!(!p.ip_allowed("169.254.169.254".parse().unwrap()));
    }
}
