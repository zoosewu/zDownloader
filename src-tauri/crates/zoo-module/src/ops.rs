//! Host functions behind the Web APIs in web.js. Module code cannot call these directly:
//! bootstrap.js removes `Deno.core` before the module loads.

use std::cell::RefCell;
use std::rc::Rc;

use deno_core::{op2, JsBuffer, OpState, ToJsBuffer};
use deno_error::JsErrorBox;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::net::{FetchRequest, Net, Redirect};

deno_core::extension!(zoo_host, ops = [op_zoo_fetch, op_zoo_random, op_zoo_digest]);

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
