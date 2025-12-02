//! This module provides engine health management functionality similar to SearXNG.
//! It includes automatic suspend/resume mechanisms, reliability tracking, and statistics.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Configuration for engine suspension times (in seconds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendedTimesConfig {
    /// Default suspension time for access denied errors (default: 86400 = 1 day)
    pub access_denied: u64,
    /// Suspension time for CAPTCHA errors (default: 86400 = 1 day)
    pub captcha: u64,
    /// Suspension time for too many requests / rate limit errors (default: 3600 = 1 hour)
    pub too_many_requests: u64,
    /// Suspension time for timeout errors (default: 60 = 1 minute)
    pub timeout: u64,
    /// Suspension time for SSL/TLS errors (default: 3600 = 1 hour)
    pub ssl_error: u64,
    /// Suspension time for HTTP errors (default: 300 = 5 minutes)
    pub http_error: u64,
    /// Suspension time for parse errors (default: 600 = 10 minutes)
    pub parse_error: u64,
    /// Default ban time on fail (default: 5 seconds)
    pub ban_time_on_fail: u64,
    /// Maximum ban time on fail (default: 120 seconds)
    pub max_ban_time_on_fail: u64,
}

impl Default for SuspendedTimesConfig {
    fn default() -> Self {
        Self {
            access_denied: 86400,      // 1 day
            captcha: 86400,            // 1 day
            too_many_requests: 3600,   // 1 hour
            timeout: 60,               // 1 minute
            ssl_error: 3600,           // 1 hour
            http_error: 300,           // 5 minutes
            parse_error: 600,          // 10 minutes
            ban_time_on_fail: 5,       // 5 seconds
            max_ban_time_on_fail: 120, // 2 minutes
        }
    }
}

/// Error types that can occur during engine requests
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineErrorType {
    /// Request timed out
    Timeout,
    /// Access denied (HTTP 403, etc.)
    AccessDenied,
    /// CAPTCHA required
    Captcha,
    /// Too many requests (HTTP 429)
    TooManyRequests,
    /// SSL/TLS error
    SslError,
    /// HTTP error (4xx, 5xx)
    HttpError(u16),
    /// Parse error (HTML/JSON parsing failed)
    ParseError,
    /// Empty result set
    EmptyResultSet,
    /// Network error
    NetworkError,
    /// Unknown/unexpected error
    UnexpectedError,
    /// Engine not found
    EngineNotFound,
}

impl EngineErrorType {
    /// Get the default suspension time for this error type
    pub fn get_suspend_time(&self, config: &SuspendedTimesConfig) -> u64 {
        match self {
            EngineErrorType::Timeout => config.timeout,
            EngineErrorType::AccessDenied => config.access_denied,
            EngineErrorType::Captcha => config.captcha,
            EngineErrorType::TooManyRequests => config.too_many_requests,
            EngineErrorType::SslError => config.ssl_error,
            EngineErrorType::HttpError(code) => {
                match code {
                    429 => config.too_many_requests,
                    403 | 401 => config.access_denied,
                    _ => config.http_error,
                }
            }
            EngineErrorType::ParseError => config.parse_error,
            EngineErrorType::EmptyResultSet => 0, // Don't suspend for empty results
            EngineErrorType::NetworkError => config.ban_time_on_fail,
            EngineErrorType::UnexpectedError => config.ban_time_on_fail,
            EngineErrorType::EngineNotFound => 0, // Don't suspend for not found
        }
    }

    /// Whether this error should trigger suspension
    pub fn should_suspend(&self) -> bool {
        match self {
            EngineErrorType::EmptyResultSet => false,
            EngineErrorType::EngineNotFound => false,
            _ => true,
        }
    }

    /// Get the severity color for UI display
    pub fn severity_color(&self) -> &'static str {
        match self {
            EngineErrorType::Timeout => "yellow",
            EngineErrorType::AccessDenied => "red",
            EngineErrorType::Captcha => "red",
            EngineErrorType::TooManyRequests => "orange",
            EngineErrorType::SslError => "red",
            EngineErrorType::HttpError(_) => "orange",
            EngineErrorType::ParseError => "yellow",
            EngineErrorType::EmptyResultSet => "blue",
            EngineErrorType::NetworkError => "yellow",
            EngineErrorType::UnexpectedError => "red",
            EngineErrorType::EngineNotFound => "red",
        }
    }

    /// Convert to display string
    pub fn to_string(&self) -> String {
        match self {
            EngineErrorType::Timeout => "Timeout".to_string(),
            EngineErrorType::AccessDenied => "Access Denied".to_string(),
            EngineErrorType::Captcha => "CAPTCHA Required".to_string(),
            EngineErrorType::TooManyRequests => "Too Many Requests".to_string(),
            EngineErrorType::SslError => "SSL/TLS Error".to_string(),
            EngineErrorType::HttpError(code) => format!("HTTP Error {}", code),
            EngineErrorType::ParseError => "Parse Error".to_string(),
            EngineErrorType::EmptyResultSet => "Empty Result Set".to_string(),
            EngineErrorType::NetworkError => "Network Error".to_string(),
            EngineErrorType::UnexpectedError => "Unexpected Error".to_string(),
            EngineErrorType::EngineNotFound => "Engine Not Found".to_string(),
        }
    }
}

