// Site module system: load JS site modules and match URLs to them.
// See docs/architecture/review-2026-10-30.md (area B) for what is and isn't wired yet.
pub mod captcha;
pub mod credential_proxy;
pub mod js_module;
pub mod js_runtime;
pub mod loader;
pub mod manifest;
pub mod registry;
pub mod sandbox;
pub mod url_detector;

#[cfg(test)]
pub mod tests;
