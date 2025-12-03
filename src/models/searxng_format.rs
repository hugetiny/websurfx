//! This module provides SearXNG-compatible response formatting.
//! It converts internal search results to SearXNG API format.

#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::models::aggregation::SearchResults;
use crate::models::engine_categories::get_engine_info;
use crate::models::infobox::Infobox as InternalInfobox;
use reqwest::Url;
use serde::Serialize;

/// SearXNG-compatible search result
#[derive(Debug, Clone, Serialize)]
pub struct SearxngResult {
    /// The URL of the result
    pub url: String,

    /// The title of the result
    pub title: String,

    /// The description/content of the result
    pub content: String,

    /// Engine that returned this result (for single engine)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,

    /// All engines that returned this result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<Vec<String>>,

    /// Positions in each engine's results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<u32>>,

    /// Parsed URL components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_url: Option<Vec<String>>,

    /// Template name for special result types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Thumbnail URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,

    /// Full-size image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,

    /// Image format (for image results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_format: Option<String>,

    /// Published date
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,

    /// Pretty URL for display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretty_url: Option<String>,

    /// Magnetlink (for torrent results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnetlink: Option<String>,

    /// Torrent info hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrentfile: Option<String>,

    /// Seed count (for torrents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,

    /// Leech count (for torrents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leech: Option<u32>,

    /// File size (for files/torrents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesize: Option<u64>,

    /// Author name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Iframe source (for embedded content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iframe_src: Option<String>,

    /// Aggregated score
    pub score: f64,

    /// Category of the result
    pub category: String,

    /// Whether the result is open access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access: Option<bool>,

    /// DOI (for academic results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    /// Comments URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
}

/// SearXNG-compatible infobox
#[derive(Debug, Clone, Serialize)]
pub struct SearxngInfobox {
    /// The title/name of the infobox
    pub infobox: String,

    /// Unique identifier
    pub id: String,

    /// Short summary content
    pub content: String,

    /// Image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,

    /// Related URLs
    pub urls: Vec<SearxngInfoboxUrl>,

    /// Key-value attributes
    pub attributes: Vec<SearxngInfoboxAttribute>,

    /// Source engine
    pub engine: String,

    /// Related topics
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relatedTopics: Vec<SearxngRelatedTopic>,
}

/// Infobox URL
#[derive(Debug, Clone, Serialize)]
pub struct SearxngInfoboxUrl {
    /// Title of the URL
    pub title: String,
    /// The URL itself
    pub url: String,
    /// Whether this is an official URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official: Option<bool>,
}

/// Infobox attribute
#[derive(Debug, Clone, Serialize)]
pub struct SearxngInfoboxAttribute {
    /// Label/name of the attribute
    pub label: String,
    /// Value of the attribute
    pub value: String,
    /// Entity type if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
}

/// Related topic
#[derive(Debug, Clone, Serialize)]
pub struct SearxngRelatedTopic {
    /// Name of the related topic
    pub name: String,
    /// URL to the related topic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Unresponsive engine info
#[derive(Debug, Clone, Serialize)]
pub struct SearxngUnresponsiveEngine {
    /// Engine name
    pub engine: String,
    /// Type of error that occurred
    pub error_type: String,
    /// Error message details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether the engine is currently suspended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended: Option<bool>,
}

/// Timing information for each engine
#[derive(Debug, Clone, Serialize)]
pub struct SearxngEngineTiming {
    /// Engine name
    pub engine: String,
    /// Time taken in seconds
    pub time: f64,
    /// Number of results from this engine
    pub count: u32,
}

/// SearXNG-compatible full search response
#[derive(Debug, Clone, Serialize)]
pub struct SearxngResponse {
    /// The query string
    pub query: String,

    /// Number of results
    pub number_of_results: u64,

    /// Search results
    pub results: Vec<SearxngResult>,

    /// Direct answers
    pub answers: Vec<String>,

    /// Spelling corrections
    pub corrections: Vec<String>,

    /// Infoboxes
    pub infoboxes: Vec<SearxngInfobox>,

