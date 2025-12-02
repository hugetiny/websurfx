//! SearXNG-compatible API module
//!
//! This module provides API endpoints that are compatible with SearXNG's API format.
//! This allows clients designed for SearXNG to work with websurfx.
//!
//! Endpoints:
//! - GET/POST /api/v1/search - Main search endpoint
//! - GET /api/v1/config - Instance configuration
//! - GET /api/v1/categories - Available categories
//! - GET /api/v1/engines - Available engines
//! - GET /api/v1/autocomplete - Search suggestions

use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::models::engine_categories::{
    get_categories_info, get_engine_registry, get_engines_by_category, EngineCategory, EngineInfo,
};
use crate::models::engine_health::get_health_manager;

/// SearXNG-compatible search request parameters
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// The search query (required)
    pub q: String,

    /// Comma-separated list of categories
    pub categories: Option<String>,

    /// Comma-separated list of engines
    pub engines: Option<String>,

    /// Language code (e.g., "en", "zh-CN")
    pub language: Option<String>,

    /// Page number (default: 1)
    pub pageno: Option<u32>,

    /// Time range: day, month, year
    pub time_range: Option<String>,

    /// Output format: json, csv, rss
    pub format: Option<String>,

    /// Safe search level: 0, 1, 2
    pub safesearch: Option<u8>,

    /// Image proxy enabled
    pub image_proxy: Option<bool>,

    /// Autocomplete provider
    pub autocomplete: Option<String>,

    /// Theme
    pub theme: Option<String>,

    /// Enabled plugins (comma-separated)
    pub enabled_plugins: Option<String>,

    /// Disabled plugins (comma-separated)
    pub disabled_plugins: Option<String>,

    /// Enabled engines (comma-separated)
    pub enabled_engines: Option<String>,

    /// Disabled engines (comma-separated)
    pub disabled_engines: Option<String>,
}

/// SearXNG-compatible search result
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_url: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,
    pub score: f64,
    pub category: String,
}

/// SearXNG-compatible search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub number_of_results: u64,
    pub results: Vec<SearchResult>,
    pub answers: Vec<String>,
    pub corrections: Vec<String>,
    pub infoboxes: Vec<Infobox>,
    pub suggestions: Vec<String>,
    pub unresponsive_engines: Vec<UnresponsiveEngine>,
}

/// Infobox data
#[derive(Debug, Serialize)]
pub struct Infobox {
    pub infobox: String,
    pub id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,
    pub urls: Vec<InfoboxUrl>,
    pub attributes: Vec<InfoboxAttribute>,
    pub engine: String,
}

/// Infobox URL
#[derive(Debug, Serialize)]
pub struct InfoboxUrl {
    pub title: String,
    pub url: String,
}

/// Infobox attribute
#[derive(Debug, Serialize)]
pub struct InfoboxAttribute {
    pub label: String,
    pub value: String,
}

/// Unresponsive engine info
#[derive(Debug, Serialize)]
pub struct UnresponsiveEngine {
    pub engine: String,
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// SearXNG-compatible configuration response
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub version: String,
    pub brand: BrandConfig,
    pub default_theme: String,
    pub default_locale: String,
    pub categories: Vec<String>,
    pub categories_as_tabs: Vec<CategoryTab>,
    pub engines: Vec<EngineConfig>,
    pub plugins: Vec<PluginConfig>,
    pub instance_name: String,
    pub safe_search: u8,
    pub autocomplete: String,
    pub default_engine: String,
}

/// Brand configuration
#[derive(Debug, Serialize)]
pub struct BrandConfig {
    pub name: String,
    pub favicon: String,
    pub new_issue_url: String,
    pub docs_url: String,
    pub public_instances: String,
    pub git_url: String,
}

/// Category tab configuration
#[derive(Debug, Serialize)]
pub struct CategoryTab {
    pub name: String,
    pub display_name: String,
    pub engines: Vec<String>,
}

/// Engine configuration
#[derive(Debug, Serialize)]
pub struct EngineConfig {
    pub name: String,
    pub display_name: String,
    pub shortcut: String,
    pub categories: Vec<String>,
    pub enabled: bool,
    pub language_support: bool,
    pub time_range_support: bool,
    pub safesearch: bool,
    pub paging: bool,
}

/// Plugin configuration
#[derive(Debug, Serialize)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: bool,
    pub description: String,
}

/// Autocomplete request
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AutocompleteRequest {
    pub q: String,
    #[serde(default)]
    pub provider: Option<String>,
}

/// Autocomplete response
#[derive(Debug, Serialize)]
pub struct AutocompleteResponse {
    pub query: String,
    pub suggestions: Vec<String>,
}

