//! This module provides image proxy and result proxy functionality
//! similar to SearXNG's image_proxy and morty.

use actix_web::{HttpRequest, HttpResponse, get, web};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;

/// Image proxy query parameters
#[derive(Deserialize)]
pub struct ImageProxyQuery {
    /// The URL to proxy (base64 encoded or plain)
    pub url: String,
    /// Optional: hash for verification
    #[allow(dead_code)]
    pub h: Option<String>,
}

/// Result proxy query parameters
#[derive(Deserialize)]
pub struct ResultProxyQuery {
    /// The URL to proxy
    pub url: String,
    /// Optional: hash for verification
    #[allow(dead_code)]
    pub h: Option<String>,
    /// Open in new tab
    #[allow(dead_code)]
    pub new_tab: Option<bool>,
}

/// Handles the /image_proxy endpoint.
/// Proxies images through the server for privacy.
#[get("/image_proxy")]
pub async fn image_proxy(
    query: web::Query<ImageProxyQuery>,
) -> HttpResponse {
    // Decode URL if it's base64 encoded
    let url = decode_url(&query.url);

    if url.is_empty() {
        return HttpResponse::BadRequest()
            .body("Invalid URL");
    }

    // Verify URL is valid HTTP/HTTPS
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return HttpResponse::BadRequest()
            .body("Only HTTP/HTTPS URLs are allowed");
    }

    // Fetch and proxy the image
    match fetch_and_proxy_image(&url).await {
        Ok((content_type, data)) => {
            HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Cache-Control", "max-age=86400"))
                .insert_header(("X-Content-Type-Options", "nosniff"))
                .body(data)
        }
        Err(e) => {
            // Return a 1x1 transparent pixel on error
            let transparent_pixel: &[u8] = &[
                0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00,
                0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
                0x00, 0x02, 0x01, 0x00, 0x00
            ];
            eprintln!("Image proxy error: {}", e);
            HttpResponse::Ok()
                .content_type("image/gif")
                .body(transparent_pixel.to_vec())
        }
    }
}

/// Handles the /morty endpoint (result proxy).
/// Proxies result links through the server for privacy.
#[get("/morty")]
pub async fn morty_proxy(
    query: web::Query<ResultProxyQuery>,
    _req: HttpRequest,
) -> HttpResponse {
    let url = decode_url(&query.url);

    if url.is_empty() {
        return HttpResponse::BadRequest()
            .body("Invalid URL");
    }

    // Verify URL is valid HTTP/HTTPS
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return HttpResponse::BadRequest()
            .body("Only HTTP/HTTPS URLs are allowed");
    }

    // For result proxy, we fetch the page and rewrite URLs
    match fetch_and_proxy_page(&url).await {
        Ok((content_type, data)) => {
            HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("X-Robots-Tag", "noindex, nofollow"))
                .body(data)
        }
        Err(e) => {
            eprintln!("Morty proxy error: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Error proxying page: {}", e))
        }
    }
}

/// Handles the /redirect endpoint.
/// Redirects to the original URL (used when proxy is disabled).
#[get("/redirect")]
pub async fn redirect_url(
    query: web::Query<ResultProxyQuery>,
) -> HttpResponse {
    let url = decode_url(&query.url);

    if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return HttpResponse::BadRequest()
            .body("Invalid URL");
    }

    HttpResponse::Found()
        .insert_header(("Location", url))
        .insert_header(("X-Robots-Tag", "noindex, nofollow"))
        .finish()
}

/// Decode URL from base64 or return as-is
fn decode_url(encoded: &str) -> String {
    // Try to decode as base64 first
    if let Ok(decoded) = URL_SAFE_NO_PAD.decode(encoded) {
        if let Ok(url) = String::from_utf8(decoded) {
            return url;
        }
    }
    // Return as-is if not base64 encoded
    encoded.to_string()
}

/// Encode URL to base64 for use in proxy URLs
#[allow(dead_code)]
pub fn encode_url(url: &str) -> String {
    URL_SAFE_NO_PAD.encode(url.as_bytes())
}

/// Fetch and proxy an image
async fn fetch_and_proxy_image(url: &str) -> Result<(String, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Accept", "image/*,*/*;q=0.8")
        .send()
        .await?;

    // Check content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    // Verify it's an image
    if !content_type.starts_with("image/") && !content_type.contains("svg") {
        return Err("Not an image".into());
    }

    // Check content length (limit to 10MB)
    if let Some(len) = response.content_length() {
        if len > 10 * 1024 * 1024 {
            return Err("Image too large".into());
        }
    }

    let bytes = response.bytes().await?;
    Ok((content_type, bytes.to_vec()))
}

/// Fetch and proxy a page
async fn fetch_and_proxy_page(url: &str) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .send()
        .await?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_string();

    let text = response.text().await?;

    // Basic URL rewriting for privacy
    // In production, this should use a proper HTML parser
    let rewritten = rewrite_urls(&text, url);

    Ok((content_type, rewritten))
}

/// Basic URL rewriting for proxied pages
fn rewrite_urls(html: &str, _base_url: &str) -> String {
    // This is a simplified implementation
    // A full implementation would parse the HTML and rewrite all URLs

    // Add a base tag and meta tags for privacy
    let privacy_header = r#"
<meta name="robots" content="noindex, nofollow">
<meta http-equiv="Content-Security-Policy" content="upgrade-insecure-requests">
"#;

    // Insert privacy headers after <head>
    if let Some(pos) = html.to_lowercase().find("<head>") {
        let mut result = String::with_capacity(html.len() + privacy_header.len());
        result.push_str(&html[..pos + 6]);
        result.push_str(privacy_header);
        result.push_str(&html[pos + 6..]);
        return result;
    }

    html.to_string()
}

/// Favicon proxy endpoint
#[get("/favicon_proxy")]
pub async fn favicon_proxy(
    query: web::Query<ImageProxyQuery>,
) -> HttpResponse {
    let url = decode_url(&query.url);

    if url.is_empty() {
        return HttpResponse::BadRequest()
            .body("Invalid URL");
    }

    // Extract domain and construct favicon URL if needed
    let favicon_url = if url.contains("/favicon") {
        url
    } else {
        // Try to get favicon from domain
        format!("{}/favicon.ico", url.trim_end_matches('/'))
    };

    match fetch_and_proxy_image(&favicon_url).await {
        Ok((content_type, data)) => {
            HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Cache-Control", "max-age=604800")) // 1 week cache
                .body(data)
        }
        Err(_) => {
            // Return a default favicon on error
            let default_favicon: &[u8] = &[
                0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x10, 0x10, 0x00, 0x00,
                0x01, 0x00, 0x20, 0x00, 0x68, 0x04, 0x00, 0x00, 0x16, 0x00,
                0x00, 0x00
            ];
            HttpResponse::Ok()
                .content_type("image/x-icon")
                .body(default_favicon.to_vec())
        }
    }
}

/// Configure all proxy-related routes
#[allow(dead_code)]
pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(image_proxy)
        .service(morty_proxy)
        .service(redirect_url)
        .service(favicon_proxy);
}
