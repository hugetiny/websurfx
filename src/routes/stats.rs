//! This module provides the engine statistics and health API routes.
//!
//! SearXNG-compatible endpoints:
//! - /api/stats - Engine statistics with P50/P80/P95 response times
//! - /api/reliabilities - Engine reliability information
//! - /api/metrics - OpenMetrics (Prometheus) format
//! - /api/checker - Background checker status

use actix_web::{get, post, web, HttpResponse, http::header::ContentType};
use serde::{Deserialize, Serialize};

use crate::models::engine_health::{get_health_manager, EngineStats as HealthStats, EngineMetrics as HealthMetrics};
use crate::metrics::{get_metrics, openmetrics, EngineStats};
use crate::checker_scheduler::{get_scheduler, run_manual_check, CheckerResult};

/// Response for engine stats API (legacy)
#[derive(Serialize)]
pub struct StatsResponse {
    pub success: bool,
    pub stats: HealthStats,
}

/// Response for single engine stats
#[derive(Serialize)]
pub struct EngineStatsResponse {
    pub success: bool,
    pub engine: Option<HealthMetrics>,
    pub error: Option<String>,
}

/// Response for suspended engines list
#[derive(Serialize)]
pub struct SuspendedEnginesResponse {
    pub success: bool,
    pub suspended_engines: Vec<SuspendedEngineInfo>,
}

#[derive(Serialize)]
pub struct SuspendedEngineInfo {
    pub name: String,
    pub remaining_seconds: u64,
    pub reason: String,
}

/// Request body for resume engine
#[derive(Deserialize)]
pub struct ResumeRequest {
    pub engine: String,
}

/// Generic API response
#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

/// SearXNG-compatible stats response
#[derive(Serialize)]
pub struct SearxngStatsResponse {
    /// Statistics for each engine
    pub time: Vec<EngineStats>,
    /// Maximum response time
    pub max_time: f64,
    /// Maximum result count
    pub max_result_count: f64,
}

/// Query parameters for stats endpoint
#[derive(Deserialize)]
pub struct StatsQuery {
    /// Specific engine to get stats for
    pub engine: Option<String>,
    /// Format: "searxng" for SearXNG-compatible format
    pub format: Option<String>,
}

/// Get statistics for all engines
///
/// GET /api/stats
///
/// Query params:
/// - format=searxng: Return SearXNG-compatible format with P50/P80/P95
/// - engine=name: Get stats for specific engine only
#[get("/api/stats")]
pub async fn get_all_stats(query: web::Query<StatsQuery>) -> HttpResponse {
    // Check if SearXNG format is requested
    if query.format.as_deref() == Some("searxng") {
        let metrics = get_metrics();

        let engine_names: Vec<String> = if let Some(ref engine) = query.engine {
            vec![engine.clone()]
        } else {
            vec![] // All engines
        };

        let stats = metrics.get_all_stats(&engine_names);

        let max_time = stats.iter()
            .filter_map(|s| s.total)
            .fold(0.0_f64, |max, t| max.max(t));

        let max_result_count = stats.iter()
            .filter_map(|s| s.result_count)
            .fold(0.0_f64, |max, r| max.max(r));

        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(SearxngStatsResponse {
                time: stats,
                max_time,
                max_result_count,
            });
    }

    // Legacy format
    let health_manager = get_health_manager();
    let stats = health_manager.get_all_stats();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(StatsResponse {
            success: true,
            stats,
        })
}

/// Get statistics for a specific engine
///
/// GET /api/stats/{engine_name}
#[get("/api/stats/{engine_name}")]
pub async fn get_engine_stats(path: web::Path<String>) -> HttpResponse {
    let engine_name = path.into_inner();

    // Try new metrics system first
    let metrics = get_metrics();
    let stats = metrics.get_all_stats(&[engine_name.clone()]);
    let reliabilities = metrics.get_reliabilities(&[engine_name.clone()]);

    if !stats.is_empty() {
        let stat = &stats[0];
        let reliability = reliabilities.get(&engine_name);

        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(serde_json::json!({
                "success": true,
                "engine": engine_name,
                "stats": stat,
                "reliability": reliability,
            }));
    }

    // Fall back to legacy health manager
    let health_manager = get_health_manager();

    match health_manager.get_engine_metrics(&engine_name) {
        Some(metrics) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(EngineStatsResponse {
                success: true,
                engine: Some(metrics),
                error: None,
            }),
        None => HttpResponse::NotFound()
            .content_type(ContentType::json())
            .json(EngineStatsResponse {
                success: false,
                engine: None,
                error: Some(format!("Engine '{}' not found", engine_name)),
            }),
    }
}

/// Get list of currently suspended engines
///
/// GET /api/stats/suspended
#[get("/api/stats/suspended")]
pub async fn get_suspended_engines() -> HttpResponse {
    let health_manager = get_health_manager();
    let suspended = health_manager.get_suspended_engines();

    let suspended_engines: Vec<SuspendedEngineInfo> = suspended
        .into_iter()
        .map(|(name, remaining, reason)| SuspendedEngineInfo {
            name,
            remaining_seconds: remaining,
            reason,
        })
        .collect();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(SuspendedEnginesResponse {
            success: true,
            suspended_engines,
        })
}

/// Resume a suspended engine manually
///
/// POST /api/stats/resume
#[post("/api/stats/resume")]
pub async fn resume_engine(body: web::Json<ResumeRequest>) -> HttpResponse {
    let health_manager = get_health_manager();
    let engine_name = &body.engine;

    if health_manager.is_suspended(engine_name) {
        health_manager.resume_engine(engine_name);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(ApiResponse {
                success: true,
                message: format!("Engine '{}' has been resumed", engine_name),
            })
    } else {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(ApiResponse {
                success: true,
                message: format!("Engine '{}' was not suspended", engine_name),
            })
    }
}

