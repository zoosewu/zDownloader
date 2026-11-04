//! One `resolve` call: a fresh V8 isolate with a heap cap and a time budget, the module loaded
//! as ESM, its default export's `resolve(url, zoo)` awaited, the result validated.
//! A fresh isolate per call means nothing leaks between calls and a module that blows a limit
//! takes down only its own isolate.

use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use deno_core::error::{CoreError, ModuleLoaderError};
use deno_core::{
    v8, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, PollEventLoopOptions, ResolutionKind, RuntimeOptions,
};
use serde::Deserialize;
use zoo_module_client::protocol::{Limits, ModuleError, ModuleRef, Resolved};

use crate::net::{Net, Policy};
use crate::ops;

/// Host-wide settings from the `serve` command line.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostOptions {
    /// Test-only: let fetch reach loopback fixtures.
    pub allow_loopback: bool,
}

// Evaluated in this order in every isolate.
const VENDOR: &str = include_str!("gen/vendor.js");
const WEB: &str = include_str!("web.js");
const BOOTSTRAP: &str = include_str!("bootstrap.js");
/// What `import { defineModule } from 'zoo-module'` gets: identity, there for the types.
const SDK: &str = "export const defineModule = (module) => module;\n";
const SDK_SPECIFIER: &str = "zoo:sdk";
const MAIN_SPECIFIER: &str = "zoo:main";

const SHAPE_HINT: &str = "export default defineModule({ async resolve(url, zoo) { ... } })";
const RESULT_HINT: &str = "resolve must return { url, filename?, headers?, size? } with an absolute http(s) url";

pub fn resolve(module: &ModuleRef, url: &str, limits: Limits, options: HostOptions) -> Result<Resolved, ModuleError> {
    let entry = std::path::absolute(&module.entry)
        .ok()
        .and_then(|path| ModuleSpecifier::from_file_path(path).ok())
        .ok_or_else(|| load_error(format!("bad module path {}", module.entry.display())))?;
    let code = std::fs::read_to_string(&module.entry)
        .map_err(|e| load_error(format!("cannot read {}: {e}", module.entry.display())))?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| ModuleError::Crashed { message: format!("cannot start event loop: {e}") })?
        .block_on(async {
            let net = Net::new(Policy::new(&module.grants.allow_domains, options.allow_loopback))
                .map_err(|message| ModuleError::Crashed { message })?;
            run(entry, code, net, url, limits).await
        })
}

/// Why a call was cut short; set by the heap callback or the watchdog.
const TRIP_NONE: u8 = 0;
const TRIP_TIMEOUT: u8 = 1;
const TRIP_MEMORY: u8 = 2;

async fn run(entry: ModuleSpecifier, code: String, net: Net, url: &str, limits: Limits) -> Result<Resolved, ModuleError> {
    let heap_bytes = limits.heap_mb as usize * 1024 * 1024;
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(Loader { entry: entry.clone(), code })),
        create_params: Some(v8::CreateParams::default().heap_limits(0, heap_bytes)),
        extensions: vec![ops::zoo_host::init()],
        ..Default::default()
    });
    runtime.op_state().borrow_mut().put(Rc::new(net));
    let tripped = Arc::new(AtomicU8::new(TRIP_NONE));
    let isolate = runtime.v8_isolate().thread_safe_handle();
    {
        let (isolate, tripped) = (isolate.clone(), tripped.clone());
        runtime.add_near_heap_limit_callback(move |current, _initial| {
            let _ = tripped.compare_exchange(TRIP_NONE, TRIP_MEMORY, Ordering::SeqCst, Ordering::SeqCst);
            isolate.terminate_execution();
            // Headroom for V8 to unwind; the terminated call never gets to use it.
            current * 2
        });
    }
    let budget = Duration::from_millis(limits.timeout_ms);
    // Synchronous JS (a busy loop) blocks this thread, so only another thread can stop it.
    let _watchdog = Watchdog::start(isolate, budget, tripped.clone());
    // Waiting on host work (no JS running) is cut off here.
    let outcome = tokio::time::timeout(budget, execute(&mut runtime, &entry, url)).await;

    match (tripped.load(Ordering::SeqCst), outcome) {
        (TRIP_MEMORY, _) => Err(ModuleError::OutOfMemory { limit_mb: limits.heap_mb }),
        (TRIP_TIMEOUT, _) | (_, Err(_)) => Err(ModuleError::Timeout { limit_ms: limits.timeout_ms }),
        (_, Ok(result)) => result,
    }
}

