//! This module provides automatic engine configuration management.
//!
//! Features:
//! - Automatically enable/disable engines based on health checks
//! - Persist engine states to config file
//! - Smart engine selection based on reliability and response time
//! - Rate limiting awareness

use crate::engine_checker::{get_checker, CheckResult, EngineCheckStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Engine configuration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Engine name
    pub name: String,
    /// Whether the engine is enabled in config
    pub enabled: bool,
    /// Whether the engine is healthy (from checker)
    pub healthy: bool,
    /// Reliability percentage (0-100)
    pub reliability: f64,
    /// Average response time in ms
    pub avg_response_time_ms: f64,
    /// Reason for current state
    pub reason: Option<String>,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Auto-configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConfigSettings {
    /// Minimum reliability percentage to keep engine enabled (0-100)
    pub min_reliability: f64,
    /// Maximum response time to keep engine enabled (ms)
    pub max_response_time_ms: u64,
    /// Number of consecutive failures before disabling
    pub disable_after_failures: u32,
    /// Number of consecutive successes to re-enable
    pub enable_after_successes: u32,
    /// Maximum number of engines to enable simultaneously
    pub max_concurrent_engines: usize,
    /// Whether to auto-enable new engines
    pub auto_enable_new: bool,
    /// Whether to persist changes to config file
    pub persist_changes: bool,
}

impl Default for AutoConfigSettings {
    fn default() -> Self {
        Self {
            min_reliability: 50.0,
            max_response_time_ms: 10000,
            disable_after_failures: 3,
            enable_after_successes: 2,
            max_concurrent_engines: 20, // SearXNG-like default
            auto_enable_new: false,
            persist_changes: true,
        }
    }
}

/// Manages engine configuration based on health checks
pub struct ConfigManager {
    /// Current engine configurations
    configs: RwLock<HashMap<String, EngineConfig>>,
    /// Auto-configuration settings
    settings: RwLock<AutoConfigSettings>,
    /// Path to config file
    config_path: Option<String>,
}

impl ConfigManager {
    /// Create a new config manager
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            settings: RwLock::new(AutoConfigSettings::default()),
            config_path: None,
        }
    }

    /// Set the config file path
    pub fn with_config_path(mut self, path: &str) -> Self {
        self.config_path = Some(path.to_string());
        self
    }

    /// Update settings
    pub async fn update_settings(&self, settings: AutoConfigSettings) {
        let mut s = self.settings.write().await;
        *s = settings;
    }

    /// Get current settings
    pub async fn get_settings(&self) -> AutoConfigSettings {
        self.settings.read().await.clone()
    }

    /// Initialize engine configuration from check results
    pub async fn init_from_check_results(&self, results: &HashMap<String, CheckResult>) {
        let mut configs = self.configs.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for (name, result) in results {
            let config = EngineConfig {
                name: name.clone(),
                enabled: result.success,
                healthy: result.success,
                reliability: if result.success { 100.0 } else { 0.0 },
                avg_response_time_ms: result.response_time_ms as f64,
                reason: if result.success {
                    Some(format!("Initial check passed ({} results)", result.result_count))
                } else {
                    result.error.clone()
                },
                last_updated: now,
            };
            configs.insert(name.clone(), config);
        }
    }

    /// Update engine configuration based on check status
    pub async fn update_from_status(&self, status: &EngineCheckStatus) {
        let settings = self.settings.read().await.clone();
        let mut configs = self.configs.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let existing = configs.get(&status.name);
        let was_enabled = existing.map(|c| c.enabled).unwrap_or(false);

        // Calculate average response time from last result
        let avg_response_time = status
            .last_result
            .as_ref()
            .map(|r| r.response_time_ms as f64)
            .unwrap_or(0.0);

        // Determine if engine should be enabled
        let should_enable = status.is_healthy
            && status.reliability() >= settings.min_reliability
            && avg_response_time <= settings.max_response_time_ms as f64;

        // Determine reason
        let reason = if !status.is_healthy {
            Some(format!(
                "Unhealthy: {} consecutive failures",
                status.consecutive_failures
            ))
        } else if status.reliability() < settings.min_reliability {
            Some(format!(
                "Low reliability: {:.1}% (min: {:.1}%)",
                status.reliability(),
                settings.min_reliability
            ))
        } else if avg_response_time > settings.max_response_time_ms as f64 {
            Some(format!(
                "Slow response: {:.0}ms (max: {}ms)",
                avg_response_time, settings.max_response_time_ms
            ))
        } else if should_enable && !was_enabled {
            Some("Re-enabled: health recovered".to_string())
        } else {
            None
        };

        let config = EngineConfig {
            name: status.name.clone(),
            enabled: should_enable,
            healthy: status.is_healthy,
            reliability: status.reliability(),
            avg_response_time_ms: avg_response_time,
            reason,
            last_updated: now,
        };

        configs.insert(status.name.clone(), config);
    }

    /// Get enabled engines (sorted by reliability and speed)
    pub async fn get_enabled_engines(&self) -> Vec<EngineConfig> {
        let configs = self.configs.read().await;
        let settings = self.settings.read().await;

        let mut enabled: Vec<_> = configs
            .values()
            .filter(|c| c.enabled && c.healthy)
            .cloned()
            .collect();

        // Sort by reliability (desc) then by response time (asc)
        enabled.sort_by(|a, b| {
            b.reliability
                .partial_cmp(&a.reliability)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.avg_response_time_ms
                        .partial_cmp(&b.avg_response_time_ms)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // Limit to max concurrent engines
        enabled.truncate(settings.max_concurrent_engines);
        enabled
    }

    /// Get disabled engines
    pub async fn get_disabled_engines(&self) -> Vec<EngineConfig> {
        let configs = self.configs.read().await;
        configs
            .values()
            .filter(|c| !c.enabled || !c.healthy)
            .cloned()
            .collect()
    }

    /// Get all engine configs
    pub async fn get_all_configs(&self) -> Vec<EngineConfig> {
        let configs = self.configs.read().await;
        configs.values().cloned().collect()
    }

    /// Manually enable an engine
    pub async fn enable_engine(&self, name: &str) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(config) = configs.get_mut(name) {
            config.enabled = true;
            config.reason = Some("Manually enabled".to_string());
            config.last_updated = now;
            Ok(())
        } else {
            // Create new config for unknown engine
            let config = EngineConfig {
                name: name.to_string(),
                enabled: true,
                healthy: true, // Assume healthy until checked
                reliability: 100.0,
                avg_response_time_ms: 0.0,
                reason: Some("Manually enabled".to_string()),
                last_updated: now,
            };
            configs.insert(name.to_string(), config);
            Ok(())
        }
    }

    /// Manually disable an engine
    pub async fn disable_engine(&self, name: &str) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(config) = configs.get_mut(name) {
            config.enabled = false;
            config.reason = Some("Manually disabled".to_string());
            config.last_updated = now;
            Ok(())
        } else {
            Err(format!("Engine '{}' not found", name))
        }
    }

    /// Generate Lua config string for engines
    pub async fn generate_lua_config(&self) -> String {
        let configs = self.configs.read().await;
        let mut lines = vec!["upstream_search_engines = {".to_string()];

        // Group by category for better organization
        let mut sorted: Vec<_> = configs.values().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        for config in sorted {
            let enabled_str = if config.enabled { "true" } else { "false" };
            let comment = if let Some(ref reason) = config.reason {
                format!(" -- {}", reason)
            } else if !config.healthy {
                " -- Unhealthy".to_string()
            } else {
                String::new()
            };

            lines.push(format!(
                "\t{} = {},{}",
                config.name, enabled_str, comment
            ));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Save current configuration to a JSON file
    pub async fn save_to_json(&self, path: &str) -> Result<(), String> {
        let configs = self.configs.read().await;
        let settings = self.settings.read().await;

        let data = serde_json::json!({
            "engines": configs.clone(),
            "settings": settings.clone(),
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(path, json)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Load configuration from a JSON file
    pub async fn load_from_json(&self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let data: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if let Some(engines) = data.get("engines") {
            let engines: HashMap<String, EngineConfig> = serde_json::from_value(engines.clone())
                .map_err(|e| format!("Failed to parse engines: {}", e))?;
            let mut configs = self.configs.write().await;
            *configs = engines;
        }

        if let Some(settings) = data.get("settings") {
            let settings: AutoConfigSettings = serde_json::from_value(settings.clone())
                .map_err(|e| format!("Failed to parse settings: {}", e))?;
            let mut s = self.settings.write().await;
            *s = settings;
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global config manager instance
static CONFIG_MANAGER: std::sync::OnceLock<Arc<ConfigManager>> = std::sync::OnceLock::new();

/// Get the global config manager instance
pub fn get_config_manager() -> Arc<ConfigManager> {
    CONFIG_MANAGER
        .get_or_init(|| Arc::new(ConfigManager::new()))
        .clone()
}

/// Run a full engine check and update configuration
pub async fn run_auto_config(engine_names: &[&str]) -> HashMap<String, EngineConfig> {
    let checker = get_checker();
    let manager = get_config_manager();

    // Check all engines
    let results = checker.check_engines(engine_names).await;

    // Update config from results
    manager.init_from_check_results(&results).await;

    // Get all statuses and update
    let statuses = checker.get_all_statuses().await;
    for status in statuses.values() {
        manager.update_from_status(status).await;
    }

    // Return all configs
    let configs = manager.get_all_configs().await;
    configs.into_iter().map(|c| (c.name.clone(), c)).collect()
}

/// Get the list of engines that should be used for a search
pub async fn get_active_engines() -> Vec<String> {
    let manager = get_config_manager();
    let enabled = manager.get_enabled_engines().await;
    enabled.into_iter().map(|c| c.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_config_settings_default() {
        let settings = AutoConfigSettings::default();
        assert_eq!(settings.min_reliability, 50.0);
        assert_eq!(settings.max_concurrent_engines, 20);
        assert!(!settings.auto_enable_new);
    }

    #[test]
    fn test_engine_config() {
        let config = EngineConfig {
            name: "test".to_string(),
            enabled: true,
            healthy: true,
            reliability: 95.0,
            avg_response_time_ms: 100.0,
            reason: None,
            last_updated: 0,
        };

        assert!(config.enabled);
        assert!(config.healthy);
    }

    #[tokio::test]
    async fn test_config_manager() {
        let manager = ConfigManager::new();

        // Test manual enable/disable
        manager.enable_engine("test_engine").await.unwrap();
        let configs = manager.get_all_configs().await;
        assert!(!configs.is_empty());

        manager.disable_engine("test_engine").await.unwrap();
        let disabled = manager.get_disabled_engines().await;
        assert!(!disabled.is_empty());
    }

    #[tokio::test]
    async fn test_generate_lua_config() {
        let manager = ConfigManager::new();
        manager.enable_engine("baidu").await.unwrap();
        manager.enable_engine("bing").await.unwrap();
        manager.disable_engine("google").await.ok();

        let lua = manager.generate_lua_config().await;
        assert!(lua.contains("upstream_search_engines"));
        assert!(lua.contains("baidu"));
    }
}