/// Reset all statistics (but keep suspension status)
///
/// POST /api/stats/reset
#[post("/api/stats/reset")]
pub async fn reset_stats() -> HttpResponse {
    // Reset both old and new metrics
    let health_manager = get_health_manager();
    health_manager.reset_stats();

    let metrics = get_metrics();
    metrics.reset();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(ApiResponse {
            success: true,
            message: "All statistics have been reset".to_string(),
        })
}

/// Resume all suspended engines
///
/// POST /api/stats/resume-all
#[post("/api/stats/resume-all")]
pub async fn resume_all_engines() -> HttpResponse {
    let health_manager = get_health_manager();
    let suspended = health_manager.get_suspended_engines();
    let count = suspended.len();

    for (name, _, _) in suspended {
        health_manager.resume_engine(&name);
    }

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(ApiResponse {
            success: true,
            message: format!("Resumed {} suspended engine(s)", count),
        })
}

// ============= SearXNG-compatible API endpoints =============

/// Get engine reliabilities (SearXNG-compatible)
///
/// GET /api/reliabilities
#[get("/api/reliabilities")]
pub async fn get_reliabilities() -> HttpResponse {
    let metrics = get_metrics();
    let scheduler = get_scheduler();

    // Get reliabilities from metrics
    let mut reliabilities = metrics.get_reliabilities(&[]);

    // Merge with checker results
    let checker_reliabilities = scheduler.get_engine_reliabilities().await;
    for (name, info) in checker_reliabilities {
        reliabilities.entry(name).or_insert(info);
    }

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(serde_json::json!({
            "reliabilities": reliabilities
        }))
}

/// Get OpenMetrics (Prometheus) format metrics
///
/// GET /api/metrics
#[get("/api/metrics")]
pub async fn get_openmetrics() -> HttpResponse {
    let metrics = get_metrics();

    let stats = metrics.get_all_stats(&[]);
    let reliabilities = metrics.get_reliabilities(&[]);

    let output = openmetrics(&stats, &reliabilities);

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output)
}

/// Get checker status
///
/// GET /api/checker
#[get("/api/checker")]
pub async fn get_checker_status() -> HttpResponse {
    let scheduler = get_scheduler();
    let result = scheduler.get_result().await;
    let is_running = scheduler.is_running().await;
    let config = scheduler.get_config().await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(serde_json::json!({
            "enabled": config.enabled,
            "running": is_running,
            "result": result,
        }))
}

/// Run a manual check
///
/// POST /api/checker/run
#[post("/api/checker/run")]
pub async fn run_checker() -> HttpResponse {
    let result = run_manual_check().await;

    match result {
        CheckerResult::Running => {
            HttpResponse::Conflict()
                .content_type(ContentType::json())
                .json(serde_json::json!({
                    "success": false,
                    "message": "Checker is already running"
                }))
        }
        CheckerResult::Ok { engines, timestamp } => {
            let passed = engines.values().filter(|e| e.success).count();
            let failed = engines.len() - passed;

            HttpResponse::Ok()
                .content_type(ContentType::json())
                .json(serde_json::json!({
                    "success": true,
                    "message": format!("Checked {} engines: {} passed, {} failed",
                        engines.len(), passed, failed),
                    "engines": engines,
                    "timestamp": timestamp,
                }))
        }
        CheckerResult::Error { message, timestamp } => {
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .json(serde_json::json!({
                    "success": false,
                    "message": message,
                    "timestamp": timestamp,
                }))
        }
        _ => {
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .json(serde_json::json!({
                    "success": false,
                    "message": "Unknown error"
                }))
        }
    }
}

/// Update checker configuration
#[derive(Deserialize)]
pub struct UpdateSchedulerRequest {
    pub enabled: Option<bool>,
    pub interval_secs: Option<u64>,
    pub max_duration_secs: Option<u64>,
    pub engines: Option<Vec<String>>,
    pub auto_disable: Option<bool>,
    pub auto_enable: Option<bool>,
    pub min_reliability: Option<f64>,
}

/// Update checker scheduler configuration
///
/// POST /api/checker/config
#[post("/api/checker/config")]
pub async fn update_checker_config(body: web::Json<UpdateSchedulerRequest>) -> HttpResponse {
    let scheduler = get_scheduler();
    let mut config = scheduler.get_config().await;

    if let Some(v) = body.enabled {
        config.enabled = v;
    }
    if let Some(v) = body.interval_secs {
        config.interval_secs = v;
    }
    if let Some(v) = body.max_duration_secs {
        config.max_duration_secs = v;
    }
    if let Some(ref v) = body.engines {
        config.engines = v.clone();
    }
    if let Some(v) = body.auto_disable {
        config.auto_disable = v;
    }
    if let Some(v) = body.auto_enable {
        config.auto_enable = v;
    }
    if let Some(v) = body.min_reliability {
        config.min_reliability = v;
    }

    scheduler.update_config(config.clone()).await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(serde_json::json!({
            "success": true,
            "message": "Configuration updated",
            "config": config,
        }))
}

/// Get checker configuration
///
/// GET /api/checker/config
#[get("/api/checker/config")]
pub async fn get_checker_config() -> HttpResponse {
    let scheduler = get_scheduler();
    let config = scheduler.get_config().await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(config)
}
