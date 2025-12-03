//! This module provides SearXNG-compatible metrics collection and analysis.
//!
//! Features (matching SearXNG's metrics system):
//! - Histogram-based response time tracking (P50, P80, P95 percentiles)
//! - Counter-based request/error/result tracking
//! - Per-engine reliability calculation
//! - Score calculation based on result counts and positions
//! - OpenMetrics (Prometheus) compatible export

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Configuration for histogram buckets
const HISTOGRAM_WIDTH: f64 = 0.1; // 100ms buckets
const HISTOGRAM_SIZE: usize = 150; // Up to 15 seconds

/// Histogram for tracking response time distributions
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Bucket counts (each bucket is HISTOGRAM_WIDTH seconds wide)
    buckets: Vec<u64>,
    /// Total sum of all values
    sum: f64,
    /// Total count of all values
    count: u64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Histogram {
    /// Create a new histogram
    pub fn new() -> Self {
        Self {
            buckets: vec![0; HISTOGRAM_SIZE],
            sum: 0.0,
            count: 0,
        }
    }

    /// Observe a value (in seconds)
    pub fn observe(&mut self, value: f64) {
        let bucket_idx = ((value / HISTOGRAM_WIDTH) as usize).min(HISTOGRAM_SIZE - 1);
        self.buckets[bucket_idx] += 1;
        self.sum += value;
        self.count += 1;
    }

    /// Get the percentile value (0-100)
    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.count == 0 {
            return None;
        }

        let target = (self.count as f64 * p / 100.0) as u64;
        let mut cumulative = 0u64;

        for (idx, &count) in self.buckets.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                // Return the middle of the bucket
                return Some((idx as f64 + 0.5) * HISTOGRAM_WIDTH);
            }
        }

        // Return the last bucket if we didn't find it
        Some((HISTOGRAM_SIZE as f64 - 0.5) * HISTOGRAM_WIDTH)
    }

    /// Get P50 (median)
    pub fn p50(&self) -> Option<f64> {
        self.percentile(50.0)
    }

    /// Get P80
    pub fn p80(&self) -> Option<f64> {
        self.percentile(80.0)
    }

    /// Get P95
    pub fn p95(&self) -> Option<f64> {
        self.percentile(95.0)
    }

    /// Get average
    pub fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    /// Get total sum
    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// Get total count
    pub fn count(&self) -> u64 {
        self.count
    }
}

/// Counter storage for tracking various metrics
#[derive(Debug, Clone, Default)]
pub struct Counters {
    /// Request counts
    pub sent: u64,
    /// Successful request counts
    pub successful: u64,
    /// Error counts
    pub errors: u64,
    /// Timeout counts
    pub timeouts: u64,
    /// Total score accumulated
    pub score: f64,
}

/// Engine-specific metrics
#[derive(Debug, Clone)]
pub struct EngineMetricsData {
    /// Request counters
    pub counters: Counters,
    /// Response time histogram (total time)
    pub time_total: Histogram,
    /// HTTP time histogram (network time only)
    pub time_http: Histogram,
    /// Result count histogram
    pub result_count: Histogram,
    /// Error type counts
    pub error_types: HashMap<String, u64>,
    /// Whether this engine passed the checker test
    pub checker_passed: bool,
    /// Failed checker tests
    pub checker_errors: HashMap<String, Vec<String>>,
}

impl Default for EngineMetricsData {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineMetricsData {
    /// Create new engine metrics
    pub fn new() -> Self {
        Self {
            counters: Counters::default(),
            time_total: Histogram::new(),
            time_http: Histogram::new(),
            result_count: Histogram::new(),
            error_types: HashMap::new(),
            checker_passed: true,
            checker_errors: HashMap::new(),
        }
    }

    /// Calculate reliability percentage (0-100)
    /// 
    /// SearXNG formula:
    /// - If checker fails on 'simple' test: 0%
    /// - If no requests sent: None
    /// - If checker passes and no errors: 100%
    /// - Otherwise: 100 - sum(error_percentages)
    pub fn reliability(&self) -> Option<f64> {
        // If simple test fails, reliability is 0
        if self.checker_errors.contains_key("simple") {
            return Some(0.0);
        }

        // If no requests, reliability is unknown
        if self.counters.sent == 0 {
            return None;
        }

        // If checker passed and no errors, 100%
        if self.checker_passed && self.counters.errors == 0 {
            return Some(100.0);
        }

        // Calculate based on error percentage
        let error_rate = self.counters.errors as f64 / self.counters.sent as f64;
        Some((1.0 - error_rate) * 100.0)
    }

