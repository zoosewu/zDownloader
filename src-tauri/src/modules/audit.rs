// Audit Logging System (Feature 004)
//
// This module implements structured audit logging for all module actions,
// including network requests, permission violations, and security events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub module_id: String,
    pub event_type: String,
    pub severity: AuditSeverity,
    pub details: serde_json::Value,
}

/// Audit event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditSeverity {
    Info,
    Warn,
    Error,
    Critical,
}

use std::sync::Mutex;

/// Global audit log storage (in-memory for MVP)
static AUDIT_LOGS: Mutex<Option<Vec<AuditLogEntry>>> = Mutex::new(None);

/// Initialize audit logging system
pub fn init_audit_logging() -> Result<(), String> {
    // Initialize the in-memory audit log storage
    let mut logs = AUDIT_LOGS.lock().map_err(|e| format!("Failed to lock audit logs: {}", e))?;
    *logs = Some(Vec::new());

    // Configure tracing subscriber for structured logging
    // Use try_init to avoid panic if subscriber is already initialized (e.g., in tests)
    use tracing_subscriber::fmt;

    let _ = fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    Ok(())
}

/// Get all audit logs (for testing and UI)
pub fn get_all_audit_logs() -> Result<Vec<AuditLogEntry>, String> {
    let logs = AUDIT_LOGS.lock().map_err(|e| format!("Failed to lock audit logs: {}", e))?;

    if let Some(ref log_vec) = *logs {
        Ok(log_vec.clone())
    } else {
        Ok(Vec::new())
    }
}

/// Clear all audit logs (for testing)
#[cfg(test)]
pub fn clear_audit_logs() -> Result<(), String> {
    let mut logs = AUDIT_LOGS.lock().map_err(|e| format!("Failed to lock audit logs: {}", e))?;
    if let Some(ref mut log_vec) = *logs {
        log_vec.clear();
    }
    Ok(())
}

/// Log a network request
pub fn log_network_request(
    module_id: &str,
    url: &str,
    method: &str,
    status_code: u16,
) {
    let entry = AuditLogEntry {
        timestamp: chrono::Utc::now(),
        module_id: module_id.to_string(),
        event_type: "network_request".to_string(),
        severity: AuditSeverity::Info,
        details: serde_json::json!({
            "url": url,
            "method": method,
            "statusCode": status_code
        }),
    };

    if let Ok(mut logs) = AUDIT_LOGS.lock() {
        if let Some(ref mut log_vec) = *logs {
            log_vec.push(entry.clone());
        }
    }

    tracing::info!(
        module_id = module_id,
        event_type = "network_request",
        url = url,
        method = method,
        status_code = status_code,
        "Network request"
    );
}

/// Log a permission violation
pub fn log_permission_violation(
    module_id: &str,
    attempted_action: &str,
    reason: &str,
) {
    let entry = AuditLogEntry {
        timestamp: chrono::Utc::now(),
        module_id: module_id.to_string(),
        event_type: "permission_violation".to_string(),
        severity: AuditSeverity::Error,
        details: serde_json::json!({
            "attemptedAction": attempted_action,
            "reason": reason
        }),
    };

    if let Ok(mut logs) = AUDIT_LOGS.lock() {
        if let Some(ref mut log_vec) = *logs {
            log_vec.push(entry.clone());
        }
    }

    tracing::error!(
        module_id = module_id,
        event_type = "permission_violation",
        attempted_action = attempted_action,
        reason = reason,
        "Permission violation"
    );
}

/// Log a security event
pub fn log_security_event(
    module_id: &str,
    event_type: &str,
    severity: AuditSeverity,
    details: serde_json::Value,
) {
    let entry = AuditLogEntry {
        timestamp: chrono::Utc::now(),
        module_id: module_id.to_string(),
        event_type: event_type.to_string(),
        severity: severity.clone(),
        details,
    };

    if let Ok(mut logs) = AUDIT_LOGS.lock() {
        if let Some(ref mut log_vec) = *logs {
            log_vec.push(entry.clone());
        }
    }

    match severity {
        AuditSeverity::Info => tracing::info!(module_id = module_id, event_type = event_type, "Security event"),
        AuditSeverity::Warn => tracing::warn!(module_id = module_id, event_type = event_type, "Security event"),
        AuditSeverity::Error => tracing::error!(module_id = module_id, event_type = event_type, "Security event"),
        AuditSeverity::Critical => tracing::error!(module_id = module_id, event_type = event_type, severity = "CRITICAL", "Security event"),
    }
}