    /// Search suggestions
    pub suggestions: Vec<String>,

    /// Unresponsive engines
    pub unresponsive_engines: Vec<SearxngUnresponsiveEngine>,

    /// Timing per engine
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub timings: Vec<SearxngEngineTiming>,

    /// Search time in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_time: Option<f64>,
}

impl SearxngResponse {
    /// Create a new empty response
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            number_of_results: 0,
            results: Vec::new(),
            answers: Vec::new(),
            corrections: Vec::new(),
            infoboxes: Vec::new(),
            suggestions: Vec::new(),
            unresponsive_engines: Vec::new(),
            timings: Vec::new(),
            search_time: None,
        }
    }
}

/// Convert internal search results to SearXNG format
pub fn convert_to_searxng_format(
    query: &str,
    results: &SearchResults,
    category: Option<&str>,
) -> SearxngResponse {
    let default_category = category.unwrap_or("general").to_string();

    let searxng_results: Vec<SearxngResult> = results
        .results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let engines = result.engine.clone();
            let primary_engine = engines.first().cloned();

            // Determine category from engine if available
            let result_category = primary_engine
                .as_ref()
                .and_then(|e| get_engine_info(e))
                .and_then(|info| info.categories.first())
                .map(|cat| cat.display_name().to_string())
                .unwrap_or_else(|| default_category.clone());

            // Parse URL for display
            let parsed_url = parse_url_components(&result.url);
            let pretty_url = create_pretty_url(&result.url);

            SearxngResult {
                url: result.url.clone(),
                title: result.title.clone(),
                content: result.description.clone(),
                engine: primary_engine,
                engines: if engines.len() > 1 { Some(engines) } else { None },
                positions: Some(vec![idx as u32 + 1]),
                parsed_url: Some(parsed_url),
                template: Some("default".to_string()),
                thumbnail: None,
                img_src: None,
                img_format: None,
                published_date: None,
                pretty_url: Some(pretty_url),
                magnetlink: None,
                torrentfile: None,
                seed: None,
                leech: None,
                filesize: None,
                author: None,
                iframe_src: None,
                score: result.relevance_score as f64,
                category: result_category,
                open_access: None,
                doi: None,
                comments: None,
            }
        })
        .collect();

    // Convert engine errors
    let unresponsive: Vec<SearxngUnresponsiveEngine> = results
        .engine_errors_info
        .iter()
        .map(|err| SearxngUnresponsiveEngine {
            engine: err.engine.clone(),
            error_type: err.error.clone(),
            message: Some(err.error.clone()),
            suspended: None,
        })
        .collect();

    SearxngResponse {
        query: query.to_string(),
        number_of_results: searxng_results.len() as u64,
        results: searxng_results,
        answers: Vec::new(),
        corrections: Vec::new(),
        infoboxes: Vec::new(),
        suggestions: Vec::new(),
        unresponsive_engines: unresponsive,
        timings: Vec::new(),
        search_time: None,
    }
}

/// Parse URL into components for display
fn parse_url_components(url_str: &str) -> Vec<String> {
    let mut components = Vec::new();

    if let Ok(parsed) = Url::parse(url_str) {
        components.push(parsed.scheme().to_string());
        if let Some(host) = parsed.host_str() {
            components.push(host.to_string());
        }
        components.push(parsed.path().to_string());
        if let Some(query) = parsed.query() {
            components.push(format!("?{}", query));
        }
    } else {
        components.push(url_str.to_string());
    }

    components
}

/// Create a pretty URL for display
fn create_pretty_url(url_str: &str) -> String {
    if let Ok(parsed) = Url::parse(url_str) {
        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();
        let truncated_path = if path.len() > 50 {
            format!("{}...", &path[..47])
        } else {
            path.to_string()
        };
        format!("{}{}", host, truncated_path)
    } else {
        if url_str.len() > 60 {
            format!("{}...", &url_str[..57])
        } else {
            url_str.to_string()
        }
    }
}