async fn execute(runtime: &mut JsRuntime, entry: &ModuleSpecifier, url: &str) -> Result<Resolved, ModuleError> {
    for (name, code) in [("zoo:vendor", VENDOR), ("zoo:web", WEB), ("zoo:bootstrap", BOOTSTRAP)] {
        runtime
            .execute_script(name, code)
            .map_err(|e| ModuleError::Crashed { message: format!("{name} failed: {e}") })?;
    }

    let main = ModuleSpecifier::parse(MAIN_SPECIFIER).expect("valid specifier");
    let wrapper = format!(
        "import module from {};\nglobalThis.__zooResult = __zooRun(module, {});\n",
        serde_json::to_string(entry.as_str()).expect("string"),
        serde_json::to_string(url).expect("string"),
    );
    let id = runtime.load_main_es_module_from_code(&main, wrapper).await.map_err(module_load_error)?;
    let evaluated = runtime.mod_evaluate(id);
    runtime
        .with_event_loop_promise(Box::pin(evaluated), PollEventLoopOptions::default())
        .await
        .map_err(module_load_error)?;

    let pending = runtime.execute_script("zoo:result", "globalThis.__zooResult").map_err(|e| script_error(*e))?;
    let settled = runtime.resolve(pending);
    let value = runtime
        .with_event_loop_promise(Box::pin(settled), PollEventLoopOptions::default())
        .await
        .map_err(|e| match *e.0 {
            deno_core::error::CoreErrorKind::Js(e) => script_error(*e),
            deno_core::error::CoreErrorKind::PendingPromiseResolution => ModuleError::Script {
                message: "resolve never settled: it awaits something that can no longer happen".into(),
                hint: None,
                stack: None,
            },
            other => ModuleError::Script { message: other.to_string(), hint: None, stack: None },
        })?;
    let json = {
        deno_core::scope!(scope, runtime);
        v8::Local::new(scope, value).to_rust_string_lossy(scope)
    };
    outcome(&json)
}

/// The JSON `__zooRun` settles with (see bootstrap.js).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Outcome {
    Ok(serde_json::Value),
    Err { message: String, hint: Option<String>, stack: Option<String> },
    Invalid(String),
}

fn outcome(json: &str) -> Result<Resolved, ModuleError> {
    match serde_json::from_str::<Outcome>(json) {
        Ok(Outcome::Ok(value)) => validate(value),
        Ok(Outcome::Err { message, hint, stack }) => Err(ModuleError::Script { message, hint, stack }),
        Ok(Outcome::Invalid(why)) => Err(ModuleError::Load { message: why, hint: Some(SHAPE_HINT.into()) }),
        Err(e) => Err(ModuleError::Crashed { message: format!("unreadable module outcome: {e}") }),
    }
}

fn validate(value: serde_json::Value) -> Result<Resolved, ModuleError> {
    let invalid = |message: String| ModuleError::Script { message, hint: Some(RESULT_HINT.into()), stack: None };
    let resolved: Resolved =
        serde_json::from_value(value).map_err(|e| invalid(format!("resolve returned an invalid result: {e}")))?;
    match deno_core::url::Url::parse(&resolved.url) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(resolved),
        _ => Err(invalid(format!("resolve returned a url that is not absolute http(s): {:?}", resolved.url))),
    }
}

fn load_error(message: String) -> ModuleError {
    ModuleError::Load { message, hint: None }
}

fn module_load_error(e: CoreError) -> ModuleError {
    let message = e.to_string();
    let hint = message.contains("export named 'default'").then(|| SHAPE_HINT.to_string());
    ModuleError::Load { message, hint }
}

fn script_error(e: deno_core::error::JsError) -> ModuleError {
    ModuleError::Script { message: e.exception_message.clone(), hint: None, stack: e.stack.clone() }
}

/// Serves exactly three modules: the generated entry wrapper, the module file, and the SDK.
/// Anything else is refused: modules ship as a single bundled file.
struct Loader {
    entry: ModuleSpecifier,
    code: String,
}

impl ModuleLoader for Loader {
    fn resolve(&self, specifier: &str, _referrer: &str, _kind: ResolutionKind) -> Result<ModuleSpecifier, ModuleLoaderError> {
        if specifier == "zoo-module" {
            return Ok(ModuleSpecifier::parse(SDK_SPECIFIER).expect("valid specifier"));
        }
        if specifier == self.entry.as_str() || specifier == MAIN_SPECIFIER {
            return ModuleSpecifier::parse(specifier).map_err(ModuleLoaderError::from_err);
        }
        Err(ModuleLoaderError::generic(format!(
            "cannot import '{specifier}': bundle the module into a single file; only 'zoo-module' can be imported"
        )))
    }

    fn load(
        &self,
        specifier: &ModuleSpecifier,
        _referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let code = if specifier == &self.entry {
            self.code.clone()
        } else if specifier.as_str() == SDK_SPECIFIER {
            SDK.to_string()
        } else {
            return ModuleLoadResponse::Sync(Err(ModuleLoaderError::generic(format!("no module at {specifier}"))));
        };
        ModuleLoadResponse::Sync(Ok(ModuleSource::new(
            ModuleType::JavaScript,
            ModuleSourceCode::String(code.into()),
            specifier,
            None,
        )))
    }
}

/// Terminates the isolate when the budget runs out. Dropping it disarms it.
struct Watchdog(#[allow(dead_code)] mpsc::Sender<()>);

impl Watchdog {
    fn start(isolate: v8::IsolateHandle, budget: Duration, tripped: Arc<AtomicU8>) -> Self {
        let (disarm, armed) = mpsc::channel::<()>();
        thread::spawn(move || {
            if armed.recv_timeout(budget) == Err(mpsc::RecvTimeoutError::Timeout) {
                let _ = tripped.compare_exchange(TRIP_NONE, TRIP_TIMEOUT, Ordering::SeqCst, Ordering::SeqCst);
                isolate.terminate_execution();
            }
        });
        Watchdog(disarm)
    }
}