// ============================================================================
// API Endpoints
// ============================================================================

/// GET /api/v1/search - SearXNG-compatible search endpoint
#[get("/api/v1/search")]
pub async fn search_get(query: web::Query<SearchRequest>) -> HttpResponse {
    handle_search(query.into_inner()).await
}

/// POST /api/v1/search - SearXNG-compatible search endpoint
#[post("/api/v1/search")]
pub async fn search_post(body: web::Json<SearchRequest>) -> HttpResponse {
    handle_search(body.into_inner()).await
}

/// Handle search request
async fn handle_search(request: SearchRequest) -> HttpResponse {
    // Parse categories
    let categories: Vec<EngineCategory> = request
        .categories
        .as_ref()
        .map(|cats| {
            cats.split(',')
                .filter_map(|c| EngineCategory::from_str(c.trim()))
                .collect()
        })
        .unwrap_or_else(|| vec![EngineCategory::General]);

    // Parse engines filter
    let engine_filter: Option<Vec<String>> = request
        .engines
        .as_ref()
        .map(|e| e.split(',').map(|s| s.trim().to_lowercase()).collect());

    // Get enabled engines based on categories and filters
    let _registry = get_engine_registry();
    let mut enabled_engines: Vec<&EngineInfo> = Vec::new();

    for category in &categories {
        let cat_engines = get_engines_by_category(*category);
        for engine in cat_engines {
            // Check if engine matches filter
            let include = match &engine_filter {
                Some(filter) => filter.contains(&engine.name.to_string()),
                None => true,
            };

            // Check disabled engines
            let disabled = request
                .disabled_engines
                .as_ref()
                .map(|d| d.split(',').any(|e| e.trim().eq_ignore_ascii_case(engine.name)))
                .unwrap_or(false);

            if include && !disabled && !enabled_engines.iter().any(|e| e.name == engine.name) {
                enabled_engines.push(engine);
            }
        }
    }

    // Build response (placeholder - actual search would go through aggregator)
    let response = SearchResponse {
        query: request.q.clone(),
        number_of_results: 0,
        results: vec![],
        answers: vec![],
        corrections: vec![],
        infoboxes: vec![],
        suggestions: vec![],
        unresponsive_engines: vec![],
    };

    // Return based on format
    match request.format.as_deref() {
        Some("json") | None => HttpResponse::Ok().json(response),
        Some("csv") => {
            // CSV format
            let mut csv = String::from("title,url,content,engine,category\n");
            for result in &response.results {
                csv.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                    result.title.replace('"', "\"\""),
                    result.url,
                    result.content.replace('"', "\"\""),
                    result.engine.as_deref().unwrap_or(""),
                    result.category
                ));
            }
            HttpResponse::Ok()
                .content_type("text/csv")
                .body(csv)
        }
        Some("rss") => {
            // RSS format
            let rss = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Websurfx Search: {}</title>
    <link>http://localhost:8080/search?q={}</link>
    <description>Search results for: {}</description>
    {}
  </channel>
</rss>"#,
                request.q,
                urlencoding::encode(&request.q),
                request.q,
                response.results.iter().map(|r| format!(
                    "<item><title>{}</title><link>{}</link><description>{}</description></item>",
                    html_escape(&r.title),
                    r.url,
                    html_escape(&r.content)
                )).collect::<Vec<_>>().join("\n    ")
            );
            HttpResponse::Ok()
                .content_type("application/rss+xml")
                .body(rss)
        }
        _ => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Unsupported format",
            "supported_formats": ["json", "csv", "rss"]
        })),
    }
}

