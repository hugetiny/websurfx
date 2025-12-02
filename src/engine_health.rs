//! This module provides engine health monitoring and auto-disable functionality.
//! It tracks engine failures and automatically disables engines that consistently fail.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::SystemTime;

/// Tracks the health status of search engines
#[derive(Clone)]
pub struct EngineHealthTracker {
    /// Stores failure counts and last failure time for each engine
    health_data: Arc<RwLock<HashMap<String, EngineHealth>>>,
    /// Configuration for health tracking
    config: HealthConfig,
}

/// Health data for a single engine
#[derive(Clone, Debug)]
struct EngineHealth {
    /// Number of consecutive failures
    consecutive_failures: u32,
    /// Total number of failures
    total_failures: u32,
    /// Total number of requests
    total_requests: u32,
    /// Last failure timestamp
    last_failure: Option<SystemTime>,
    /// Last success timestamp
    last_success: Option<SystemTime>,
    /// Whether the engine is currently disabled
    is_disabled: bool,
    /// When the engine was disabled
    disabled_at: Option<SystemTime>,
}

/// Configuration for health tracking
#[derive(Clone, Debug)]
pub struct HealthConfig {
    /// Number of consecutive failures before disabling
    pub max_consecutive_failures: u32,
    /// Failure rate threshold (0.0 - 1.0) before disabling
    pub max_failure_rate: f32,
    /// Minimum requests before checking failure rate
    pub min_requests_for_rate_check: u32,
    /// Time to wait before re-enabling a disabled engine (seconds)
    pub re_enable_after_seconds: u64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            max_consecutive_failures: 5,
            max_failure_rate: 0.8,
            min_requests_for_rate_check: 10,
            re_enable_after_seconds: 300, // 5 minutes
        }
    }
}

impl Default for EngineHealth {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            total_failures: 0,
            total_requests: 0,
            last_failure: None,
            last_success: None,
            is_disabled: false,
            disabled_at: None,
        }
    }
}

impl EngineHealthTracker {
    /// Creates a new engine health tracker
    pub fn new(config: HealthConfig) -> Self {
        Self {
            health_data: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Creates a new tracker with default configuration
    pub fn with_defaults() -> Self {
        Self::new(HealthConfig::default())
    }

    /// Records a successful request for an engine
    pub async fn record_success(&self, engine_name: &str) {
        let mut data = self.health_data.write().await;
        let health = data.entry(engine_name.to_string()).or_default();

        health.consecutive_failures = 0;
        health.total_requests += 1;
        health.last_success = Some(SystemTime::now());

        log::debug!("Engine {} succeeded. Total requests: {}, Total failures: {}",
                   engine_name, health.total_requests, health.total_failures);
    }

    /// Records a failed request for an engine
    pub async fn record_failure(&self, engine_name: &str) {
        let mut data = self.health_data.write().await;
        let health = data.entry(engine_name.to_string()).or_default();

        health.consecutive_failures += 1;
        health.total_failures += 1;
        health.total_requests += 1;
        health.last_failure = Some(SystemTime::now());

        // Check if engine should be disabled
        let should_disable = self.should_disable_engine(health);

        if should_disable && !health.is_disabled {
            health.is_disabled = true;
            health.disabled_at = Some(SystemTime::now());
            log::warn!(
                "Engine {} auto-disabled due to high failure rate. Consecutive failures: {}, Total failures: {}/{}",
                engine_name,
                health.consecutive_failures,
                health.total_failures,
                health.total_requests
            );
        } else {
            log::debug!(
                "Engine {} failed. Consecutive failures: {}, Total failures: {}/{}",
                engine_name,
                health.consecutive_failures,
                health.total_failures,
                health.total_requests
            );
        }
    }

    /// Checks if an engine should be disabled based on its health
    fn should_disable_engine(&self, health: &EngineHealth) -> bool {
        // Check consecutive failures
        if health.consecutive_failures >= self.config.max_consecutive_failures {
            return true;
        }

        // Check failure rate (only if we have enough requests)
        if health.total_requests >= self.config.min_requests_for_rate_check {
            let failure_rate = health.total_failures as f32 / health.total_requests as f32;
            if failure_rate >= self.config.max_failure_rate {
                return true;
            }
        }

        false
    }

    /// Checks if an engine is currently disabled
    pub async fn is_engine_disabled(&self, engine_name: &str) -> bool {
        let mut data = self.health_data.write().await;

        if let Some(health) = data.get_mut(engine_name) {
            // Check if enough time has passed to re-enable
            if health.is_disabled {
                if let Some(disabled_at) = health.disabled_at {
                    if let Ok(elapsed) = SystemTime::now().duration_since(disabled_at) {
                        if elapsed.as_secs() >= self.config.re_enable_after_seconds {
                            // Re-enable the engine
                            health.is_disabled = false;
                            health.consecutive_failures = 0;
                            log::info!("Engine {} re-enabled after {} seconds",
                                     engine_name, elapsed.as_secs());
                            return false;
                        }
                    }
                }
                return true;
            }
        }

        false
    }

    /// Gets health statistics for all engines
    pub async fn get_health_stats(&self) -> HashMap<String, EngineHealthStats> {
        let data = self.health_data.read().await;
        data.iter()
            .map(|(name, health)| {
                (
                    name.clone(),
                    EngineHealthStats {
                        total_requests: health.total_requests,
                        total_failures: health.total_failures,
                        consecutive_failures: health.consecutive_failures,
                        failure_rate: if health.total_requests > 0 {
                            health.total_failures as f32 / health.total_requests as f32
                        } else {
                            0.0
                        },
                        is_disabled: health.is_disabled,
                    },
                )
            })
            .collect()
    }

    /// Reset health data for a specific engine
    pub async fn reset_engine(&self, engine_name: &str) {
        let mut data = self.health_data.write().await;
        if let Some(health) = data.get_mut(engine_name) {
            *health = EngineHealth::default();
            log::info!("Reset health data for engine: {}", engine_name);
        }
    }

    /// Reset health data for all engines
    pub async fn reset_all_engines(&self) {
        let mut data = self.health_data.write().await;
        for (name, health) in data.iter_mut() {
            *health = EngineHealth::default();
            log::info!("Reset health data for engine: {}", name);
        }
    }
}

/// Public health statistics for an engine
#[derive(Clone, Debug)]
pub struct EngineHealthStats {
    /// Total number of requests made to this engine
    pub total_requests: u32,
    /// Total number of failed requests
    pub total_failures: u32,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Failure rate (0.0 - 1.0)
    pub failure_rate: f32,
    /// Whether the engine is currently disabled
    pub is_disabled: bool,
}
