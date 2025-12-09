// Modules system - dynamic module loading
pub mod auth;
pub mod browser;
pub mod captcha;
pub mod interface;
pub mod loader;
pub mod registry;
pub mod url_detector;

// Feature 004: Site Module Integration
pub mod audit;
pub mod context_api;
pub mod credential_proxy;
pub mod error_format;
pub mod init;
pub mod js_module;
pub mod js_runtime;
pub mod manifest;
pub mod network_monitor;
pub mod permission;
pub mod resource_limiter;
pub mod sandbox;
pub mod url_validator;

// Tests for Feature 004
#[cfg(test)]
pub mod tests;
