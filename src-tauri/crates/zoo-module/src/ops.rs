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
use zoo_module_client::manifest::Text;
use zoo_module_client::protocol::Captcha;

use crate::budget::Budget;
use crate::hooks::Hooks;
use crate::runtime::ApiLevel;
use crate::net::{FetchRequest, Net, Redirect};

/// Longest single `zoo.wait`.
const MAX_WAIT: Duration = Duration::from_secs(600);
/// How long a captcha may wait for the user.
const MAX_CAPTCHA_WAIT: Duration = Duration::from_secs(900);

deno_core::extension!(
    zoo_host,
    ops = [op_zoo_fetch, op_zoo_random, op_zoo_digest, op_zoo_credential, op_zoo_wait, op_zoo_captcha, op_zoo_log]
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

#[op2]
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

#[op2]
async fn op_zoo_wait(state: Rc<RefCell<OpState>>, ms: f64, #[string] reason: String) -> Result<(), JsErrorBox> {
    let ms = ms.max(0.0) as u64;
    let reason = (!reason.is_empty()).then_some(reason);
    let wait = Duration::from_millis(ms);
    if wait > MAX_WAIT {
        return Err(JsErrorBox::range_error(format!("zoo.wait: at most {} seconds", MAX_WAIT.as_secs())));
    }
    let (budget, hooks) = {
        let state = state.borrow();
        (state.borrow::<Arc<Budget>>().clone(), state.borrow::<Rc<dyn Hooks>>().clone())
    };
    if hooks.waiting(ms, reason) {
        let _paused = budget.pause();
        tokio::time::sleep(wait).await;
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum CaptchaArgs {
    Recaptcha { site_key: String, page_url: String },
    Image { url: String },
    Ask { id: String, message: Text, #[serde(default)] secret: bool, #[serde(default)] remember: bool, error: Option<Text> },
}

/// `zoo.ask`'s arguments, as a module author needs to hear about mistakes in them.
fn check_ask(id: &str, message: &Text, error: Option<&Text>) -> Result<(), String> {
    let id_ok = !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !id_ok {
        return Err(format!("zoo.ask: id {id:?} must be 1 to 64 letters, digits, _ or -"));
    }
    message.check("zoo.ask: message", 300)?;
    error.map_or(Ok(()), |e| e.check("zoo.ask: error", 300))
}

/// The user's solution, or null if they declined.
#[op2]
#[string]
async fn op_zoo_captcha(state: Rc<RefCell<OpState>>, #[serde] args: CaptchaArgs) -> Result<Option<String>, JsErrorBox> {
    let (budget, hooks, net) = {
        let state = state.borrow();
        (state.borrow::<Arc<Budget>>().clone(), state.borrow::<Rc<dyn Hooks>>().clone(), state.borrow::<Rc<Net>>().clone())
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
        CaptchaArgs::Ask { id, message, secret, remember, error } => {
            if state.borrow().borrow::<ApiLevel>().0 < 3 {
                return Err(JsErrorBox::type_error("zoo.ask needs \"apiVersion\": 3 in manifest.json"));
            }
            check_ask(&id, &message, error.as_ref()).map_err(JsErrorBox::type_error)?;
            Captcha::Ask { id, message, secret, remember, error }
        }
    };
    let answer = hooks.ask(challenge);
    let _paused = budget.pause();
    match tokio::time::timeout(MAX_CAPTCHA_WAIT, answer).await {
        Ok(solution) => Ok(solution.unwrap_or(None)),
        Err(_) => Ok(None),
    }
}

/// `console.*` and `zoo.log.*`.
#[op2(fast)]
fn op_zoo_log(state: &mut OpState, #[string] level: &str, #[string] message: &str) {
    state.borrow::<Rc<dyn Hooks>>().log(level, message);
}
