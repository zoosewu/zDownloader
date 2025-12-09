// T138: Unified error message formatting with module context
use super::interface::ModuleError;

/// Format a module error with module context for user-friendly display
///
/// This function ensures all error messages include the module name,
/// making it clear to users which module encountered the error.
pub fn format_module_error(module_name: &str, error: &ModuleError) -> String {
    let base_message = match error {
        ModuleError::UnsupportedUrl(url) => format!("Unsupported URL: {}", url),
        ModuleError::NetworkError(msg) => format!("Network error: {}", msg),
        ModuleError::CaptchaRequired(_) => "CAPTCHA verification required".to_string(),
        ModuleError::AuthRequired => "Authentication required".to_string(),
        ModuleError::InvalidCredentials => "Invalid credentials".to_string(),
        ModuleError::ParseError(msg) => format!("Parse error: {}", msg),
        ModuleError::InternalError(msg) => format!("Internal error: {}", msg),
    };

    format!("[Module: {}] {}", module_name, base_message)
}

/// Check if an error is recoverable (can retry after user action)
pub fn is_recoverable_error(error: &ModuleError) -> bool {
    matches!(
        error,
        ModuleError::NetworkError(_) |
        ModuleError::CaptchaRequired(_) |
        ModuleError::AuthRequired |
        ModuleError::InvalidCredentials
    )
}

/// Suggest recovery action for an error
pub fn suggest_recovery_action(error: &ModuleError) -> &'static str {
    match error {
        ModuleError::NetworkError(_) => "Check your internet connection and retry",
        ModuleError::CaptchaRequired(_) => "Complete the CAPTCHA challenge",
        ModuleError::AuthRequired | ModuleError::InvalidCredentials => {
            "Update your login credentials in settings"
        }
        ModuleError::ParseError(_) => "The website format may have changed. Contact support",
        ModuleError::UnsupportedUrl(_) => "This URL is not supported by the module",
        ModuleError::InternalError(_) => "An internal error occurred. Try again or contact support",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_format_module_error_includes_module_name() {
        let error = ModuleError::NetworkError("Connection timeout".to_string());
        let formatted = format_module_error("k2s", &error);

        assert!(formatted.contains("[Module: k2s]"));
        assert!(formatted.contains("Connection timeout"));
    }

    #[test]
    fn test_is_recoverable_error() {
        assert!(is_recoverable_error(&ModuleError::NetworkError("test".to_string())));
        assert!(is_recoverable_error(&ModuleError::AuthRequired));
        assert!(!is_recoverable_error(&ModuleError::UnsupportedUrl("test".to_string())));
    }

    #[test]
    fn test_suggest_recovery_action() {
        let error = ModuleError::AuthRequired;
        let suggestion = suggest_recovery_action(&error);

        assert!(suggestion.contains("credentials"));
    }
}
