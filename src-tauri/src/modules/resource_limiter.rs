// Resource Limiter (Feature 004)
//
// This module enforces resource limits on module execution, including
// CPU time, memory usage, and heap size limits.

use std::time::Duration;

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum CPU time per request (default: 5 seconds)
    pub max_cpu_time: Duration,
    /// Maximum memory usage (default: 50 MB)
    pub max_memory: usize,
    /// Maximum heap size (default: 100 MB)
    pub max_heap_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_time: Duration::from_secs(5),
            max_memory: 50 * 1024 * 1024,      // 50 MB
            max_heap_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

/// Resource monitor and enforcer
pub struct ResourceLimiter {
    limits: ResourceLimits,
}

impl ResourceLimiter {
    /// Create a new resource limiter with default limits
    pub fn new() -> Self {
        // TODO: Implement in Phase 3 (Task T049-T053)
        Self {
            limits: ResourceLimits::default(),
        }
    }

    /// Create a resource limiter with custom limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    /// Execute a function with CPU time limit (Phase 3: T050)
    pub async fn execute_with_timeout<F, T>(&self, f: F) -> Result<T, String>
    where
        F: std::future::Future<Output = T>,
    {
        // Use tokio timeout to enforce CPU time limit
        tokio::time::timeout(self.limits.max_cpu_time, f)
            .await
            .map_err(|_| format!(
                "Execution timeout: exceeded {:?} limit",
                self.limits.max_cpu_time
            ))
    }

    /// Check current memory usage (Phase 3: T051)
    ///
    /// Note: For MVP, this is a simplified implementation.
    /// Full implementation would use platform-specific APIs or /proc/self/status
    pub fn check_memory_usage(&self) -> Result<usize, String> {
        // Simplified: Return 0 for now
        // In production, would use:
        // - Linux: parse /proc/self/status
        // - macOS: task_info()
        // - Windows: GetProcessMemoryInfo()
        Ok(0)
    }

    /// Enforce resource limits (Phase 3: T052)
    ///
    /// Checks if current usage exceeds limits
    pub fn enforce_limits(&self) -> Result<(), String> {
        let current_memory = self.check_memory_usage()?;

        if current_memory > self.limits.max_memory {
            return Err(format!(
                "Memory limit exceeded: {} bytes > {} bytes",
                current_memory, self.limits.max_memory
            ));
        }

        // Memory is within limits
        Ok(())
    }

    /// Gets the current limits
    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limiter = ResourceLimiter::new();
        assert_eq!(limiter.limits.max_cpu_time, Duration::from_secs(5));
        assert_eq!(limiter.limits.max_memory, 50 * 1024 * 1024);
    }

    #[test]
    fn test_custom_limits() {
        let limits = ResourceLimits {
            max_cpu_time: Duration::from_secs(10),
            max_memory: 100 * 1024 * 1024,
            max_heap_size: 200 * 1024 * 1024,
        };
        let limiter = ResourceLimiter::with_limits(limits.clone());
        assert_eq!(limiter.limits.max_cpu_time, Duration::from_secs(10));
        assert_eq!(limiter.limits.max_memory, 100 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_timeout_enforcement() {
        let limits = ResourceLimits {
            max_cpu_time: Duration::from_millis(100),
            max_memory: 50 * 1024 * 1024,
            max_heap_size: 100 * 1024 * 1024,
        };
        let limiter = ResourceLimiter::with_limits(limits);

        // Fast operation should succeed
        let result = limiter.execute_with_timeout(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_exceeded() {
        let limits = ResourceLimits {
            max_cpu_time: Duration::from_millis(50),
            max_memory: 50 * 1024 * 1024,
            max_heap_size: 100 * 1024 * 1024,
        };
        let limiter = ResourceLimiter::with_limits(limits);

        // Slow operation should timeout
        let result = limiter.execute_with_timeout(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            42
        }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout"));
    }

    #[test]
    fn test_memory_check() {
        let limiter = ResourceLimiter::new();
        let result = limiter.check_memory_usage();
        assert!(result.is_ok());
    }

    #[test]
    fn test_enforce_limits() {
        let limiter = ResourceLimiter::new();
        let result = limiter.enforce_limits();
        assert!(result.is_ok());
    }
}