/// GET /api/v1/config - Get instance configuration
#[get("/api/v1/config")]
pub async fn get_config() -> HttpResponse {
    let registry = get_engine_registry();
    let _categories_info = get_categories_info();

    // Build engines config
    let engines: Vec<EngineConfig> = registry
        .values()
        .map(|info| EngineConfig {
            name: info.name.to_string(),
            display_name: info.display_name.to_string(),
            shortcut: info.shortcut.to_string(),
            categories: info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
            enabled: info.default_enabled,
            language_support: true,
            time_range_support: false,
            safesearch: true,
            paging: true,
        })
        .collect();

    // Build category tabs
    let categories_as_tabs: Vec<CategoryTab> = EngineCategory::all()
        .iter()
        .map(|cat| {
            let cat_engines = get_engines_by_category(*cat);
            CategoryTab {
                name: format!("{:?}", cat).to_lowercase(),
                display_name: cat.display_name().to_string(),
                engines: cat_engines.iter().map(|e| e.name.to_string()).collect(),
            }
        })
        .collect();

    let config = ConfigResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        brand: BrandConfig {
            name: "Websurfx".to_string(),
            favicon: "/favicon.ico".to_string(),
            new_issue_url: "https://github.com/neon-mmd/websurfx/issues".to_string(),
            docs_url: "https://github.com/neon-mmd/websurfx".to_string(),
            public_instances: "https://github.com/neon-mmd/websurfx".to_string(),
            git_url: "https://github.com/neon-mmd/websurfx".to_string(),
        },
        default_theme: "simple".to_string(),
        default_locale: "en".to_string(),
        categories: EngineCategory::all()
            .iter()
            .map(|c| format!("{:?}", c).to_lowercase())
            .collect(),
        categories_as_tabs,
        engines,
        plugins: vec![
            PluginConfig {
                name: "Hash_plugin".to_string(),
                enabled: true,
                description: "Converts hash queries".to_string(),
            },
            PluginConfig {
                name: "Self_Information".to_string(),
                enabled: true,
                description: "Shows your IP address".to_string(),
            },
            PluginConfig {
                name: "Tracker_URL_remover".to_string(),
                enabled: true,
                description: "Removes tracking parameters".to_string(),
            },
        ],
        instance_name: "Websurfx".to_string(),
        safe_search: 2,
        autocomplete: "duckduckgo".to_string(),
        default_engine: "duckduckgo".to_string(),
    };

    HttpResponse::Ok().json(config)
}

/// GET /api/v1/categories - Get all categories (SearXNG format)
#[get("/api/v1/categories")]
pub async fn list_categories_v1() -> HttpResponse {
    let categories: Vec<serde_json::Value> = EngineCategory::all()
        .iter()
        .map(|cat| {
            let engines = get_engines_by_category(*cat);
            serde_json::json!({
                "name": format!("{:?}", cat).to_lowercase(),
                "display_name": cat.display_name(),
                "bang": cat.bang(),
                "engines": engines.iter().map(|e| serde_json::json!({
                    "name": e.name,
                    "display_name": e.display_name,
                    "shortcut": e.shortcut,
                    "enabled": e.default_enabled
                })).collect::<Vec<_>>(),
                "engine_count": engines.len()
            })
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "categories": categories,
        "total": categories.len()
    }))
}

/// GET /api/v1/engines - Get all engines (SearXNG format)
#[get("/api/v1/engines")]
pub async fn list_engines_v1(query: web::Query<EnginesQueryV1>) -> HttpResponse {
    let registry = get_engine_registry();
    let mut engines: Vec<&EngineInfo> = registry.values().collect();

    // Filter by category
    if let Some(ref cat_str) = query.category {
        if let Some(category) = EngineCategory::from_str(cat_str) {
            engines.retain(|e| e.categories.contains(&category));
        }
    }

    // Filter by enabled only
    if query.enabled_only.unwrap_or(false) {
        engines.retain(|e| e.default_enabled);
    }

    let result: Vec<serde_json::Value> = engines
        .iter()
        .map(|e| serde_json::json!({
            "name": e.name,
            "display_name": e.display_name,
            "shortcut": e.shortcut,
            "categories": e.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect::<Vec<_>>(),
            "subcategory": format!("{:?}", e.subcategory).to_lowercase(),
            "enabled": e.default_enabled,
            "region": e.region,
            "description": e.description,
            "language_support": true,
            "time_range_support": false,
            "safesearch": true,
            "paging": true
        }))
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "engines": result,
        "total": result.len()
    }))
}

/// Query parameters for engines v1 endpoint
#[derive(Debug, Deserialize)]
pub struct EnginesQueryV1 {
    pub category: Option<String>,
    pub enabled_only: Option<bool>,
}

/// GET /api/v1/autocomplete - Autocomplete suggestions
#[get("/api/v1/autocomplete")]
pub async fn autocomplete(query: web::Query<AutocompleteRequest>) -> HttpResponse {
    // Placeholder - would need to implement actual autocomplete
    // Could use DuckDuckGo, Google, or other providers
    let response = AutocompleteResponse {
        query: query.q.clone(),
        suggestions: vec![],
    };

    HttpResponse::Ok().json(response)
}

/// GET /api/v1/preferences - Get search preferences
#[get("/api/v1/preferences")]
pub async fn get_preferences() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "categories": EngineCategory::all()
            .iter()
            .map(|c| format!("{:?}", c).to_lowercase())
            .collect::<Vec<_>>(),
        "engines": get_engine_registry()
            .values()
            .map(|e| serde_json::json!({
                "name": e.name,
                "enabled": e.default_enabled
            }))
            .collect::<Vec<serde_json::Value>>(),
        "safe_search": 2,
        "language": "all",
        "theme": "simple",
        "autocomplete": "duckduckgo",
        "image_proxy": false,
        "method": "GET",
        "doi_resolver": "oadoi.org",
        "tokens": []
    }))
}