    /// Get score per result
    pub fn score_per_result(&self) -> f64 {
        let result_sum = self.result_count.sum();
        if result_sum > 0.0 {
            self.counters.score / result_sum
        } else {
            0.0
        }
    }
}

/// Statistics for a single engine (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Engine name
    pub name: String,
    /// Total response time (P50)
    pub total: Option<f64>,
    /// Total response time (P80)
    pub total_p80: Option<f64>,
    /// Total response time (P95)
    pub total_p95: Option<f64>,
    /// HTTP time (P50)
    pub http: Option<f64>,
    /// HTTP time (P80)
    pub http_p80: Option<f64>,
    /// HTTP time (P95)
    pub http_p95: Option<f64>,
    /// Processing time (total - http)
    pub processing: Option<f64>,
    /// Processing time (P80)
    pub processing_p80: Option<f64>,
    /// Processing time (P95)
    pub processing_p95: Option<f64>,
    /// Accumulated score
    pub score: f64,
    /// Score per result
    pub score_per_result: f64,
    /// Median result count
    pub result_count: Option<f64>,
    /// Total request count
    pub sent_count: u64,
    /// Successful request count
    pub successful_count: u64,
    /// Reliability percentage
    pub reliability: Option<f64>,
}

/// Global metrics storage
pub struct MetricsStorage {
    /// Per-engine metrics
    engines: RwLock<HashMap<String, EngineMetricsData>>,
    /// Whether metrics collection is enabled
    enabled: bool,
}

impl MetricsStorage {
    /// Create new metrics storage
    pub fn new(enabled: bool) -> Self {
        Self {
            engines: RwLock::new(HashMap::new()),
            enabled,
        }
    }

    /// Ensure an engine exists in storage
    fn ensure_engine(&self, name: &str) {
        let mut engines = self.engines.write().unwrap();
        if !engines.contains_key(name) {
            engines.insert(name.to_string(), EngineMetricsData::new());
        }
    }

    /// Record a request was sent
    pub fn record_sent(&self, engine_name: &str) {
        if !self.enabled {
            return;
        }
        self.ensure_engine(engine_name);
        let mut engines = self.engines.write().unwrap();
        if let Some(metrics) = engines.get_mut(engine_name) {
            metrics.counters.sent += 1;
        }
    }

    /// Record a successful request
    pub fn record_success(
        &self,
        engine_name: &str,
        total_time_secs: f64,
        http_time_secs: f64,
        result_count: usize,
        score: f64,
    ) {
        if !self.enabled {
            return;
        }
        self.ensure_engine(engine_name);
        let mut engines = self.engines.write().unwrap();
        if let Some(metrics) = engines.get_mut(engine_name) {
            metrics.counters.successful += 1;
            metrics.counters.score += score;
            metrics.time_total.observe(total_time_secs);
            metrics.time_http.observe(http_time_secs);
            metrics.result_count.observe(result_count as f64);
        }
    }

