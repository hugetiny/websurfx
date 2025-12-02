//! This module provides the engine checker API routes.

use actix_web::{get, post, web, HttpResponse, http::header::ContentType};
use serde::{Deserialize, Serialize};

use crate::engine_checker::{get_checker, CheckResult, EngineCheckStatus};
use std::collections::HashMap;

/// Response for checker status
#[derive(Serialize)]
pub struct CheckerStatusResponse {
    pub success: bool,
    pub engines: HashMap<String, EngineCheckStatus>,
}

/// Response for single engine check
#[derive(Serialize)]
pub struct CheckEngineResponse {
    pub success: bool,
    pub engine: String,
    pub result: CheckResult,
}

/// Response for multiple engine check
#[derive(Serialize)]
pub struct CheckEnginesResponse {
    pub success: bool,
    pub results: HashMap<String, CheckResult>,
    pub healthy_count: usize,
    pub unhealthy_count: usize,
}

/// Request body for checking specific engines
#[derive(Deserialize)]
pub struct CheckEnginesRequest {
    pub engines: Vec<String>,
}

/// Generic API response
#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

/// Get all engine check statuses
///
/// GET /api/checker/status
#[get("/api/checker/status")]
pub async fn get_checker_status() -> HttpResponse {
    let checker = get_checker();
    let statuses = checker.get_all_statuses().await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(CheckerStatusResponse {
            success: true,
            engines: statuses,
        })
}

/// Get status of a specific engine
///
/// GET /api/checker/status/{engine_name}
#[get("/api/checker/status/{engine_name}")]
pub async fn get_engine_check_status(path: web::Path<String>) -> HttpResponse {
    let engine_name = path.into_inner();
    let checker = get_checker();

    match checker.get_status(&engine_name).await {
        Some(status) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(serde_json::json!({
                "success": true,
                "engine": status
            })),
        None => HttpResponse::NotFound()
            .content_type(ContentType::json())
            .json(serde_json::json!({
                "success": false,
                "error": format!("No check status found for engine '{}'", engine_name)
            })),
    }
}

/// Check a single engine
///
/// POST /api/checker/check/{engine_name}
#[post("/api/checker/check/{engine_name}")]
pub async fn check_single_engine(path: web::Path<String>) -> HttpResponse {
    let engine_name = path.into_inner();
    let checker = get_checker();

    let result = checker.check_engine(&engine_name).await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(CheckEngineResponse {
            success: result.success,
            engine: engine_name,
            result,
        })
}

/// Check multiple engines
///
/// POST /api/checker/check
#[post("/api/checker/check")]
pub async fn check_multiple_engines(body: web::Json<CheckEnginesRequest>) -> HttpResponse {
    let checker = get_checker();
    let engine_names: Vec<&str> = body.engines.iter().map(|s| s.as_str()).collect();

    let results = checker.check_engines(&engine_names).await;

    let healthy_count = results.values().filter(|r| r.success).count();
    let unhealthy_count = results.len() - healthy_count;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(CheckEnginesResponse {
            success: true,
            results,
            healthy_count,
            unhealthy_count,
        })
}

/// Get list of healthy engines
///
/// GET /api/checker/healthy
#[get("/api/checker/healthy")]
pub async fn get_healthy_engines() -> HttpResponse {
    let checker = get_checker();
    let healthy = checker.get_healthy_engines().await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(serde_json::json!({
            "success": true,
            "healthy_engines": healthy,
            "count": healthy.len()
        }))
}

/// Get list of unhealthy engines
///
/// GET /api/checker/unhealthy
#[get("/api/checker/unhealthy")]
pub async fn get_unhealthy_engines() -> HttpResponse {
    let checker = get_checker();
    let unhealthy = checker.get_unhealthy_engines().await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(serde_json::json!({
            "success": true,
            "unhealthy_engines": unhealthy,
            "count": unhealthy.len()
        }))
}

/// Reset checker status for an engine
///
/// POST /api/checker/reset/{engine_name}
#[post("/api/checker/reset/{engine_name}")]
pub async fn reset_engine_check_status(path: web::Path<String>) -> HttpResponse {
    let engine_name = path.into_inner();
    let checker = get_checker();

    checker.reset_status(&engine_name).await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(ApiResponse {
            success: true,
            message: format!("Check status for engine '{}' has been reset", engine_name),
        })
}

/// Reset all checker statuses
///
/// POST /api/checker/reset
#[post("/api/checker/reset")]
pub async fn reset_all_check_statuses() -> HttpResponse {
    let checker = get_checker();
    checker.reset_all().await;

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(ApiResponse {
            success: true,
            message: "All check statuses have been reset".to_string(),
        })
}
