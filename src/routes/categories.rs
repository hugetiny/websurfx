//! This module provides API routes for engine categories functionality.

use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::models::engine_categories::{
    get_categories_info, get_default_engines, get_engine_info, get_engines_by_category,
    get_engines_by_region, get_engine_registry, parse_bang, BangResult, EngineCategory,
};

/// Response for category list endpoint
#[derive(Debug, Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryData>,
    pub total_engines: usize,
}

/// Individual category data
#[derive(Debug, Serialize)]
pub struct CategoryData {
    pub id: String,
    pub name: String,
    pub bang: String,
    pub engine_count: usize,
}

/// Response for engines list endpoint
#[derive(Debug, Serialize)]
pub struct EnginesResponse {
    pub engines: Vec<EngineData>,
    pub total: usize,
}

/// Individual engine data
#[derive(Debug, Serialize)]
pub struct EngineData {
    pub name: String,
    pub display_name: String,
    pub categories: Vec<String>,
    pub subcategory: String,
    pub shortcut: String,
    pub default_enabled: bool,
    pub region: String,
    pub description: String,
}

/// Query parameters for engines endpoint
#[derive(Debug, Deserialize)]
pub struct EnginesQuery {
    /// Filter by category
    pub category: Option<String>,
    /// Filter by region
    pub region: Option<String>,
    /// Filter by default enabled status
    pub default_only: Option<bool>,
}

/// Response for bang parse endpoint
#[derive(Debug, Serialize)]
pub struct BangParseResponse {
    pub input: String,
    #[serde(flatten)]
    pub result: BangParseResult,
}

/// Bang parse result
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum BangParseResult {
    #[serde(rename = "no_bang")]
    NoBang,
    #[serde(rename = "category")]
    Category { id: String, name: String, engines: Vec<String> },
    #[serde(rename = "engine")]
    Engine { name: String, display_name: String },
    #[serde(rename = "unknown")]
    Unknown { bang: String },
}

/// Get all categories with engine counts
#[get("/api/categories")]
pub async fn list_categories() -> HttpResponse {
    let categories_info = get_categories_info();

    let categories: Vec<CategoryData> = categories_info
        .iter()
        .map(|info| CategoryData {
            id: info.id.clone(),
            name: info.name.clone(),
            bang: info.bang.clone(),
            engine_count: info.engine_count,
        })
        .collect();

    let response = CategoriesResponse {
        total_engines: get_engine_registry().len(),
        categories,
    };

    HttpResponse::Ok().json(response)
}

/// Get detailed info for a specific category
#[get("/api/categories/{category_id}")]
pub async fn get_category(path: web::Path<String>) -> HttpResponse {
    let category_id = path.into_inner();

    let category = match EngineCategory::from_str(&category_id) {
        Some(cat) => cat,
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Category not found",
                "available_categories": EngineCategory::all()
                    .iter()
                    .map(|c| format!("{:?}", c).to_lowercase())
                    .collect::<Vec<_>>()
            }));
        }
    };

    let engines = get_engines_by_category(category);
    let engine_data: Vec<EngineData> = engines
        .iter()
        .map(|info| EngineData {
            name: info.name.to_string(),
            display_name: info.display_name.to_string(),
            categories: info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
            subcategory: format!("{:?}", info.subcategory).to_lowercase(),
            shortcut: info.shortcut.to_string(),
            default_enabled: info.default_enabled,
            region: info.region.to_string(),
            description: info.description.to_string(),
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "category": {
            "id": format!("{:?}", category).to_lowercase(),
            "name": category.display_name(),
            "bang": category.bang()
        },
        "engines": engine_data,
        "engine_count": engine_data.len()
    }))
}

/// Get all engines with optional filtering
#[get("/api/engines")]
pub async fn list_engines(query: web::Query<EnginesQuery>) -> HttpResponse {
    let mut engines: Vec<_> = get_engine_registry().values().collect();

    // Filter by category if specified
    if let Some(ref category_str) = query.category {
        if let Some(category) = EngineCategory::from_str(category_str) {
            engines.retain(|e| e.categories.contains(&category));
        }
    }

    // Filter by region if specified
    if let Some(ref region) = query.region {
        engines.retain(|e| e.region.eq_ignore_ascii_case(region) || e.region.is_empty());
    }

    // Filter by default enabled if specified
    if let Some(default_only) = query.default_only {
        if default_only {
            engines.retain(|e| e.default_enabled);
        }
    }

    let engine_data: Vec<EngineData> = engines
        .iter()
        .map(|info| EngineData {
            name: info.name.to_string(),
            display_name: info.display_name.to_string(),
            categories: info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
            subcategory: format!("{:?}", info.subcategory).to_lowercase(),
            shortcut: info.shortcut.to_string(),
            default_enabled: info.default_enabled,
            region: info.region.to_string(),
            description: info.description.to_string(),
        })
        .collect();

    let response = EnginesResponse {
        total: engine_data.len(),
        engines: engine_data,
    };

    HttpResponse::Ok().json(response)
}