    /// Record an error
    pub fn record_error(&self, engine_name: &str, error_type: &str) {
        if !self.enabled {
            return;
        }
        self.ensure_engine(engine_name);
        let mut engines = self.engines.write().unwrap();
        if let Some(metrics) = engines.get_mut(engine_name) {
            metrics.counters.errors += 1;
            *metrics.error_types.entry(error_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a timeout
    pub fn record_timeout(&self, engine_name: &str) {
        if !self.enabled {
            return;
        }
        self.ensure_engine(engine_name);
        let mut engines = self.engines.write().unwrap();
        if let Some(metrics) = engines.get_mut(engine_name) {
            metrics.counters.timeouts += 1;
            metrics.counters.errors += 1;
            *metrics.error_types.entry("timeout".to_string()).or_insert(0) += 1;
        }
    }

    /// Set checker result for an engine
    pub fn set_checker_result(&self, engine_name: &str, passed: bool, errors: HashMap<String, Vec<String>>) {
        self.ensure_engine(engine_name);
        let mut engines = self.engines.write().unwrap();
        if let Some(metrics) = engines.get_mut(engine_name) {
            metrics.checker_passed = passed;
            metrics.checker_errors = errors;
        }
    }

    /// Get statistics for all engines
    pub fn get_all_stats(&self, engine_names: &[String]) -> Vec<EngineStats> {
        let engines = self.engines.read().unwrap();
        let mut stats = Vec::new();
        let mut max_time_total: f64 = 0.0;

        for name in engine_names {
            if let Some(metrics) = engines.get(name) {
                if metrics.counters.sent == 0 {
                    continue;
                }

                let total = metrics.time_total.p50();
                let total_p80 = metrics.time_total.p80();
                let total_p95 = metrics.time_total.p95();
                let http = metrics.time_http.p50();
                let http_p80 = metrics.time_http.p80();
                let http_p95 = metrics.time_http.p95();

                if let Some(t) = total {
                    max_time_total = max_time_total.max(t);
                }

                let processing = match (total, http) {
                    (Some(t), Some(h)) => Some(t - h),
                    _ => None,
                };
                let processing_p80 = match (total_p80, http_p80) {
                    (Some(t), Some(h)) => Some(t - h),
                    _ => None,
                };
                let processing_p95 = match (total_p95, http_p95) {
                    (Some(t), Some(h)) => Some(t - h),
                    _ => None,
                };

                stats.push(EngineStats {
                    name: name.clone(),
                    total: total.map(|t| (t * 10.0).round() / 10.0),
                    total_p80: total_p80.map(|t| (t * 10.0).round() / 10.0),
                    total_p95: total_p95.map(|t| (t * 10.0).round() / 10.0),
                    http: http.map(|t| (t * 10.0).round() / 10.0),
                    http_p80: http_p80.map(|t| (t * 10.0).round() / 10.0),
                    http_p95: http_p95.map(|t| (t * 10.0).round() / 10.0),
                    processing: processing.map(|t| (t * 10.0).round() / 10.0),
                    processing_p80: processing_p80.map(|t| (t * 10.0).round() / 10.0),
                    processing_p95: processing_p95.map(|t| (t * 10.0).round() / 10.0),
                    score: metrics.counters.score,
                    score_per_result: metrics.score_per_result(),
                    result_count: metrics.result_count.p50(),
                    sent_count: metrics.counters.sent,
                    successful_count: metrics.counters.successful,
                    reliability: metrics.reliability(),
                });
            }
        }

        stats
    }

    /// Get reliability information for all engines
    pub fn get_reliabilities(&self, engine_names: &[String]) -> HashMap<String, ReliabilityInfo> {
        let engines = self.engines.read().unwrap();
        let mut reliabilities = HashMap::new();

        for name in engine_names {
            if let Some(metrics) = engines.get(name) {
                let errors: Vec<ErrorInfo> = metrics
                    .error_types
                    .iter()
                    .filter_map(|(error_type, count)| {
                        if metrics.counters.sent > 0 {
                            let percentage = (*count as f64 / metrics.counters.sent as f64) * 100.0;
                            Some(ErrorInfo {
                                error_type: error_type.clone(),
                                count: *count,
                                percentage,
                                secondary: error_type == "timeout", // Timeouts are secondary
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                let checker_errors: Vec<String> = if metrics.checker_passed {
                    vec![]
                } else {
                    metrics.checker_errors.keys().cloned().collect()
                };

                reliabilities.insert(
                    name.clone(),
                    ReliabilityInfo {
                        reliability: metrics.reliability(),
                        errors,
                        checker: checker_errors,
                    },
                );
            }
        }

        reliabilities
    }

    /// Reset all metrics
    pub fn reset(&self) {
        let mut engines = self.engines.write().unwrap();
        engines.clear();
    }

    /// Get raw metrics for an engine
    pub fn get_engine_metrics(&self, name: &str) -> Option<EngineMetricsData> {
        let engines = self.engines.read().unwrap();
        engines.get(name).cloned()
    }
}

/// Error information for reliability reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error type
    pub error_type: String,
    /// Number of occurrences
    pub count: u64,
    /// Percentage of total requests
    pub percentage: f64,
    /// Whether this is a secondary error (warnings)
    pub secondary: bool,
}

/// Reliability information for an engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityInfo {
    /// Reliability percentage (0-100)
    pub reliability: Option<f64>,
    /// Error details
    pub errors: Vec<ErrorInfo>,
    /// Failed checker tests
    pub checker: Vec<String>,
}

/// Generate OpenMetrics (Prometheus) compatible output
pub fn openmetrics(engine_stats: &[EngineStats], reliabilities: &HashMap<String, ReliabilityInfo>) -> String {
    let mut output = String::new();

    // Engine sent count
    output.push_str("# HELP searxng_engines_sent_total Total number of search requests sent\n");
    output.push_str("# TYPE searxng_engines_sent_total counter\n");
    for stat in engine_stats {
        output.push_str(&format!(
            "searxng_engines_sent_total{{engine_name=\"{}\"}} {}\n",
            stat.name, stat.sent_count
        ));
    }

    // Engine successful count
    output.push_str("# HELP searxng_engines_successful_total Total number of successful search requests\n");
    output.push_str("# TYPE searxng_engines_successful_total counter\n");
    for stat in engine_stats {
        output.push_str(&format!(
            "searxng_engines_successful_total{{engine_name=\"{}\"}} {}\n",
            stat.name, stat.successful_count
        ));
    }

    // Engine response time
    output.push_str("# HELP searxng_engines_time_total Total response time in seconds\n");
    output.push_str("# TYPE searxng_engines_time_total gauge\n");
    for stat in engine_stats {
        if let Some(total) = stat.total {
            output.push_str(&format!(
                "searxng_engines_time_total{{engine_name=\"{}\"}} {}\n",
                stat.name, total
            ));
        }
    }

    // Engine reliability
    output.push_str("# HELP searxng_engines_reliability_total Engine reliability percentage\n");
    output.push_str("# TYPE searxng_engines_reliability_total gauge\n");
    for stat in engine_stats {
        let reliability = reliabilities
            .get(&stat.name)
            .and_then(|r| r.reliability)
            .unwrap_or(0.0);
        output.push_str(&format!(
            "searxng_engines_reliability_total{{engine_name=\"{}\"}} {}\n",
            stat.name, reliability
        ));
    }

    // Engine score
    output.push_str("# HELP searxng_engines_score_total Accumulated score\n");
    output.push_str("# TYPE searxng_engines_score_total counter\n");
    for stat in engine_stats {
        output.push_str(&format!(
            "searxng_engines_score_total{{engine_name=\"{}\"}} {}\n",
            stat.name, stat.score
        ));
    }

    output
}

/// Global metrics storage singleton
static METRICS_STORAGE: std::sync::OnceLock<Arc<MetricsStorage>> = std::sync::OnceLock::new();

/// Get the global metrics storage
pub fn get_metrics() -> Arc<MetricsStorage> {
    METRICS_STORAGE
        .get_or_init(|| Arc::new(MetricsStorage::new(true)))
        .clone()
}

/// Initialize metrics storage with custom settings
pub fn init_metrics(enabled: bool) -> Arc<MetricsStorage> {
    let storage = Arc::new(MetricsStorage::new(enabled));
    let _ = METRICS_STORAGE.set(storage.clone());
    storage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram() {
        let mut h = Histogram::new();
        
        // Add some values
        for _ in 0..10 {
            h.observe(0.5); // 500ms
        }
        for _ in 0..5 {
            h.observe(1.0); // 1000ms
        }

        assert_eq!(h.count(), 15);
        assert!(h.p50().is_some());
        assert!(h.p50().unwrap() < 1.0);
    }

    #[test]
    fn test_engine_metrics() {
        let mut metrics = EngineMetricsData::new();
        
        metrics.counters.sent = 100;
        metrics.counters.successful = 90;
        metrics.counters.errors = 10;

        let reliability = metrics.reliability();
        assert!(reliability.is_some());
        assert!((reliability.unwrap() - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_reliability_with_checker_failure() {
        let mut metrics = EngineMetricsData::new();
        
        metrics.counters.sent = 100;
        metrics.counters.successful = 90;
        metrics.checker_passed = false;
        metrics.checker_errors.insert("simple".to_string(), vec!["Test failed".to_string()]);

        let reliability = metrics.reliability();
        assert_eq!(reliability, Some(0.0));
    }

    #[test]
    fn test_metrics_storage() {
        let storage = MetricsStorage::new(true);
        
        storage.record_sent("test_engine");
        storage.record_success("test_engine", 0.5, 0.3, 10, 5.0);
        storage.record_sent("test_engine");
        storage.record_error("test_engine", "timeout");

        let stats = storage.get_all_stats(&["test_engine".to_string()]);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].sent_count, 2);
        assert_eq!(stats[0].successful_count, 1);
    }
}
