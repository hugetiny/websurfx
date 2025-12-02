//! This module provides an engine health checker that periodically tests engines.
//! Similar to SearXNG's checker functionality.

use crate::models::engine::EngineHandler;
use crate::models::engine_health::{get_health_manager, EngineErrorType};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Result of a single engine check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Whether the check was successful
    pub success: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Number of results returned
    pub result_count: usize,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp of the check (unix seconds)
    pub timestamp: u64,
}

impl CheckResult {
    /// Create a successful check result
    pub fn success(response_time_ms: u64, result_count: usize) -> Self {
        Self {
            success: true,
            response_time_ms,
            result_count,
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a failed check result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            response_time_ms: 0,
            result_count: 0,
            error: Some(error),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Engine checker status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCheckStatus {
    /// Engine name
    pub name: String,
    /// Last check result
    pub last_result: Option<CheckResult>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Number of consecutive successes
    pub consecutive_successes: u32,
    /// Total checks performed
    pub total_checks: u64,
    /// Total successful checks
    pub successful_checks: u64,
    /// Whether the engine is considered healthy
    pub is_healthy: bool,
}

impl EngineCheckStatus {
    /// Create a new check status for an engine
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            last_result: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
            total_checks: 0,
            successful_checks: 0,
            is_healthy: true,
        }
    }

    /// Record a successful check
    pub fn record_success(&mut self, result: CheckResult) {
        self.last_result = Some(result);
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;
        self.total_checks += 1;
        self.successful_checks += 1;

        // Mark as healthy after 2 consecutive successes
        if self.consecutive_successes >= 2 {
            self.is_healthy = true;
        }
    }

    /// Record a failed check
    pub fn record_failure(&mut self, result: CheckResult) {
        self.last_result = Some(result);
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
        self.total_checks += 1;

        // Mark as unhealthy after 3 consecutive failures
        if self.consecutive_failures >= 3 {
            self.is_healthy = false;
        }
    }

    /// Calculate reliability percentage
    pub fn reliability(&self) -> f64 {
        if self.total_checks == 0 {
            return 100.0;
        }
        (self.successful_checks as f64 / self.total_checks as f64) * 100.0
    }
}

/// Engine health checker
pub struct EngineChecker {
    /// HTTP client for making requests
    client: Client,
    /// Status for each engine
    statuses: RwLock<HashMap<String, EngineCheckStatus>>,
    /// Test query to use for checking
    test_query: String,
    /// Request timeout in seconds
    #[allow(dead_code)]
    timeout_secs: u64,
}

impl EngineChecker {
    /// Create a new engine checker
    pub fn new(timeout_secs: u64) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(timeout_secs))
            .use_rustls_tls()
            .https_only(false)
            .gzip(true)
            .brotli(true)
            .build()
            .unwrap();

