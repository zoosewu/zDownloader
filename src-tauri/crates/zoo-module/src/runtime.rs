//! One `resolve` call: a fresh V8 isolate with a heap cap and a time budget, the module loaded
//! as ESM, its default export's `resolve(url, zoo)` awaited, the result validated.
//! A fresh isolate per call means nothing leaks between calls and a module that blows a limit
//! takes down only its own isolate.

use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use deno_core::error::{CoreError, ModuleLoaderError};
use deno_core::{
    v8, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, PollEventLoopOptions, ResolutionKind, RuntimeOptions,
};
use serde::Deserialize;
use zoo_module_client::protocol::{Limits, ModuleError, ModuleRef, Resolved};

use crate::budget::{Budget, TRIP_CANCELLED, TRIP_MEMORY, TRIP_NONE, TRIP_TIMEOUT};
use crate::hooks::Hooks;
use crate::link::CallLink;
use crate::net::{Credentials, Net, Policy, Transport};
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
const RESULT_HINT: &str = "resolve must return { url, filename?, headers?, size?, maxConnections?, maxSegmentSize? } \
     with an absolute http(s) url";
/// Smaller segments would cost one request per few KiB.
const MIN_SEGMENT_SIZE: u64 = 64 * 1024;

/// What a call does with the module.
#[derive(Debug, Clone, Copy)]
pub enum Work<'a> {
    Resolve(&'a str),
    /// Load it and check its shape without calling `resolve` (`zoo-module check`).
    Load,
}

/// A `resolve` call for the app (`serve`).
pub fn resolve(
    module: &ModuleRef,
    url: &str,
    limits: Limits,
    options: HostOptions,
    link: CallLink,
) -> Result<Resolved, ModuleError> {
    let policy = Policy::new(&module.grants.allow_domains, options.allow_loopback);
    let net = Net::new(policy, Credentials::new(&module.grants.credentials), Transport::Live, None)
        .map_err(|message| ModuleError::Crashed { message })?;
    run_call(&module.entry, Work::Resolve(url), limits, Rc::new(net), Rc::new(link))
        .map(|resolved| resolved.expect("a resolve call yields a result"))
}

/// Runs `work` on the module at `entry` in a fresh isolate on this thread. `Some` result for
/// `Work::Resolve`, `None` for `Work::Load`.
pub fn run_call(
    entry: &Path,
    work: Work,
    limits: Limits,
    net: Rc<Net>,
    hooks: Rc<dyn Hooks>,
) -> Result<Option<Resolved>, ModuleError> {
    let specifier = std::path::absolute(entry)
        .ok()
        .and_then(|path| ModuleSpecifier::from_file_path(path).ok())
        .ok_or_else(|| load_error(format!("bad module path {}", entry.display())))?;
    let code =
        std::fs::read_to_string(entry).map_err(|e| load_error(format!("cannot read {}: {e}", entry.display())))?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| ModuleError::Crashed { message: format!("cannot start event loop: {e}") })?
        .block_on(run(specifier, code, net, hooks, work, limits))
}

async fn run(
    entry: ModuleSpecifier,
    code: String,
    net: Rc<Net>,
    hooks: Rc<dyn Hooks>,
    work: Work<'_>,
    limits: Limits,
) -> Result<Option<Resolved>, ModuleError> {
    let heap_bytes = limits.heap_mb as usize * 1024 * 1024;
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(Loader { entry: entry.clone(), code })),
        create_params: Some(v8::CreateParams::default().heap_limits(0, heap_bytes)),
        extensions: vec![ops::zoo_host::init()],
        ..Default::default()
    });
    runtime.op_state().borrow_mut().put(net.clone());
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
    let budget = Budget::start(Duration::from_millis(limits.timeout_ms), isolate, tripped.clone());
    runtime.op_state().borrow_mut().put(budget.clone());
    hooks.attach(budget.clone());
    runtime.op_state().borrow_mut().put(hooks);
    // The watchdog stops synchronous JS; this stops waiting on host work.
    let outcome = tokio::select! {
        result = execute(&mut runtime, &entry, work) => Some(result),
        _ = budget.expired() => None,
    };
    budget.finish();

    match (tripped.load(Ordering::SeqCst), outcome) {
        (TRIP_CANCELLED, _) => Err(ModuleError::Cancelled),
        (TRIP_MEMORY, _) => Err(ModuleError::OutOfMemory { limit_mb: limits.heap_mb }),
        (TRIP_TIMEOUT, _) | (_, None) => Err(ModuleError::Timeout { limit_ms: limits.timeout_ms }),
        (_, Some(result)) => result.map(|resolved| {
            resolved.map(|mut resolved| {
                net.finish(&mut resolved);
                resolved
            })
        }),
    }
}

async fn execute(runtime: &mut JsRuntime, entry: &ModuleSpecifier, work: Work<'_>) -> Result<Option<Resolved>, ModuleError> {
    for (name, code) in [("zoo:vendor", VENDOR), ("zoo:web", WEB), ("zoo:bootstrap", BOOTSTRAP)] {
        runtime
            .execute_script(name, code)
            .map_err(|e| ModuleError::Crashed { message: format!("{name} failed: {e}") })?;
    }

    let main = ModuleSpecifier::parse(MAIN_SPECIFIER).expect("valid specifier");
    let wrapper = format!(
        "import module from {};\nglobalThis.__zooResult = __zooRun(module, {});\n",
        serde_json::to_string(entry.as_str()).expect("string"),
        match work {
            Work::Resolve(url) => serde_json::to_string(url).expect("string"),
            Work::Load => "null".into(),
        },
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
    Checked(serde::de::IgnoredAny),
}

fn outcome(json: &str) -> Result<Option<Resolved>, ModuleError> {
    match serde_json::from_str::<Outcome>(json) {
        Ok(Outcome::Ok(value)) => validate(value).map(Some),
        Ok(Outcome::Checked(_)) => Ok(None),
        Ok(Outcome::Err { message, hint, stack }) => Err(ModuleError::Script { message, hint, stack }),
        Ok(Outcome::Invalid(why)) => Err(ModuleError::Load { message: why, hint: Some(SHAPE_HINT.into()) }),
        Err(e) => Err(ModuleError::Crashed { message: format!("unreadable module outcome: {e}") }),
    }
}

fn validate(value: serde_json::Value) -> Result<Resolved, ModuleError> {
    let invalid = |message: String| ModuleError::Script { message, hint: Some(RESULT_HINT.into()), stack: None };
    let resolved: Resolved =
        serde_json::from_value(value).map_err(|e| invalid(format!("resolve returned an invalid result: {e}")))?;
    if resolved.max_segment_size.is_some_and(|n| n < MIN_SEGMENT_SIZE) {
        return Err(invalid(format!(
            "resolve returned maxSegmentSize below {MIN_SEGMENT_SIZE} bytes; set it only to the largest range the site serves"
        )));
    }
    if resolved.max_connections == Some(0) {
        return Err(invalid("resolve returned maxConnections 0; use at least 1, or leave it out".into()));
    }
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
