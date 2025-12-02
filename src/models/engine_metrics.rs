//! This module provides detailed engine metrics including histograms and scoring.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A simple histogram for tracking response times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    /// Bucket boundaries in milliseconds
    boundaries: Vec<f64>,
    /// Count in each bucket
    counts: Vec<u64>,
    /// Total count
    total_count: u64,
    /// Sum of all values
    sum: f64,
    /// Minimum value observed
    min: Option<f64>,
    /// Maximum value observed
    max: Option<f64>,
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Histogram {
    /// Create a new histogram with default buckets suitable for HTTP response times
    /// Buckets: 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000 ms
    pub fn new() -> Self {
        Self::with_boundaries(vec![
            10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
        ])
    }

    /// Create a histogram with custom bucket boundaries
    pub fn with_boundaries(boundaries: Vec<f64>) -> Self {
        let counts = vec![0u64; boundaries.len() + 1];
        Self {
            boundaries,
            counts,
            total_count: 0,
            sum: 0.0,
            min: None,
            max: None,
        }
    }

    /// Record a value in the histogram
    pub fn observe(&mut self, value: f64) {
        // Update sum and count
        self.sum += value;
        self.total_count += 1;

        // Update min/max
        self.min = Some(self.min.map_or(value, |m| m.min(value)));
        self.max = Some(self.max.map_or(value, |m| m.max(value)));

        // Find the right bucket
        let bucket_idx = self
            .boundaries
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(self.boundaries.len());

        self.counts[bucket_idx] += 1;
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        self.total_count
    }

    /// Get the sum of all observations
    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// Get the average value
    pub fn average(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }
        self.sum / self.total_count as f64
    }

    /// Get a percentile value (0-100)
    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.total_count == 0 {
            return None;
        }

        let target = (p / 100.0 * self.total_count as f64).ceil() as u64;
        let mut cumulative = 0u64;

        for (i, &count) in self.counts.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                // Return the upper boundary of this bucket
                if i < self.boundaries.len() {
                    return Some(self.boundaries[i]);
                } else {
                    // Last bucket, return max
                    return self.max;
                }
            }
        }

        self.max
    }

    /// Get the P50 (median) value
    pub fn p50(&self) -> Option<f64> {
        self.percentile(50.0)
    }

    /// Get the P80 value
    pub fn p80(&self) -> Option<f64> {
        self.percentile(80.0)
    }

    /// Get the P95 value
    pub fn p95(&self) -> Option<f64> {
        self.percentile(95.0)
    }

    /// Get the P99 value
    pub fn p99(&self) -> Option<f64> {
        self.percentile(99.0)
    }

    /// Get the minimum value
    pub fn min(&self) -> Option<f64> {
        self.min
    }

    /// Get the maximum value
    pub fn max(&self) -> Option<f64> {
        self.max
    }

    /// Reset the histogram
    pub fn reset(&mut self) {
        self.counts = vec![0u64; self.boundaries.len() + 1];
        self.total_count = 0;
        self.sum = 0.0;
        self.min = None;
        self.max = None;
    }
}

/// Engine score calculator
/// Score is based on:
/// - Reliability (success rate)
/// - Response time
/// - Result count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineScore {
    /// Base score from reliability (0-100)
    pub reliability_score: f64,
    /// Score adjustment from response time
    pub speed_score: f64,
    /// Score adjustment from result count
    pub result_score: f64,
    /// Final combined score (0-100)
    pub total_score: f64,
}

impl Default for EngineScore {
    fn default() -> Self {
        Self {
            reliability_score: 100.0,
            speed_score: 0.0,
            result_score: 0.0,
            total_score: 100.0,
        }
    }
}

impl EngineScore {
    /// Calculate score based on metrics
    ///
    /// # Arguments
    /// * `success_rate` - Success rate as a percentage (0-100)
    /// * `avg_response_time_ms` - Average response time in milliseconds
    /// * `avg_results_per_request` - Average number of results per successful request
    pub fn calculate(
        success_rate: f64,
        avg_response_time_ms: f64,
        avg_results_per_request: f64,
    ) -> Self {
        // Reliability score (0-100): direct mapping from success rate
        let reliability_score = success_rate.clamp(0.0, 100.0);

        // Speed score (0-20): faster is better
        // < 100ms = 20, 100-500ms = 15, 500-1000ms = 10, 1000-2000ms = 5, > 2000ms = 0
        let speed_score = if avg_response_time_ms <= 100.0 {
            20.0
        } else if avg_response_time_ms <= 500.0 {
            15.0 + 5.0 * (500.0 - avg_response_time_ms) / 400.0
        } else if avg_response_time_ms <= 1000.0 {
            10.0 + 5.0 * (1000.0 - avg_response_time_ms) / 500.0
        } else if avg_response_time_ms <= 2000.0 {
            5.0 * (2000.0 - avg_response_time_ms) / 1000.0
        } else {
            0.0
        };

        // Result score (0-10): more results is better (up to 20)
        let result_score = (avg_results_per_request.min(20.0) / 20.0) * 10.0;

        // Total score: weighted combination
        // Reliability is most important (70%), speed (20%), results (10%)
        let total_score =
            reliability_score * 0.7 + speed_score * 1.0 + result_score * 1.0;

        Self {
            reliability_score,
            speed_score,
            result_score,
            total_score: total_score.clamp(0.0, 100.0),
        }
    }
}