        Self {
            client,
            statuses: RwLock::new(HashMap::new()),
            test_query: "test".to_string(),
            timeout_secs,
        }
    }

    /// Set the test query
    pub fn with_test_query(mut self, query: &str) -> Self {
        self.test_query = query.to_string();
        self
    }

    /// Check a single engine
    pub async fn check_engine(&self, engine_name: &str) -> CheckResult {
        let handler = match EngineHandler::new(engine_name) {
            Ok(h) => h,
            Err(e) => {
                return CheckResult::failure(format!("Failed to create engine handler: {:?}", e));
            }
        };

        let (name, engine) = handler.into_name_engine();
        let start = Instant::now();

        // Use a common user agent for testing
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        let result = engine
            .results(&self.test_query, 1, user_agent, &self.client, 0)
            .await;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(results) => {
                let count = results.len();
                let check_result = CheckResult::success(elapsed_ms, count);

                // Update status
                let mut statuses = self.statuses.write().await;
                let status = statuses
                    .entry(name.to_string())
                    .or_insert_with(|| EngineCheckStatus::new(name));
                status.record_success(check_result.clone());

                // Also record in health manager
                get_health_manager().record_success(name, elapsed_ms, count);

                check_result
            }
            Err(e) => {
                let error_msg = format!("{:?}", e);
                let check_result = CheckResult::failure(error_msg.clone());

                // Update status
                let mut statuses = self.statuses.write().await;
                let status = statuses
                    .entry(name.to_string())
                    .or_insert_with(|| EngineCheckStatus::new(name));
                status.record_failure(check_result.clone());

                // Determine error type and record in health manager
                let error_type = if error_msg.contains("timeout") {
                    EngineErrorType::Timeout
                } else if error_msg.contains("403") || error_msg.contains("denied") {
                    EngineErrorType::AccessDenied
                } else if error_msg.contains("429") {
                    EngineErrorType::TooManyRequests
                } else {
                    EngineErrorType::NetworkError
                };

                get_health_manager().record_error(name, error_type, Some(error_msg));

                check_result
            }
        }
    }

    /// Check multiple engines concurrently
    pub async fn check_engines(&self, engine_names: &[&str]) -> HashMap<String, CheckResult> {
        let mut results = HashMap::new();
        let mut handles = Vec::new();

        for &name in engine_names {
            let name_owned = name.to_string();
            let self_ref = self;
            handles.push(async move {
                let result = self_ref.check_engine(&name_owned).await;
                (name_owned, result)
            });
        }

        // Run all checks concurrently with a limit
        let checks: Vec<_> = futures::future::join_all(handles).await;

        for (name, result) in checks {
            results.insert(name, result);
        }

        results
    }

    /// Get the status of a specific engine
    pub async fn get_status(&self, engine_name: &str) -> Option<EngineCheckStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(engine_name).cloned()
    }

    /// Get all engine statuses
    pub async fn get_all_statuses(&self) -> HashMap<String, EngineCheckStatus> {
        self.statuses.read().await.clone()
    }

    /// Get list of healthy engines
    pub async fn get_healthy_engines(&self) -> Vec<String> {
        let statuses = self.statuses.read().await;
        statuses
            .iter()
            .filter(|(_, status)| status.is_healthy)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get list of unhealthy engines
    pub async fn get_unhealthy_engines(&self) -> Vec<String> {
        let statuses = self.statuses.read().await;
        statuses
            .iter()
            .filter(|(_, status)| !status.is_healthy)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Reset status for an engine
    pub async fn reset_status(&self, engine_name: &str) {
        let mut statuses = self.statuses.write().await;
        statuses.remove(engine_name);
    }

    /// Reset all statuses
    pub async fn reset_all(&self) {
        let mut statuses = self.statuses.write().await;
        statuses.clear();
    }
}

/// Global checker instance
static CHECKER: std::sync::OnceLock<Arc<EngineChecker>> = std::sync::OnceLock::new();

/// Get the global checker instance
pub fn get_checker() -> Arc<EngineChecker> {
    CHECKER
        .get_or_init(|| Arc::new(EngineChecker::new(10)))
        .clone()
}

/// Initialize the checker with custom timeout
pub fn init_checker(timeout_secs: u64) -> Arc<EngineChecker> {
    let checker = Arc::new(EngineChecker::new(timeout_secs));
    let _ = CHECKER.set(checker.clone());
    checker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result() {
        let success = CheckResult::success(100, 10);
        assert!(success.success);
        assert_eq!(success.response_time_ms, 100);
        assert_eq!(success.result_count, 10);
        assert!(success.error.is_none());

        let failure = CheckResult::failure("test error".to_string());
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[test]
    fn test_engine_check_status() {
        let mut status = EngineCheckStatus::new("test");
        assert!(status.is_healthy);

        // Record failures
        for _ in 0..3 {
            status.record_failure(CheckResult::failure("error".to_string()));
        }
        assert!(!status.is_healthy);
        assert_eq!(status.consecutive_failures, 3);

        // Record successes to recover
        for _ in 0..2 {
            status.record_success(CheckResult::success(100, 10));
        }
        assert!(status.is_healthy);
        assert_eq!(status.consecutive_successes, 2);
    }
}