/// GET /api/v1/languages - Get supported languages
#[get("/api/v1/languages")]
pub async fn get_languages() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "languages": [
            {"code": "all", "name": "All Languages"},
            {"code": "en", "name": "English"},
            {"code": "zh", "name": "中文"},
            {"code": "zh-CN", "name": "简体中文"},
            {"code": "zh-TW", "name": "繁體中文"},
            {"code": "ja", "name": "日本語"},
            {"code": "ko", "name": "한국어"},
            {"code": "de", "name": "Deutsch"},
            {"code": "fr", "name": "Français"},
            {"code": "es", "name": "Español"},
            {"code": "it", "name": "Italiano"},
            {"code": "pt", "name": "Português"},
            {"code": "ru", "name": "Русский"},
            {"code": "ar", "name": "العربية"},
            {"code": "hi", "name": "हिन्दी"}
        ]
    }))
}

/// GET /api/v1/regions - Get supported regions
#[get("/api/v1/regions")]
pub async fn get_regions() -> HttpResponse {
    // Get unique regions from engines
    let registry = get_engine_registry();
    let mut regions: Vec<&str> = registry
        .values()
        .filter(|e| !e.region.is_empty())
        .map(|e| e.region)
        .collect();
    regions.sort();
    regions.dedup();

    let region_info: Vec<serde_json::Value> = regions
        .iter()
        .map(|r| {
            let name = match *r {
                "CN" => "中国",
                "US" => "United States",
                "UK" => "United Kingdom",
                "EU" => "European Union",
                "DE" => "Deutschland",
                "FR" => "France",
                "JP" => "日本",
                "KR" => "대한민국",
                "RU" => "Россия",
                "IT" => "Italia",
                "CZ" => "Česká republika",
                _ => r,
            };
            serde_json::json!({
                "code": r,
                "name": name
            })
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "regions": region_info
    }))
}

// ============================================================================
// Additional SearXNG-compatible endpoints
// ============================================================================

/// GET /api/v1/engines/stats - Get engine statistics (SearXNG style)
#[get("/api/v1/engines/stats")]
pub async fn get_engine_stats() -> HttpResponse {
    let health_manager = get_health_manager();
    let all_stats = health_manager.get_all_stats();
    let registry = get_engine_registry();

    let engine_stats: Vec<serde_json::Value> = registry
        .values()
        .map(|info| {
            let metrics = all_stats.engines.get(info.name);
            let is_suspended = health_manager.is_suspended(info.name);
            let suspension_info = health_manager.get_suspension_info(info.name);

            serde_json::json!({
                "name": info.name,
                "display_name": info.display_name,
                "categories": info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect::<Vec<_>>(),
                "enabled": info.default_enabled && !is_suspended,
                "suspended": is_suspended,
                "suspension_end_time": suspension_info.as_ref().map(|(secs, _)| *secs),
                "suspend_reason": suspension_info.as_ref().map(|(_, reason)| reason.clone()),
                "stats": metrics.map(|m| serde_json::json!({
                    "total_requests": m.total_requests,
                    "successful_requests": m.successful_requests,
                    "total_results": m.total_results,
                    "avg_response_time_ms": if m.successful_requests > 0 {
                        m.total_response_time_ms / m.successful_requests
                    } else { 0 },
                    "reliability": if m.total_requests > 0 {
                        (m.successful_requests as f64 / m.total_requests as f64) * 100.0
                    } else { 100.0 },
                    "error_counts": m.error_counts,
                }))
            })
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "engines": engine_stats,
        "total": engine_stats.len(),
        "generated_at": all_stats.generated_at
    }))
}

/// GET /api/v1/engines/{engine}/toggle - Toggle engine enabled/disabled
#[post("/api/v1/engines/{engine}/toggle")]
pub async fn toggle_engine(path: web::Path<String>) -> HttpResponse {
    let engine_name = path.into_inner();
    let health_manager = get_health_manager();

    if health_manager.is_suspended(&engine_name) {
        // Resume the engine
        health_manager.resume_engine(&engine_name);
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "engine": engine_name,
            "action": "resumed",
            "message": format!("Engine '{}' has been resumed", engine_name)
        }))
    } else {
        // For now, just report current status (suspension would need config changes)
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "engine": engine_name,
            "status": "active",
            "message": format!("Engine '{}' is currently active", engine_name)
        }))
    }
}

