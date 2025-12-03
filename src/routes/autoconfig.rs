//! Auto-configuration API routes.
//!
//! These routes provide automatic engine configuration management.

use crate::config_manager::{get_config_manager, run_auto_config, EngineConfig};
use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

/// Response for auto-config status
#[derive(Serialize)]
struct AutoConfigResponse {
    success: bool,
    message: String,
    enabled_count: usize,
    disabled_count: usize,
    engines: Vec<EngineConfig>,
}

/// Request to run auto-config
#[derive(Deserialize)]
pub struct RunAutoConfigRequest {
    /// List of engine names to check (optional, uses all if not specified)
    engines: Option<Vec<String>>,
    /// Whether to save results
    save: Option<bool>,
}

/// Request to update settings
#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    min_reliability: Option<f64>,
    max_response_time_ms: Option<u64>,
    disable_after_failures: Option<u32>,
    enable_after_successes: Option<u32>,
    max_concurrent_engines: Option<usize>,
    auto_enable_new: Option<bool>,
    persist_changes: Option<bool>,
}

/// Get current auto-config status
#[get("/api/autoconfig/status")]
pub async fn get_status() -> HttpResponse {
    let manager = get_config_manager();
    let enabled = manager.get_enabled_engines().await;
    let disabled = manager.get_disabled_engines().await;
    let all = manager.get_all_configs().await;

    HttpResponse::Ok().json(AutoConfigResponse {
        success: true,
        message: "Current configuration status".to_string(),
        enabled_count: enabled.len(),
        disabled_count: disabled.len(),
        engines: all,
    })
}

/// Get current settings
#[get("/api/autoconfig/settings")]
pub async fn get_settings() -> HttpResponse {
    let manager = get_config_manager();
    let settings = manager.get_settings().await;
    HttpResponse::Ok().json(settings)
}

/// Update auto-config settings
#[post("/api/autoconfig/settings")]
pub async fn update_settings(body: web::Json<UpdateSettingsRequest>) -> HttpResponse {
    let manager = get_config_manager();
    let mut settings = manager.get_settings().await;

    if let Some(v) = body.min_reliability {
        settings.min_reliability = v;
    }
    if let Some(v) = body.max_response_time_ms {
        settings.max_response_time_ms = v;
    }
    if let Some(v) = body.disable_after_failures {
        settings.disable_after_failures = v;
    }
    if let Some(v) = body.enable_after_successes {
        settings.enable_after_successes = v;
    }
    if let Some(v) = body.max_concurrent_engines {
        settings.max_concurrent_engines = v;
    }
    if let Some(v) = body.auto_enable_new {
        settings.auto_enable_new = v;
    }
    if let Some(v) = body.persist_changes {
        settings.persist_changes = v;
    }

    manager.update_settings(settings.clone()).await;

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Settings updated",
        "settings": settings
    }))
}

/// Run auto-configuration check
#[post("/api/autoconfig/run")]
pub async fn run_config(body: web::Json<RunAutoConfigRequest>) -> HttpResponse {
    // Default engine list if not specified
    let default_engines = vec![
        "baidu",
        "bing",
        "duckduckgo",
        "google",
        "sogou",
        "search360",
        "wikipedia",
        "github",
        "arxiv",
        "hackernews",
    ];

    let engines: Vec<&str> = body
        .engines
        .as_ref()
        .map(|e| e.iter().map(|s| s.as_str()).collect())
        .unwrap_or(default_engines);

    let results = run_auto_config(&engines).await;

    let manager = get_config_manager();
    let enabled = manager.get_enabled_engines().await;
    let disabled = manager.get_disabled_engines().await;

    // Save if requested
    if body.save.unwrap_or(false) {
        if let Err(e) = manager.save_to_json("engine_config.json").await {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to save: {}", e)
            }));
        }
    }

    HttpResponse::Ok().json(AutoConfigResponse {
        success: true,
        message: format!("Checked {} engines", results.len()),
        enabled_count: enabled.len(),
        disabled_count: disabled.len(),
        engines: results.into_values().collect(),
    })
}

/// Get enabled engines list
#[get("/api/autoconfig/enabled")]
pub async fn get_enabled() -> HttpResponse {
    let manager = get_config_manager();
    let enabled = manager.get_enabled_engines().await;

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "count": enabled.len(),
        "engines": enabled
    }))
}

/// Get disabled engines list
#[get("/api/autoconfig/disabled")]
pub async fn get_disabled() -> HttpResponse {
    let manager = get_config_manager();
    let disabled = manager.get_disabled_engines().await;

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "count": disabled.len(),
        "engines": disabled
    }))
}

/// Manually enable an engine
#[post("/api/autoconfig/enable/{engine}")]
pub async fn enable_engine(path: web::Path<String>) -> HttpResponse {
    let engine = path.into_inner();
    let manager = get_config_manager();

    match manager.enable_engine(&engine).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Engine '{}' enabled", engine)
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Manually disable an engine
#[post("/api/autoconfig/disable/{engine}")]
pub async fn disable_engine(path: web::Path<String>) -> HttpResponse {
    let engine = path.into_inner();
    let manager = get_config_manager();

    match manager.disable_engine(&engine).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Engine '{}' disabled", engine)
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Generate Lua config
#[get("/api/autoconfig/lua")]
pub async fn get_lua_config() -> HttpResponse {
    let manager = get_config_manager();
    let lua = manager.generate_lua_config().await;

    HttpResponse::Ok()
        .content_type("text/plain")
        .body(lua)
}

/// Save configuration to file
#[post("/api/autoconfig/save")]
pub async fn save_config() -> HttpResponse {
    let manager = get_config_manager();

    match manager.save_to_json("engine_config.json").await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Configuration saved to engine_config.json"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Load configuration from file
#[post("/api/autoconfig/load")]
pub async fn load_config() -> HttpResponse {
    let manager = get_config_manager();

    match manager.load_from_json("engine_config.json").await {
        Ok(_) => {
            let all = manager.get_all_configs().await;
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Configuration loaded",
                "engines": all
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}