/// Get info for a specific engine
#[get("/api/engines/{engine_name}")]
pub async fn get_engine(path: web::Path<String>) -> HttpResponse {
    let engine_name = path.into_inner();

    match get_engine_info(&engine_name) {
        Some(info) => {
            HttpResponse::Ok().json(EngineData {
                name: info.name.to_string(),
                display_name: info.display_name.to_string(),
                categories: info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
                subcategory: format!("{:?}", info.subcategory).to_lowercase(),
                shortcut: info.shortcut.to_string(),
                default_enabled: info.default_enabled,
                region: info.region.to_string(),
                description: info.description.to_string(),
            })
        }
        None => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Engine not found",
                "engine": engine_name
            }))
        }
    }
}

/// Get default enabled engines
#[get("/api/engines/defaults")]
pub async fn list_default_engines() -> HttpResponse {
    let defaults = get_default_engines();

    let engine_data: Vec<EngineData> = defaults
        .iter()
        .map(|info| EngineData {
            name: info.name.to_string(),
            display_name: info.display_name.to_string(),
            categories: info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
            subcategory: format!("{:?}", info.subcategory).to_lowercase(),
            shortcut: info.shortcut.to_string(),
            default_enabled: info.default_enabled,
            region: info.region.to_string(),
            description: info.description.to_string(),
        })
        .collect();

    let response = EnginesResponse {
        total: engine_data.len(),
        engines: engine_data,
    };

    HttpResponse::Ok().json(response)
}

/// Get engines for a specific region
#[get("/api/engines/region/{region}")]
pub async fn list_engines_by_region(path: web::Path<String>) -> HttpResponse {
    let region = path.into_inner();
    let engines = get_engines_by_region(&region);

    let engine_data: Vec<EngineData> = engines
        .iter()
        .map(|info| EngineData {
            name: info.name.to_string(),
            display_name: info.display_name.to_string(),
            categories: info.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect(),
            subcategory: format!("{:?}", info.subcategory).to_lowercase(),
            shortcut: info.shortcut.to_string(),
            default_enabled: info.default_enabled,
            region: info.region.to_string(),
            description: info.description.to_string(),
        })
        .collect();

    let response = EnginesResponse {
        total: engine_data.len(),
        engines: engine_data,
    };

    HttpResponse::Ok().json(response)
}

/// Parse bang query parameters
#[derive(Debug, Deserialize)]
pub struct BangQuery {
    /// The query string containing potential bang syntax
    pub q: String,
}

/// Parse bang syntax in a query
#[get("/api/bang/parse")]
pub async fn parse_bang_query(query: web::Query<BangQuery>) -> HttpResponse {
    let input = query.q.trim();

    // Find all bang patterns in the query
    let words: Vec<&str> = input.split_whitespace().collect();
    let bangs: Vec<&str> = words.iter().filter(|w| w.starts_with('!')).copied().collect();

    if bangs.is_empty() {
        return HttpResponse::Ok().json(BangParseResponse {
            input: input.to_string(),
            result: BangParseResult::NoBang,
        });
    }

    // Parse the first bang found
    let bang = bangs[0];
    let result = parse_bang(bang);

    let parse_result = match result {
        BangResult::NoBang => BangParseResult::NoBang,
        BangResult::Category(category) => {
            let engines = get_engines_by_category(category);
            BangParseResult::Category {
                id: format!("{:?}", category).to_lowercase(),
                name: category.display_name().to_string(),
                engines: engines.iter().map(|e| e.name.to_string()).collect(),
            }
        }
        BangResult::Engine(engine_name) => {
            if let Some(info) = get_engine_info(engine_name) {
                BangParseResult::Engine {
                    name: info.name.to_string(),
                    display_name: info.display_name.to_string(),
                }
            } else {
                BangParseResult::Unknown { bang: bang.to_string() }
            }
        }
        BangResult::Unknown(b) => BangParseResult::Unknown { bang: b },
    };

    HttpResponse::Ok().json(BangParseResponse {
        input: input.to_string(),
        result: parse_result,
    })
}

/// Get all available bangs (shortcuts)
#[get("/api/bang/list")]
pub async fn list_bangs() -> HttpResponse {
    let mut bangs: Vec<serde_json::Value> = Vec::new();

    // Add category bangs
    for category in EngineCategory::all() {
        bangs.push(serde_json::json!({
            "type": "category",
            "bang": category.bang(),
            "name": category.display_name(),
            "description": format!("Search in {} engines", category.display_name())
        }));
    }

    // Add engine bangs
    for info in get_engine_registry().values() {
        bangs.push(serde_json::json!({
            "type": "engine",
            "bang": info.shortcut,
            "name": info.display_name,
            "description": info.description
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "bangs": bangs,
        "total": bangs.len()
    }))
}

/// Configure routes for categories API
#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_categories)
        .service(list_engines)
        .service(list_default_engines)
        .service(parse_bang_query)
        .service(list_bangs)
        .service(get_category)
        .service(get_engine)
        .service(list_engines_by_region);
}