/// GET /api/v1/time_range - Get supported time ranges
#[get("/api/v1/time_range")]
pub async fn get_time_ranges() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "time_ranges": [
            {"value": "", "name": "Any time"},
            {"value": "day", "name": "Last day"},
            {"value": "week", "name": "Last week"},
            {"value": "month", "name": "Last month"},
            {"value": "year", "name": "Last year"}
        ]
    }))
}

/// GET /api/v1/safesearch - Get safe search options
#[get("/api/v1/safesearch")]
pub async fn get_safesearch_options() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "options": [
            {"value": 0, "name": "None", "description": "No safe search filtering"},
            {"value": 1, "name": "Moderate", "description": "Filter explicit content"},
            {"value": 2, "name": "Strict", "description": "Strict filtering of explicit content"}
        ],
        "default": 2
    }))
}

/// GET /api/v1/formats - Get supported output formats
#[get("/api/v1/formats")]
pub async fn get_formats() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "formats": [
            {"value": "json", "name": "JSON", "content_type": "application/json"},
            {"value": "csv", "name": "CSV", "content_type": "text/csv"},
            {"value": "rss", "name": "RSS", "content_type": "application/rss+xml"}
        ],
        "default": "json"
    }))
}

/// GET /api/v1/plugins - Get available plugins (SearXNG compatibility)
#[get("/api/v1/plugins")]
pub async fn get_plugins() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "plugins": [
            {
                "name": "Hash_plugin",
                "enabled": true,
                "description": "Converts hash queries to direct results",
                "default_on": true
            },
            {
                "name": "Self_Information",
                "enabled": true,
                "description": "Shows your IP address and user agent",
                "default_on": true
            },
            {
                "name": "Tracker_URL_remover",
                "enabled": true,
                "description": "Removes tracking parameters from URLs",
                "default_on": true
            },
            {
                "name": "Ahmia_blacklist",
                "enabled": false,
                "description": "Filters results using Ahmia blacklist",
                "default_on": false
            }
        ]
    }))
}

/// GET /api/v1/shortcuts - Get all bang shortcuts
#[get("/api/v1/shortcuts")]
pub async fn get_shortcuts() -> HttpResponse {
    let registry = get_engine_registry();

    let mut shortcuts: Vec<serde_json::Value> = Vec::new();

    // Add category bangs
    for category in EngineCategory::all() {
        shortcuts.push(serde_json::json!({
            "bang": category.bang(),
            "name": category.display_name(),
            "type": "category",
            "description": format!("Search in {} category", category.display_name())
        }));
    }

    // Add engine bangs
    for info in registry.values() {
        shortcuts.push(serde_json::json!({
            "bang": info.shortcut,
            "name": info.display_name,
            "type": "engine",
            "categories": info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect::<Vec<_>>(),
            "description": info.description
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "shortcuts": shortcuts,
        "total": shortcuts.len()
    }))
}

/// GET /api/v1/info - Get instance information (SearXNG style)
#[get("/api/v1/info")]
pub async fn get_info() -> HttpResponse {
    let registry = get_engine_registry();
    let health_manager = get_health_manager();
    let suspended = health_manager.get_suspended_engines();

    let total_engines = registry.len();
    let active_engines = total_engines - suspended.len();

    HttpResponse::Ok().json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "Websurfx",
        "description": "SearXNG-compatible meta search engine",
        "engines": {
            "total": total_engines,
            "active": active_engines,
            "suspended": suspended.len()
        },
        "categories": EngineCategory::all().len(),
        "features": {
            "safe_search": true,
            "autocomplete": true,
            "bang_syntax": true,
            "image_proxy": false,
            "categories": true,
            "engines_health": true,
            "api_v1": true
        },
        "endpoints": {
            "search": "/api/v1/search",
            "config": "/api/v1/config",
            "categories": "/api/v1/categories",
            "engines": "/api/v1/engines",
            "autocomplete": "/api/v1/autocomplete",
            "preferences": "/api/v1/preferences"
        }
    }))
}

/// Helper function to escape HTML
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Configure routes for SearXNG-compatible API
#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(search_get)
        .service(search_post)
        .service(get_config)
        .service(list_categories_v1)
        .service(list_engines_v1)
        .service(autocomplete)
        .service(get_preferences)
        .service(get_languages)
        .service(get_regions)
        .service(get_engine_stats)
        .service(toggle_engine)
        .service(get_time_ranges)
        .service(get_safesearch_options)
        .service(get_formats)
        .service(get_plugins)
        .service(get_shortcuts)
        .service(get_info);
}
