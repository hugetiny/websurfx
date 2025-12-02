//! This module provides modules which in turn provides various models for aggregrating search
//! results, parsing config file, providing trait to standardize search engine handling code,
//! custom engine error for the search engine, engine health management, engine categorization, etc.

pub mod aggregation;
pub mod engine;
pub mod engine_categories;
pub mod engine_health;
pub mod engine_metrics;
pub mod infobox;
pub mod parser;
pub mod safesearch;
pub mod search_route;
pub mod searxng_format;
