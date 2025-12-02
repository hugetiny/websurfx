//! This module provides OpenSearch Description Document support.
//! OpenSearch allows browsers to add websurfx as a search engine.

use actix_web::{HttpRequest, HttpResponse, get, http::header::ContentType};

/// Generate the base URL from the request
fn get_base_url(req: &HttpRequest) -> String {
    let conn_info = req.connection_info();
    let scheme = conn_info.scheme();
    let host = conn_info.host();
    format!("{}://{}", scheme, host)
}

/// Handles the /opensearch.xml endpoint.
/// Returns an OpenSearch Description Document that allows browsers to add websurfx as a search engine.
#[get("/opensearch.xml")]
pub async fn opensearch_xml(req: HttpRequest) -> HttpResponse {
    let base_url = get_base_url(&req);

    let xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/"
                       xmlns:moz="http://www.mozilla.org/2006/browser/search/">
    <ShortName>WebSurfX</ShortName>
    <Description>WebSurfX - A fast, private meta-search engine</Description>
    <InputEncoding>UTF-8</InputEncoding>
    <OutputEncoding>UTF-8</OutputEncoding>
    <Image width="16" height="16" type="image/x-icon">{base_url}/favicon.ico</Image>
    <Url type="text/html" method="GET" template="{base_url}/search?q={{searchTerms}}"/>
    <Url type="application/x-suggestions+json" method="GET" template="{base_url}/autocomplete?q={{searchTerms}}"/>
    <Url type="application/opensearchdescription+xml" rel="self" template="{base_url}/opensearch.xml"/>
    <moz:SearchForm>{base_url}/</moz:SearchForm>
    <Tags>search meta websurfx privacy</Tags>
    <Contact>admin@example.com</Contact>
    <LongName>WebSurfX Meta Search Engine</LongName>
    <Developer>WebSurfX Team</Developer>
    <Attribution>Search data provided by multiple search engines</Attribution>
    <SyndicationRight>open</SyndicationRight>
    <AdultContent>false</AdultContent>
    <Language>*</Language>
</OpenSearchDescription>"#);

    HttpResponse::Ok()
        .content_type("application/opensearchdescription+xml")
        .insert_header(("Cache-Control", "max-age=86400"))
        .body(xml)
}

/// Handles the /autocomplete endpoint for OpenSearch suggestions.
/// Returns JSON array format for browser autocomplete.
#[get("/autocomplete")]
pub async fn autocomplete(
    query: actix_web::web::Query<AutocompleteQuery>,
) -> HttpResponse {
    let q = query.q.as_deref().unwrap_or("");

    if q.is_empty() {
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("[\"\", []]");
    }

    // Fetch autocomplete suggestions from DuckDuckGo
    match fetch_duckduckgo_suggestions(q).await {
        Ok(suggestions) => {
            let response = serde_json::json!([q, suggestions]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
        Err(_) => {
            // Return empty suggestions on error
            let response = serde_json::json!([q, []]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
    }
}

#[derive(serde::Deserialize)]
pub struct AutocompleteQuery {
    pub q: Option<String>,
}

/// Fetch autocomplete suggestions from DuckDuckGo
async fn fetch_duckduckgo_suggestions(query: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    let url = format!(
        "https://duckduckgo.com/ac/?q={}&type=list",
        urlencoding::encode(query)
    );

    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let text = response.text().await?;

    // DuckDuckGo returns format: ["query", ["suggestion1", "suggestion2", ...]]
    let json: serde_json::Value = serde_json::from_str(&text)?;

    if let Some(arr) = json.as_array() {
        if arr.len() >= 2 {
            if let Some(suggestions) = arr[1].as_array() {
                return Ok(suggestions
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect());
            }
        }
    }

    Ok(Vec::new())
}

/// Handles the /autocomplete/google endpoint for Google suggestions.
#[get("/autocomplete/google")]
pub async fn autocomplete_google(
    query: actix_web::web::Query<AutocompleteQuery>,
) -> HttpResponse {
    let q = query.q.as_deref().unwrap_or("");

    if q.is_empty() {
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("[\"\", []]");
    }

    match fetch_google_suggestions(q).await {
        Ok(suggestions) => {
            let response = serde_json::json!([q, suggestions]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
        Err(_) => {
            let response = serde_json::json!([q, []]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
    }
}

/// Fetch autocomplete suggestions from Google
async fn fetch_google_suggestions(query: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    let url = format!(
        "https://suggestqueries.google.com/complete/search?client=firefox&q={}",
        urlencoding::encode(query)
    );

    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let text = response.text().await?;

    // Google returns format: ["query", ["suggestion1", "suggestion2", ...]]
    let json: serde_json::Value = serde_json::from_str(&text)?;

    if let Some(arr) = json.as_array() {
        if arr.len() >= 2 {
            if let Some(suggestions) = arr[1].as_array() {
                return Ok(suggestions
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect());
            }
        }
    }

    Ok(Vec::new())
}

/// Handles the /autocomplete/wikipedia endpoint for Wikipedia suggestions.
#[get("/autocomplete/wikipedia")]
pub async fn autocomplete_wikipedia(
    query: actix_web::web::Query<AutocompleteQuery>,
) -> HttpResponse {
    let q = query.q.as_deref().unwrap_or("");

    if q.is_empty() {
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("[\"\", []]");
    }

    match fetch_wikipedia_suggestions(q).await {
        Ok(suggestions) => {
            let response = serde_json::json!([q, suggestions]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
        Err(_) => {
            let response = serde_json::json!([q, []]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
    }
}

/// Fetch autocomplete suggestions from Wikipedia
async fn fetch_wikipedia_suggestions(query: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=opensearch&format=json&search={}",
        urlencoding::encode(query)
    );

    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let text = response.text().await?;

    // Wikipedia returns format: ["query", ["title1", "title2", ...], [...], [...]]
    let json: serde_json::Value = serde_json::from_str(&text)?;

    if let Some(arr) = json.as_array() {
        if arr.len() >= 2 {
            if let Some(suggestions) = arr[1].as_array() {
                return Ok(suggestions
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect());
            }
        }
    }

    Ok(Vec::new())
}

/// Handles the /autocomplete/startpage endpoint for Startpage suggestions.
#[get("/autocomplete/startpage")]
pub async fn autocomplete_startpage(
    query: actix_web::web::Query<AutocompleteQuery>,
) -> HttpResponse {
    let q = query.q.as_deref().unwrap_or("");

    if q.is_empty() {
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("[\"\", []]");
    }

    // Startpage uses similar API to DuckDuckGo, fallback to DDG
    match fetch_duckduckgo_suggestions(q).await {
        Ok(suggestions) => {
            let response = serde_json::json!([q, suggestions]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
        Err(_) => {
            let response = serde_json::json!([q, []]);
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response.to_string())
        }
    }
}

/// Configure all OpenSearch-related routes
#[allow(dead_code)]
pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(opensearch_xml)
        .service(autocomplete)
        .service(autocomplete_google)
        .service(autocomplete_wikipedia)
        .service(autocomplete_startpage);
}
