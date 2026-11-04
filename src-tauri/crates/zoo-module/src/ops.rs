//! Host functions behind the Web APIs in web.js. Module code cannot call these directly:
//! bootstrap.js removes `Deno.core` before the module loads.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use deno_core::{op2, JsBuffer, OpState, ToJsBuffer};
use base64::Engine;
use deno_error::JsErrorBox;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use zoo_module_client::protocol::{CallEvent, Captcha};

use crate::budget::Budget;
use crate::link::CallLink;
use crate::net::{FetchRequest, Net, Redirect};

/// Longest single `zoo.wait`.
const MAX_WAIT: Duration = Duration::from_secs(600);
/// How long a captcha may wait for the user.
const MAX_CAPTCHA_WAIT: Duration = Duration::from_secs(900);

deno_core::extension!(
    zoo_host,
    ops = [op_zoo_fetch, op_zoo_random, op_zoo_digest, op_zoo_credential, op_zoo_wait, op_zoo_captcha]
);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchArgs {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<JsBuffer>,
    redirect: Redirect,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchReply {
    status: u16,
    status_text: String,
    url: String,
    redirected: bool,
    headers: Vec<(String, String)>,
    body: ToJsBuffer,
}

#[op2(async)]
#[serde]
async fn op_zoo_fetch(state: Rc<RefCell<OpState>>, #[serde] args: FetchArgs) -> Result<FetchReply, JsErrorBox> {
    let net = state.borrow().borrow::<Rc<Net>>().clone();
    let request = FetchRequest {
        url: args.url,
        method: args.method,
        headers: args.headers,
        body: args.body.map(|b| b.to_vec()),
        redirect: args.redirect,
    };
    let response = net.fetch(request).await.map_err(JsErrorBox::type_error)?;
    Ok(FetchReply {
        status: response.status,
        status_text: response.status_text,
        url: response.url,
        redirected: response.redirected,
        headers: response.headers,
        body: response.body.into(),
    })
}

#[op2(fast)]
fn op_zoo_random(#[buffer] buf: &mut [u8]) -> Result<(), JsErrorBox> {
    getrandom::fill(buf).map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[op2]
#[buffer]
fn op_zoo_digest(#[string] algorithm: &str, #[buffer] data: &[u8]) -> Result<Vec<u8>, JsErrorBox> {
    Ok(match algorithm {
        "sha1" => Sha1::digest(data).to_vec(),
        "sha256" => Sha256::digest(data).to_vec(),
        "sha384" => Sha384::digest(data).to_vec(),
        "sha512" => Sha512::digest(data).to_vec(),
        other => return Err(JsErrorBox::type_error(format!("unsupported digest {other}"))),
    })
}

/// The placeholder for a granted credential, or null.
#[op2]
#[string]
fn op_zoo_credential(state: &mut OpState, #[string] key: &str) -> Option<String> {
    state.borrow::<Rc<Net>>().credentials.placeholder(key)
}

#[op2(async)]
async fn op_zoo_wait(state: Rc<RefCell<OpState>>, ms: f64, #[string] reason: String) -> Result<(), JsErrorBox> {
    let ms = ms.max(0.0) as u64;
    let reason = (!reason.is_empty()).then_some(reason);
    let wait = Duration::from_millis(ms);
    if wait > MAX_WAIT {
        return Err(JsErrorBox::range_error(format!("zoo.wait: at most {} seconds", MAX_WAIT.as_secs())));
    }
    let (budget, link) = {
        let state = state.borrow();
        (state.borrow::<Arc<Budget>>().clone(), state.borrow::<Rc<CallLink>>().clone())
    };
    link.event(CallEvent::Waiting { ms, reason });
    let _paused = budget.pause();
    tokio::time::sleep(wait).await;
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum CaptchaArgs {
    Recaptcha { site_key: String, page_url: String },
    Image { url: String },
}

/// The user's solution, or null if they declined.
#[op2(async)]
#[string]
async fn op_zoo_captcha(state: Rc<RefCell<OpState>>, #[serde] args: CaptchaArgs) -> Result<Option<String>, JsErrorBox> {
    let (budget, link, net) = {
        let state = state.borrow();
        (state.borrow::<Arc<Budget>>().clone(), state.borrow::<Rc<CallLink>>().clone(), state.borrow::<Rc<Net>>().clone())
    };
    let challenge = match args {
        CaptchaArgs::Recaptcha { site_key, page_url } => Captcha::Recaptcha { site_key, page_url },
        CaptchaArgs::Image { url } => {
            // Fetched here, through the Policy and the call's session: image captchas are tied to cookies.
            let image = net
                .fetch(FetchRequest { url, method: "GET".into(), headers: vec![], body: None, redirect: Redirect::Follow })
                .await
                .map_err(JsErrorBox::type_error)?;
            let mime = image
                .headers
                .iter()
                .find(|(k, _)| k == "content-type")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "application/octet-stream".into());
            Captcha::Image { mime, base64: base64::engine::general_purpose::STANDARD.encode(&image.body) }
        }
    };
    let answer = link.ask(challenge);
    let _paused = budget.pause();
    match tokio::time::timeout(MAX_CAPTCHA_WAIT, answer).await {
        Ok(solution) => Ok(solution.unwrap_or(None)),
        Err(_) => Ok(None),
    }
}
