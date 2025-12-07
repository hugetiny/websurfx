//! Background checker scheduler for periodic engine health checks.
//!
//! This module provides SearXNG-compatible background checking functionality:
//! - Periodic engine health checks
//! - Automatic engine disable/enable based on check results
//! - Result caching and persistence
//! - Integration with Valkey/Redis for distributed environments (optional)

use crate::engine_checker::get_checker;
use crate::metrics::{get_metrics, ReliabilityInfo};
use crate::config_manager::get_config_manager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

/// Checker result status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum CheckerResult {
    /// Checker is currently running
    #[serde(rename = "running")]
    Running,
    /// Checker completed successfully
    #[serde(rename = "ok")]
    Ok {
        /// Per-engine results
        engines: HashMap<String, EngineCheckResult>,
        /// Timestamp of the check
        timestamp: u64,
    },
    /// Checker encountered an error
    #[serde(rename = "error")]
    Error {
        /// Error message
        message: String,
        /// Timestamp of the error
        timestamp: u64,
    },
    /// Checker is disabled
    #[serde(rename = "disabled")]
    Disabled,
    /// Unknown status
    #[serde(rename = "unknown")]
    Unknown,
}

/// Result of checking a single engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCheckResult {
    /// Whether the check was successful
    pub success: bool,
    /// Error details if failed (test_name -> error messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Vec<String>>>,
}

/// Checker scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Whether the scheduler is enabled
    pub enabled: bool,
    /// Check interval in seconds (default: 3600 = 1 hour)
    pub interval_secs: u64,
    /// Maximum check duration in seconds (default: 300 = 5 minutes)
    pub max_duration_secs: u64,
    /// List of engines to check (empty = all)
    pub engines: Vec<String>,
    /// Whether to auto-disable failing engines
    pub auto_disable: bool,
    /// Whether to auto-enable recovered engines
    pub auto_enable: bool,
    /// Minimum reliability to keep engine enabled
    pub min_reliability: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600, // 1 hour
            max_duration_secs: 300, // 5 minutes
            engines: vec![],
            auto_disable: true,
            auto_enable: true,
            min_reliability: 50.0,
        }
    }
}