/// Rolling statistics for recent performance tracking
#[derive(Debug, Clone)]
pub struct RollingStats {
    /// Recent response times (last N requests)
    response_times: VecDeque<f64>,
    /// Recent result counts (last N requests)
    result_counts: VecDeque<usize>,
    /// Recent success/failure (last N requests)
    successes: VecDeque<bool>,
    /// Maximum number of samples to keep
    max_samples: usize,
}

impl Default for RollingStats {
    fn default() -> Self {
        Self::new(100)
    }
}

impl RollingStats {
    /// Create a new rolling stats tracker
    pub fn new(max_samples: usize) -> Self {
        Self {
            response_times: VecDeque::with_capacity(max_samples),
            result_counts: VecDeque::with_capacity(max_samples),
            successes: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self, response_time_ms: f64, result_count: usize) {
        self.push_with_limit(&mut self.response_times.clone(), response_time_ms);
        self.push_with_limit(&mut self.result_counts.clone(), result_count);
        self.push_with_limit(&mut self.successes.clone(), true);

        // Apply changes
        if self.response_times.len() >= self.max_samples {
            self.response_times.pop_front();
        }
        self.response_times.push_back(response_time_ms);

        if self.result_counts.len() >= self.max_samples {
            self.result_counts.pop_front();
        }
        self.result_counts.push_back(result_count);

        if self.successes.len() >= self.max_samples {
            self.successes.pop_front();
        }
        self.successes.push_back(true);
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        if self.successes.len() >= self.max_samples {
            self.successes.pop_front();
        }
        self.successes.push_back(false);
    }

    fn push_with_limit<T: Clone>(&self, _queue: &mut VecDeque<T>, _value: T) {
        // Helper function - actual push happens in record_success
    }

    /// Get the recent success rate (0-100)
    pub fn recent_success_rate(&self) -> f64 {
        if self.successes.is_empty() {
            return 100.0;
        }
        let success_count = self.successes.iter().filter(|&&s| s).count();
        (success_count as f64 / self.successes.len() as f64) * 100.0
    }

    /// Get the recent average response time
    pub fn recent_avg_response_time(&self) -> f64 {
        if self.response_times.is_empty() {
            return 0.0;
        }
        self.response_times.iter().sum::<f64>() / self.response_times.len() as f64
    }

    /// Get the recent average result count
    pub fn recent_avg_results(&self) -> f64 {
        if self.result_counts.is_empty() {
            return 0.0;
        }
        self.result_counts.iter().sum::<usize>() as f64 / self.result_counts.len() as f64
    }

    /// Get the number of samples
    pub fn sample_count(&self) -> usize {
        self.successes.len()
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.response_times.clear();
        self.result_counts.clear();
        self.successes.clear();
    }
}

/// Detailed engine statistics for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedEngineStats {
    /// Engine name
    pub name: String,

    // Counters
    /// Total requests sent
    pub sent_count: u64,
    /// Successful requests
    pub successful_count: u64,
    /// Failed requests
    pub error_count: u64,
    /// Total results returned
    pub result_count_total: u64,

    // Timing statistics
    /// Average response time (ms)
    pub time_total_avg: f64,
    /// P50 response time (ms)
    pub time_total_p50: Option<f64>,
    /// P80 response time (ms)
    pub time_total_p80: Option<f64>,
    /// P95 response time (ms)
    pub time_total_p95: Option<f64>,
    /// Minimum response time (ms)
    pub time_min: Option<f64>,
    /// Maximum response time (ms)
    pub time_max: Option<f64>,

    // Result statistics
    /// Average results per request
    pub result_count_avg: f64,

    // Reliability
    /// Reliability percentage (0-100)
    pub reliability: f64,

    // Score
    /// Engine score details
    pub score: EngineScore,

    // Status
    /// Whether the engine is suspended
    pub is_suspended: bool,
    /// Remaining suspension time in seconds
    pub suspension_remaining_secs: u64,
    /// Suspension reason
    pub suspension_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram() {
        let mut h = Histogram::new();

        // Add some values
        h.observe(50.0);
        h.observe(100.0);
        h.observe(150.0);
        h.observe(200.0);
        h.observe(500.0);

        assert_eq!(h.count(), 5);
        assert_eq!(h.sum(), 1000.0);
        assert_eq!(h.average(), 200.0);
        assert!(h.min().unwrap() <= 50.0);
        assert!(h.max().unwrap() >= 500.0);
    }

    #[test]
    fn test_engine_score() {
        // Perfect engine
        let score = EngineScore::calculate(100.0, 50.0, 15.0);
        assert!(score.total_score > 90.0);

        // Slow engine
        let score = EngineScore::calculate(100.0, 3000.0, 15.0);
        assert!(score.total_score < 90.0);

        // Unreliable engine
        let score = EngineScore::calculate(50.0, 50.0, 15.0);
        assert!(score.total_score < 70.0);
    }

    #[test]
    fn test_rolling_stats() {
        let mut stats = RollingStats::new(5);

        stats.record_success(100.0, 10);
        stats.record_success(200.0, 15);
        stats.record_failure();

        assert_eq!(stats.sample_count(), 3);
        assert!((stats.recent_success_rate() - 66.666).abs() < 1.0);
        assert_eq!(stats.recent_avg_response_time(), 150.0);
    }
}