/// Convert internal infobox to SearXNG format
pub fn convert_infobox_to_searxng(infobox: &InternalInfobox) -> SearxngInfobox {
    SearxngInfobox {
        infobox: infobox.infobox.clone(),
        id: infobox.id.clone(),
        content: infobox.content.clone().unwrap_or_default(),
        img_src: infobox.img_src.clone(),
        urls: infobox.urls.iter().map(|u| SearxngInfoboxUrl {
            title: u.title.clone(),
            url: u.url.clone(),
            official: Some(u.official),
        }).collect(),
        attributes: infobox.attributes.iter().map(|a| SearxngInfoboxAttribute {
            label: a.label.clone(),
            value: a.value.clone(),
            entity: a.entity.clone(),
        }).collect(),
        engine: infobox.engine.clone(),
        relatedTopics: infobox.relatedTopics.iter().map(|t| SearxngRelatedTopic {
            name: t.name.clone(),
            url: t.url.clone(),
        }).collect(),
    }
}

/// Export format types supported by SearXNG
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// RSS format
    Rss,
}

impl ExportFormat {
    /// Parse format string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "csv" => Some(ExportFormat::Csv),
            "rss" => Some(ExportFormat::Rss),
            _ => None,
        }
    }

    /// Get content type for this format
    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Json => "application/json",
            ExportFormat::Csv => "text/csv",
            ExportFormat::Rss => "application/rss+xml",
        }
    }
}

/// Convert response to CSV format
pub fn to_csv(response: &SearxngResponse) -> String {
    let mut csv = String::from("title,url,content,engine,category,score\n");

    for result in &response.results {
        let engine = result.engine.as_deref().unwrap_or("");
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}\n",
            escape_csv(&result.title),
            escape_csv(&result.url),
            escape_csv(&result.content),
            engine,
            result.category,
            result.score
        ));
    }

    csv
}

/// Get current RFC 2822 date string
fn get_rfc2822_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Simple RFC 2822 format approximation
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    // Calculate date components (simplified)
    let secs_per_day = 86400u64;
    let days_since_epoch = now / secs_per_day;
    let day_of_week = ((days_since_epoch + 4) % 7) as usize; // Jan 1, 1970 was Thursday

    // Approximate year/month/day calculation
    let mut year = 1970i32;
    let mut remaining_days = days_since_epoch as i32;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0usize;
    for (i, &days_in_month) in days_in_months.iter().enumerate() {
        if remaining_days < days_in_month {
            month = i;
            break;
        }
        remaining_days -= days_in_month;
    }

    let day = remaining_days + 1;
    let hour = (now % secs_per_day) / 3600;
    let minute = (now % 3600) / 60;
    let second = now % 60;

    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        days[day_of_week],
        day,
        months[month],
        year,
        hour,
        minute,
        second
    )
}

/// Convert response to RSS format
pub fn to_rss(response: &SearxngResponse, base_url: &str) -> String {
    let current_date = get_rfc2822_date();

    let items: Vec<String> = response.results.iter().map(|r| {
        format!(
            r#"    <item>
      <title>{}</title>
      <link>{}</link>
      <description>{}</description>
      <pubDate>{}</pubDate>
    </item>"#,
            escape_xml(&r.title),
            escape_xml(&r.url),
            escape_xml(&r.content),
            current_date.clone()
        )
    }).collect();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>WebSurfX Search: {}</title>
    <link>{}/search?q={}</link>
    <description>Search results for: {}</description>
    <lastBuildDate>{}</lastBuildDate>
    <atom:link href="{}/search?q={}&amp;format=rss" rel="self" type="application/rss+xml"/>
{}
  </channel>
</rss>"#,
        escape_xml(&response.query),
        base_url,
        urlencoding::encode(&response.query),
        escape_xml(&response.query),
        current_date,
        base_url,
        urlencoding::encode(&response.query),
        items.join("\n")
    )
}

/// Escape special characters for CSV
fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"").replace('\n', " ").replace('\r', "")
}

/// Escape special characters for XML
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
