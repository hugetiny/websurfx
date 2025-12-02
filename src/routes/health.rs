//! This module provides the health check endpoint for monitoring engine status.
//!
//! This is a compatibility layer that maps to the stats API.

use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::engine_health::get_health_manager;

/// A structure to hold engine health statistics for JSON response
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EngineHealthResponse {
    pub total_requests: u64,
    pub total_failures: u64,
    pub consecutive_failures: u32,
    pub failure_rate: f64,
    pub is_disabled: bool,
    pub status: String,
}

/// Handles the route of health check page of the `websurfx` meta search engine website.
///
/// GET /health
#[get("/health")]
pub async fn health() -> HttpResponse {
    let health_manager = get_health_manager();
    let all_stats = health_manager.get_all_stats();

    let response: HashMap<String, EngineHealthResponse> = all_stats.engines
        .into_iter()
        .map(|(name, metrics)| {
            let failure_rate = if metrics.total_requests > 0 {
                (metrics.total_requests - metrics.successful_requests) as f64 / metrics.total_requests as f64
            } else {
                0.0
            };

            let status = if metrics.is_suspended {
                "disabled".to_string()
            } else if failure_rate > 0.5 {
                "degraded".to_string()
            } else {
                "healthy".to_string()
            };

            (
                name,
                EngineHealthResponse {
                    total_requests: metrics.total_requests,
                    total_failures: metrics.total_requests.saturating_sub(metrics.successful_requests),
                    consecutive_failures: 0, // Not tracked in the same way
                    failure_rate,
                    is_disabled: metrics.is_suspended,
                    status,
                },
            )
        })
        .collect();

    HttpResponse::Ok().json(response)
}

#[derive(Deserialize)]
pub struct ResetEngineRequest {
    engine: String,
}

/// POST /health/reset - Reset health data for a specific engine
#[post("/health/reset")]
pub async fn reset_engine(body: web::Json<ResetEngineRequest>) -> HttpResponse {
    let health_manager = get_health_manager();
    health_manager.resume_engine(&body.engine);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Reset health data for engine: {}", body.engine)
    }))
}

/// POST /health/reset-all - Reset health data for all engines
#[post("/health/reset-all")]
pub async fn reset_all() -> HttpResponse {
    let health_manager = get_health_manager();
    health_manager.reset_stats();

    // Also resume all suspended engines
    let suspended = health_manager.get_suspended_engines();
    for (name, _, _) in suspended {
        health_manager.resume_engine(&name);
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Reset health data for all engines"
    }))
}
