//! The only way a module reaches the network. Every hop of every request passes the Policy:
//! http(s) only, host on the module's allowlist, and (after DNS, so rebinding cannot dodge it)
//! a public address. Redirects are followed here, hop by hop, never by reqwest.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use deno_core::url::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
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

/// One per `resolve` call: its own cookie jar, like a private browser session.
pub struct Net {
    client: reqwest::Client,
    policy: Arc<Policy>,
}

impl Net {
    pub fn new(policy: Policy) -> Result<Self, String> {
        let policy = Arc::new(policy);
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            // A proxy would resolve names itself, out of the Policy's sight.
            .no_proxy()
            .dns_resolver(Arc::new(PolicyResolver(policy.clone())))
            .connect_timeout(Duration::from_secs(15))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| format!("cannot build HTTP client: {e}"))?;
        Ok(Self { client, policy })
    }

    pub async fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, String> {
        let mut url = Url::parse(&request.url).map_err(|e| format!("fetch: invalid url {:?}: {e}", request.url))?;
        let mut method =
            Method::from_bytes(request.method.as_bytes()).map_err(|_| format!("fetch: bad method {}", request.method))?;
        let mut headers = HeaderMap::new();
        for (name, value) in &request.headers {
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| format!("fetch: bad header name {name:?}"))?;
            let value = HeaderValue::from_str(value).map_err(|_| format!("fetch: bad value for header {name}"))?;
            headers.append(name, value);
        }
        let mut body = request.body;
        let mut redirected = false;

        for _ in 0..=MAX_REDIRECTS {
            self.policy.check_url(&url)?;
            let mut builder = self.client.request(method.clone(), url.clone()).headers(headers.clone());
            if let Some(body) = &body {
                builder = builder.body(body.clone());
            }
            let response = builder.send().await.map_err(|e| describe(&url, &e))?;
            let status = response.status();
            let location = response.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok());

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
                    url = next;
                    redirected = true;
                    continue;
                }
            }
            return read(response, &url, redirected).await;
        }
        Err(format!("fetch {url}: more than {MAX_REDIRECTS} redirects"))
    }
}

async fn read(mut response: reqwest::Response, url: &Url, redirected: bool) -> Result<FetchResponse, String> {
    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned()))
        .collect();
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| describe(url, &e))? {
        if body.len() + chunk.len() > MAX_BODY {
            return Err(format!("fetch {url}: response larger than {} MB", MAX_BODY / 1024 / 1024));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(FetchResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_string(),
        url: url.to_string(),
        redirected,
        headers,
        body,
    })
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