/// Represents a single error occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    /// The type of error
    pub error_type: EngineErrorType,
    /// When the error occurred
    #[serde(skip)]
    pub timestamp: Option<Instant>,
    /// Unix timestamp for serialization
    pub unix_timestamp: u64,
    /// Optional error message
    pub message: Option<String>,
}

impl ErrorRecord {
    pub fn new(error_type: EngineErrorType, message: Option<String>) -> Self {
        Self {
            error_type,
            timestamp: Some(Instant::now()),
            unix_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            message,
        }
    }
}

/// Engine suspension status
#[derive(Debug, Clone)]
pub struct SuspendedStatus {
    /// Whether the engine is currently suspended
    pub is_suspended: bool,
    /// When the suspension ends
    pub suspend_end_time: Option<Instant>,
    /// The reason for suspension
    pub suspend_reason: Option<EngineErrorType>,
    /// Number of consecutive errors
    pub continuous_errors: u32,
}

impl Default for SuspendedStatus {
    fn default() -> Self {
        Self {
            is_suspended: false,
            suspend_end_time: None,
            suspend_reason: None,
            continuous_errors: 0,
        }
    }
}

impl SuspendedStatus {
    /// Check if the engine is currently suspended
    pub fn check_suspended(&self) -> bool {
        if !self.is_suspended {
            return false;
        }
        if let Some(end_time) = self.suspend_end_time {
            Instant::now() < end_time
        } else {
            false
        }
    }

    /// Get remaining suspension time in seconds
    pub fn remaining_suspension_secs(&self) -> u64 {
        if !self.is_suspended {
            return 0;
        }
        if let Some(end_time) = self.suspend_end_time {
            let now = Instant::now();
            if now < end_time {
                (end_time - now).as_secs()
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Suspend the engine for a specified duration
    pub fn suspend(&mut self, duration_secs: u64, reason: EngineErrorType) {
        self.is_suspended = true;
        self.suspend_end_time = Some(Instant::now() + Duration::from_secs(duration_secs));
        self.suspend_reason = Some(reason);
        self.continuous_errors += 1;
    }

    /// Resume the engine (reset suspension)
    pub fn resume(&mut self) {
        self.is_suspended = false;
        self.suspend_end_time = None;
        self.suspend_reason = None;
        self.continuous_errors = 0;
    }
}

/// Metrics for a single engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    /// Engine name
    pub name: String,
    /// Total number of requests sent
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Total number of results returned
    pub total_results: u64,
    /// Total response time in milliseconds
    pub total_response_time_ms: u64,
    /// Error counts by type
    pub error_counts: HashMap<String, u64>,
    /// Recent errors (last N errors)
    pub recent_errors: Vec<ErrorRecord>,
    /// Whether currently suspended
    pub is_suspended: bool,
    /// Remaining suspension time in seconds
    pub suspension_remaining_secs: u64,
    /// Suspension reason if suspended
    pub suspension_reason: Option<String>,
}

impl EngineMetrics {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            total_requests: 0,
            successful_requests: 0,
            total_results: 0,
            total_response_time_ms: 0,
            error_counts: HashMap::new(),
            recent_errors: Vec::new(),
            is_suspended: false,
            suspension_remaining_secs: 0,
            suspension_reason: None,
        }
    }

    /// Calculate reliability percentage (0-100)
    pub fn reliability(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate average response time in milliseconds
    pub fn avg_response_time_ms(&self) -> f64 {
        if self.successful_requests == 0 {
            return 0.0;
        }
        self.total_response_time_ms as f64 / self.successful_requests as f64
    }

    /// Calculate average results per request
    pub fn avg_results_per_request(&self) -> f64 {
        if self.successful_requests == 0 {
            return 0.0;
        }
        self.total_results as f64 / self.successful_requests as f64
    }
}

/// Statistics summary for all engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Individual engine metrics
    pub engines: HashMap<String, EngineMetrics>,
    /// Timestamp when stats were generated
    pub generated_at: u64,
}