/// Background checker scheduler
pub struct CheckerScheduler {
    /// Configuration
    config: RwLock<SchedulerConfig>,
    /// Last check result
    last_result: RwLock<CheckerResult>,
    /// Whether a check is currently running
    is_running: RwLock<bool>,
    /// Handle to the background task
    task_handle: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl CheckerScheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            config: RwLock::new(SchedulerConfig::default()),
            last_result: RwLock::new(CheckerResult::Unknown),
            is_running: RwLock::new(false),
            task_handle: RwLock::new(None),
        }
    }

    /// Create scheduler with config
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            config: RwLock::new(config),
            last_result: RwLock::new(CheckerResult::Unknown),
            is_running: RwLock::new(false),
            task_handle: RwLock::new(None),
        }
    }

    /// Update configuration
    pub async fn update_config(&self, config: SchedulerConfig) {
        let mut c = self.config.write().await;
        *c = config;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> SchedulerConfig {
        self.config.read().await.clone()
    }

    /// Get the last check result
    pub async fn get_result(&self) -> CheckerResult {
        self.last_result.read().await.clone()
    }

    /// Check if currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Run a single check cycle
    pub async fn run_check(&self) -> CheckerResult {
        // Check if already running
        {
            let mut running = self.is_running.write().await;
            if *running {
                return CheckerResult::Running;
            }
            *running = true;
        }

        // Set status to running
        {
            let mut result = self.last_result.write().await;
            *result = CheckerResult::Running;
        }

        let config = self.config.read().await.clone();
        let checker = get_checker();
        let config_manager = get_config_manager();
        let metrics = get_metrics();

        // Get list of engines to check (owned strings to avoid lifetime issues)
        let engine_names_owned: Vec<String> = if config.engines.is_empty() {
            // Get all registered engines from config manager
            let all_configs = config_manager.get_all_configs().await;
            all_configs.iter().map(|c| c.name.clone()).collect()
        } else {
            config.engines.clone()
        };

        let engine_names: Vec<&str> = engine_names_owned.iter().map(|s| s.as_str()).collect();

        // Run the checks with timeout
        let check_timeout = Duration::from_secs(config.max_duration_secs);
        let check_future = checker.check_engines(&engine_names);

        let results = match tokio::time::timeout(check_timeout, check_future).await {
            Ok(results) => results,
            Err(_) => {
                let result = CheckerResult::Error {
                    message: "Check timeout exceeded".to_string(),
                    timestamp: current_timestamp(),
                };

                // Store result
                {
                    let mut r = self.last_result.write().await;
                    *r = result.clone();
                }

                // Mark as not running
                {
                    let mut running = self.is_running.write().await;
                    *running = false;
                }

                return result;
            }
        };

        // Process results
        let mut engine_results = HashMap::new();

        for (name, check_result) in &results {
            let success = check_result.success;
            let errors = if success {
                None
            } else {
                let mut error_map = HashMap::new();
                if let Some(ref error_msg) = check_result.error {
                    error_map.insert("simple".to_string(), vec![error_msg.clone()]);
                }
                Some(error_map)
            };

            engine_results.insert(name.clone(), EngineCheckResult {
                success,
                errors: errors.clone(),
            });

            // Update metrics with checker results
            metrics.set_checker_result(
                name,
                success,
                errors.unwrap_or_default(),
            );

            // Auto-disable/enable based on results
            if config.auto_disable && !success {
                let _ = config_manager.disable_engine(name).await;
                log::warn!("Auto-disabled engine '{}' due to failed check", name);
            } else if config.auto_enable && success {
                // Check if reliability is above threshold
                let status = checker.get_status(name).await;
                if let Some(status) = status {
                    if status.reliability() >= config.min_reliability {
                        let _ = config_manager.enable_engine(name).await;
                        log::info!("Auto-enabled engine '{}' after successful check", name);
                    }
                }
            }
        }

        let result = CheckerResult::Ok {
            engines: engine_results,
            timestamp: current_timestamp(),
        };

        // Store result
        {
            let mut r = self.last_result.write().await;
            *r = result.clone();
        }

        // Mark as not running
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        log::info!("Checker completed: {} engines checked", results.len());
        result
    }

    /// Start the background scheduler
    pub async fn start(self: Arc<Self>) {
        let config = self.config.read().await.clone();

        if !config.enabled {
            log::info!("Checker scheduler is disabled");
            {
                let mut result = self.last_result.write().await;
                *result = CheckerResult::Disabled;
            }
            return;
        }

        let scheduler = self.clone();
        let interval_duration = Duration::from_secs(config.interval_secs);

        let handle = tokio::spawn(async move {
            let mut ticker = interval(interval_duration);

            // Run initial check
            log::info!("Starting initial engine check");
            scheduler.run_check().await;

            loop {
                ticker.tick().await;

                // Check if still enabled
                let config = scheduler.get_config().await;
                if !config.enabled {
                    log::info!("Checker scheduler disabled, stopping");
                    break;
                }

                log::info!("Running scheduled engine check");
                scheduler.run_check().await;
            }
        });

        // Store the handle
        {
            let mut task = self.task_handle.write().await;
            *task = Some(handle);
        }
    }

    /// Stop the background scheduler
    pub async fn stop(&self) {
        let mut task = self.task_handle.write().await;
        if let Some(handle) = task.take() {
            handle.abort();
            log::info!("Checker scheduler stopped");
        }
    }

    /// Get engine reliabilities from last check
    pub async fn get_engine_reliabilities(&self) -> HashMap<String, ReliabilityInfo> {
        let result = self.last_result.read().await.clone();

        match result {
            CheckerResult::Ok { engines, .. } => {
                let mut reliabilities = HashMap::new();

                for (name, check_result) in engines {
                    let reliability = if check_result.success {
                        Some(100.0)
                    } else if check_result.errors.as_ref().map(|e| e.contains_key("simple")).unwrap_or(false) {
                        Some(0.0)
                    } else {
                        None
                    };

                    let checker = if check_result.success {
                        vec![]
                    } else {
                        check_result.errors
                            .unwrap_or_default()
                            .keys()
                            .cloned()
                            .collect()
                    };

                    reliabilities.insert(name, ReliabilityInfo {
                        reliability,
                        errors: vec![],
                        checker,
                    });
                }

                reliabilities
            }
            _ => HashMap::new(),
        }
    }
}

impl Default for CheckerScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Global scheduler singleton
static SCHEDULER: std::sync::OnceLock<Arc<CheckerScheduler>> = std::sync::OnceLock::new();

/// Get the global scheduler instance
pub fn get_scheduler() -> Arc<CheckerScheduler> {
    SCHEDULER
        .get_or_init(|| Arc::new(CheckerScheduler::new()))
        .clone()
}

/// Initialize the scheduler with custom configuration
pub fn init_scheduler(config: SchedulerConfig) -> Arc<CheckerScheduler> {
    let scheduler = Arc::new(CheckerScheduler::with_config(config));
    let _ = SCHEDULER.set(scheduler.clone());
    scheduler
}

/// Start the global scheduler
pub async fn start_scheduler() {
    let scheduler = get_scheduler();
    scheduler.start().await;
}

/// Stop the global scheduler
pub async fn stop_scheduler() {
    let scheduler = get_scheduler();
    scheduler.stop().await;
}

/// Run a manual check
pub async fn run_manual_check() -> CheckerResult {
    let scheduler = get_scheduler();
    scheduler.run_check().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_config() {
        let config = SchedulerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 3600);
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = CheckerScheduler::new();
        assert!(!scheduler.is_running().await);

        let result = scheduler.get_result().await;
        assert!(matches!(result, CheckerResult::Unknown));
    }
}
