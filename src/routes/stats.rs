//! This module provides the engine statistics and health API routes.

use actix_web::{get, post, web, HttpResponse, http::header::ContentType};
use serde::{Deserialize, Serialize};

use crate::models::engine_health::{get_health_manager, EngineStats, EngineMetrics};

/// Response for engine stats API
#[derive(Serialize)]
pub struct StatsResponse {
    pub success: bool,
    pub stats: EngineStats,
}

/// Response for single engine stats
#[derive(Serialize)]
pub struct EngineStatsResponse {
    pub success: bool,
    pub engine: Option<EngineMetrics>,
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

/// Get statistics for all engines
///
/// GET /api/stats
#[get("/api/stats")]
pub async fn get_all_stats() -> HttpResponse {
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
    let health_manager = get_health_manager();
    health_manager.reset_stats();

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