/// Internal state for a single engine
struct EngineState {
    suspended_status: SuspendedStatus,
    metrics: EngineMetrics,
}

impl EngineState {
    fn new(name: &str) -> Self {
        Self {
            suspended_status: SuspendedStatus::default(),
            metrics: EngineMetrics::new(name),
        }
    }
}

/// Global engine health manager
pub struct EngineHealthManager {
    /// Engine states (name -> state)
    states: RwLock<HashMap<String, EngineState>>,
    /// Suspension times configuration
    config: SuspendedTimesConfig,
    /// Maximum number of recent errors to keep per engine
    max_recent_errors: usize,
}

impl EngineHealthManager {
    /// Create a new health manager with default configuration
    pub fn new() -> Self {
        Self::with_config(SuspendedTimesConfig::default())
    }

    /// Create a new health manager with custom configuration
    pub fn with_config(config: SuspendedTimesConfig) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            config,
            max_recent_errors: 50,
        }
    }

    /// Ensure an engine state exists
    fn ensure_engine(&self, engine_name: &str) {
        let mut states = self.states.write().unwrap();
        if !states.contains_key(engine_name) {
            states.insert(engine_name.to_string(), EngineState::new(engine_name));
        }
    }

    /// Check if an engine is currently suspended
    pub fn is_suspended(&self, engine_name: &str) -> bool {
        self.ensure_engine(engine_name);
        let states = self.states.read().unwrap();
        if let Some(state) = states.get(engine_name) {
            state.suspended_status.check_suspended()
        } else {
            false
        }
    }

    /// Get the remaining suspension time for an engine
    pub fn get_suspension_info(&self, engine_name: &str) -> Option<(u64, String)> {
        self.ensure_engine(engine_name);
        let states = self.states.read().unwrap();
        if let Some(state) = states.get(engine_name) {
            if state.suspended_status.check_suspended() {
                let remaining = state.suspended_status.remaining_suspension_secs();
                let reason = state.suspended_status.suspend_reason
                    .as_ref()
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                Some((remaining, reason))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Record a successful request
    pub fn record_success(&self, engine_name: &str, response_time_ms: u64, result_count: usize) {
        self.ensure_engine(engine_name);
        let mut states = self.states.write().unwrap();
        if let Some(state) = states.get_mut(engine_name) {
            state.metrics.total_requests += 1;
            state.metrics.successful_requests += 1;
            state.metrics.total_results += result_count as u64;
            state.metrics.total_response_time_ms += response_time_ms;
            // Resume the engine on success (reset error count)
            state.suspended_status.resume();
        }
    }

    /// Record a failed request and potentially suspend the engine
    pub fn record_error(&self, engine_name: &str, error_type: EngineErrorType, message: Option<String>) -> bool {
        self.ensure_engine(engine_name);
        let mut states = self.states.write().unwrap();
        if let Some(state) = states.get_mut(engine_name) {
            state.metrics.total_requests += 1;

            // Update error counts
            let error_key = error_type.to_string();
            *state.metrics.error_counts.entry(error_key).or_insert(0) += 1;

            // Add to recent errors
            let error_record = ErrorRecord::new(error_type.clone(), message);
            state.metrics.recent_errors.push(error_record);

            // Trim recent errors if needed
            if state.metrics.recent_errors.len() > self.max_recent_errors {
                state.metrics.recent_errors.remove(0);
            }

            // Check if we should suspend
            if error_type.should_suspend() {
                let suspend_time = error_type.get_suspend_time(&self.config);

                // Calculate actual suspend time based on continuous errors
                let actual_suspend_time = if state.suspended_status.continuous_errors > 0 {
                    // Exponential backoff: min(base_time * 2^errors, max_ban_time)
                    let multiplier = 2_u64.pow(state.suspended_status.continuous_errors.min(10));
                    suspend_time.saturating_mul(multiplier).min(self.config.max_ban_time_on_fail)
                } else {
                    suspend_time.max(self.config.ban_time_on_fail)
                };

                state.suspended_status.suspend(actual_suspend_time, error_type);
                log::warn!(
                    "Engine '{}' suspended for {} seconds due to: {}",
                    engine_name,
                    actual_suspend_time,
                    state.suspended_status.suspend_reason.as_ref().unwrap().to_string()
                );
                return true;
            }
        }
        false
    }

    /// Manually resume a suspended engine
    pub fn resume_engine(&self, engine_name: &str) {
        self.ensure_engine(engine_name);
        let mut states = self.states.write().unwrap();
        if let Some(state) = states.get_mut(engine_name) {
            state.suspended_status.resume();
            log::info!("Engine '{}' manually resumed", engine_name);
        }
    }

    /// Get metrics for a specific engine
    pub fn get_engine_metrics(&self, engine_name: &str) -> Option<EngineMetrics> {
        self.ensure_engine(engine_name);
        let states = self.states.read().unwrap();
        states.get(engine_name).map(|state| {
            let mut metrics = state.metrics.clone();
            metrics.is_suspended = state.suspended_status.check_suspended();
            metrics.suspension_remaining_secs = state.suspended_status.remaining_suspension_secs();
            metrics.suspension_reason = state.suspended_status.suspend_reason
                .as_ref()
                .map(|r| r.to_string());
            metrics
        })
    }

    /// Get stats for all engines
    pub fn get_all_stats(&self) -> EngineStats {
        let states = self.states.read().unwrap();
        let mut engines = HashMap::new();

        for (name, state) in states.iter() {
            let mut metrics = state.metrics.clone();
            metrics.is_suspended = state.suspended_status.check_suspended();
            metrics.suspension_remaining_secs = state.suspended_status.remaining_suspension_secs();
            metrics.suspension_reason = state.suspended_status.suspend_reason
                .as_ref()
                .map(|r| r.to_string());
            engines.insert(name.clone(), metrics);
        }

        EngineStats {
            engines,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Reset all statistics (but keep suspension status)
    pub fn reset_stats(&self) {
        let mut states = self.states.write().unwrap();
        for state in states.values_mut() {
            state.metrics.total_requests = 0;
            state.metrics.successful_requests = 0;
            state.metrics.total_results = 0;
            state.metrics.total_response_time_ms = 0;
            state.metrics.error_counts.clear();
            state.metrics.recent_errors.clear();
        }
    }

    /// Get list of currently suspended engines
    pub fn get_suspended_engines(&self) -> Vec<(String, u64, String)> {
        let states = self.states.read().unwrap();
        let mut suspended = Vec::new();

        for (name, state) in states.iter() {
            if state.suspended_status.check_suspended() {
                let remaining = state.suspended_status.remaining_suspension_secs();
                let reason = state.suspended_status.suspend_reason
                    .as_ref()
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                suspended.push((name.clone(), remaining, reason));
            }
        }

        suspended
    }

    /// Get list of active (non-suspended) engines from a list
    pub fn filter_active_engines<'a>(&self, engines: &[&'a str]) -> Vec<&'a str> {
        engines.iter()
            .filter(|name| !self.is_suspended(name))
            .copied()
            .collect()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SuspendedTimesConfig) {
        self.config = config;
    }
}

impl Default for EngineHealthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton for the health manager
static HEALTH_MANAGER: std::sync::OnceLock<Arc<EngineHealthManager>> = std::sync::OnceLock::new();

/// Get the global health manager instance
pub fn get_health_manager() -> Arc<EngineHealthManager> {
    HEALTH_MANAGER
        .get_or_init(|| Arc::new(EngineHealthManager::new()))
        .clone()
}

/// Initialize the health manager with custom configuration
pub fn init_health_manager(config: SuspendedTimesConfig) -> Arc<EngineHealthManager> {
    let manager = Arc::new(EngineHealthManager::with_config(config));
    let _ = HEALTH_MANAGER.set(manager.clone());
    manager
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspended_status() {
        let mut status = SuspendedStatus::default();
        assert!(!status.check_suspended());

        status.suspend(5, EngineErrorType::Timeout);
        assert!(status.check_suspended());
        assert!(status.remaining_suspension_secs() <= 5);

        status.resume();
        assert!(!status.check_suspended());
    }

    #[test]
    fn test_engine_health_manager() {
        let manager = EngineHealthManager::new();

        // Test recording success
        manager.record_success("test_engine", 100, 10);
        let metrics = manager.get_engine_metrics("test_engine").unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.reliability(), 100.0);

        // Test recording error with suspension
        let suspended = manager.record_error("test_engine", EngineErrorType::TooManyRequests, None);
        assert!(suspended);
        assert!(manager.is_suspended("test_engine"));

        // Test manual resume
        manager.resume_engine("test_engine");
        assert!(!manager.is_suspended("test_engine"));
    }

    #[test]
    fn test_error_types() {
        let config = SuspendedTimesConfig::default();

        assert_eq!(EngineErrorType::Timeout.get_suspend_time(&config), 60);
        assert_eq!(EngineErrorType::TooManyRequests.get_suspend_time(&config), 3600);
        assert_eq!(EngineErrorType::EmptyResultSet.get_suspend_time(&config), 0);

        assert!(EngineErrorType::Timeout.should_suspend());
        assert!(!EngineErrorType::EmptyResultSet.should_suspend());
    }
}