/// Retrieve audit logs for a module
pub fn get_module_audit_logs(
    module_id: &str,
    limit: Option<usize>,
    offset: Option<usize>,
    severity: Option<AuditSeverity>,
) -> Result<Vec<AuditLogEntry>, String> {
    let logs = AUDIT_LOGS.lock()
        .map_err(|e| format!("Failed to lock audit logs: {}", e))?;

    if let Some(ref log_vec) = *logs {
        let filtered: Vec<AuditLogEntry> = log_vec
            .iter()
            .filter(|entry| entry.module_id == module_id)
            .filter(|entry| {
                if let Some(ref sev) = severity {
                    // Compare severity levels
                    matches!(
                        (&entry.severity, sev),
                        (AuditSeverity::Info, AuditSeverity::Info) |
                        (AuditSeverity::Warn, AuditSeverity::Warn) |
                        (AuditSeverity::Error, AuditSeverity::Error) |
                        (AuditSeverity::Critical, AuditSeverity::Critical)
                    )
                } else {
                    true
                }
            })
            .skip(offset.unwrap_or(0))
            .take(limit.unwrap_or(100))
            .cloned()
            .collect();

        Ok(filtered)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            timestamp: chrono::Utc::now(),
            module_id: "test-module".to_string(),
            event_type: "network_request".to_string(),
            severity: AuditSeverity::Info,
            details: serde_json::json!({"url": "https://example.com"}),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify camelCase naming
        assert!(parsed.get("moduleId").is_some());
        assert!(parsed.get("eventType").is_some());
        assert!(parsed.get("module_id").is_none());
        assert!(parsed.get("event_type").is_none());
    }

    #[test]
    fn test_audit_severity_serialization() {
        // Test each severity level
        let severities = vec![
            (AuditSeverity::Info, "INFO"),
            (AuditSeverity::Warn, "WARN"),
            (AuditSeverity::Error, "ERROR"),
            (AuditSeverity::Critical, "CRITICAL"),
        ];

        for (severity, expected) in severities {
            let json = serde_json::to_string(&severity).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }

    #[test]
    #[serial]
    fn test_init_audit_logging() {
        let result = init_audit_logging();
        assert!(result.is_ok(), "Should initialize audit logging successfully");

        // Verify logs storage is initialized
        let logs = get_all_audit_logs();
        assert!(logs.is_ok());
    }

    #[test]
    #[serial]
    fn test_log_network_request() {
        // Initialize logging system
        init_audit_logging().unwrap();
        clear_audit_logs().unwrap();

        log_network_request("test-module", "https://example.com", "GET", 200);

        let logs = get_all_audit_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].module_id, "test-module");
        assert_eq!(logs[0].event_type, "network_request");

        // Verify details contain URL, method, statusCode
        assert!(logs[0].details.get("url").is_some());
        assert!(logs[0].details.get("method").is_some());
        assert!(logs[0].details.get("statusCode").is_some());
    }

    #[test]
    #[serial]
    fn test_log_permission_violation() {
        init_audit_logging().unwrap();
        clear_audit_logs().unwrap();

        log_permission_violation("test-module", "network access", "domain not whitelisted");

        let logs = get_all_audit_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].event_type, "permission_violation");
        assert!(matches!(logs[0].severity, AuditSeverity::Error));
    }

    #[test]
    #[serial]
    fn test_log_security_event() {
        init_audit_logging().unwrap();
        clear_audit_logs().unwrap();

        log_security_event(
            "test-module",
            "sandbox_violation",
            AuditSeverity::Critical,
            serde_json::json!({"violation": "eval detected"}),
        );

        let logs = get_all_audit_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].event_type, "sandbox_violation");
        assert!(matches!(logs[0].severity, AuditSeverity::Critical));
    }

    #[test]
    #[serial]
    fn test_get_module_audit_logs_filter_by_module() {
        init_audit_logging().unwrap();
        clear_audit_logs().unwrap();

        // Add logs for different modules
        log_network_request("module-a", "https://a.com", "GET", 200);
        log_network_request("module-b", "https://b.com", "GET", 200);
        log_network_request("module-a", "https://a2.com", "POST", 201);

        // Get logs for module-a only
        let logs = get_module_audit_logs("module-a", None, None, None).unwrap();
        assert_eq!(logs.len(), 2);
        assert!(logs.iter().all(|log| log.module_id == "module-a"));
    }

    #[test]
    #[serial]
    fn test_get_module_audit_logs_filter_by_severity() {
        init_audit_logging().unwrap();
        clear_audit_logs().unwrap();

        log_network_request("test-module", "https://example.com", "GET", 200);
        log_permission_violation("test-module", "action", "reason");
        log_security_event("test-module", "event", AuditSeverity::Critical, serde_json::json!({}));

        // Get only ERROR severity logs
        let logs = get_module_audit_logs("test-module", None, None, Some(AuditSeverity::Error)).unwrap();
        assert_eq!(logs.len(), 1);
        assert!(matches!(logs[0].severity, AuditSeverity::Error));
    }

    #[test]
    #[serial]
    fn test_get_module_audit_logs_pagination() {
        init_audit_logging().unwrap();
        clear_audit_logs().unwrap();

        // Add 5 logs
        for i in 0..5 {
            log_network_request("test-module", &format!("https://example{}.com", i), "GET", 200);
        }

        // Test limit
        let logs = get_module_audit_logs("test-module", Some(2), None, None).unwrap();
        assert_eq!(logs.len(), 2);

        // Test offset
        let logs = get_module_audit_logs("test-module", None, Some(2), None).unwrap();
        assert_eq!(logs.len(), 3);

        // Test limit + offset
        let logs = get_module_audit_logs("test-module", Some(2), Some(1), None).unwrap();
        assert_eq!(logs.len(), 2);
    }
}
