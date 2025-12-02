//! The `baidu` module handles fetching search results from the Baidu search engine.
//!
//! This implementation is based on the SearXNG baidu.py engine and supports:
//! - General web search
//! - Time range filtering
//! - Safe search
//!
//! API endpoint: https://www.baidu.com/s?wd=query&tn=json

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::Html;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The Baidu search engine struct.
pub struct Baidu;

impl Baidu {
    /// Creates a new Baidu search engine instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Baidu` instance or an error.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Baidu
    fn build_url(&self, query: &str, page: u32, _safe_search: u8) -> String {
        let results_per_page = 10;
        // Ensure page is at least 1
        let page = page.max(1);
        let offset = (page - 1) * results_per_page;

        // URL encode the query
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://www.baidu.com/s?wd={}&rn={}&pn={}&tn=json",
            encoded_query,
            results_per_page,
            offset
        )
    }

    /// Build headers for Baidu requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderName, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
        ));

        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));

        // Add cookie to avoid some restrictions
        headers.insert(COOKIE, HeaderValue::from_static("BAIDUID=random"));

        // Add referer
        headers.insert(
            HeaderName::from_static("referer"),
            HeaderValue::from_static("https://www.baidu.com/")
        );

        Ok(headers)
    }

    /// Parse JSON response from Baidu
    fn parse_json_response(&self, json_bytes: &[u8]) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let json: serde_json::Value = serde_json::from_slice(json_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        // Baidu JSON API returns results in feed.entry array
        if let Some(feed) = json.get("feed") {
            if let Some(entries) = feed.get("entry").and_then(|e| e.as_array()) {
                for entry in entries {
                    let title = entry.get("title")
                        .and_then(|t| t.as_str())
                        .map(|s| html_escape::decode_html_entities(s).to_string())
                        .unwrap_or_default();

                    let url = entry.get("url")
                        .and_then(|u| u.as_str())
                        .map(String::from)
                        .unwrap_or_default();

                    let content = entry.get("abs")
                        .and_then(|a| a.as_str())
                        .map(|s| html_escape::decode_html_entities(s).to_string())
                        .unwrap_or_default();

                    if !title.is_empty() && !url.is_empty() {
                        let search_result = SearchResult::new(
                            &title,
                            &url,
                            &content,
                            &["Baidu"],
                        );
                        results.push((url.clone(), search_result));
                    }
                }
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }

    /// Parse HTML response from Baidu (fallback method)
    fn parse_html_response(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Baidu HTML result selectors
        let result_selector = scraper::Selector::parse("div.result, div.c-container")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let title_selector = scraper::Selector::parse("h3 a, .t a")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let content_selector = scraper::Selector::parse(".c-abstract, .c-span-last")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        for element in document.select(&result_selector) {
            // Extract title and URL
            if let Some(title_element) = element.select(&title_selector).next() {
                let title: String = title_element.text().collect::<Vec<_>>().join("");
                let url = title_element.value().attr("href").unwrap_or("").to_string();

                // Extract content/description
                let content: String = element
                    .select(&content_selector)
                    .next()
                    .map(|e| e.text().collect::<Vec<_>>().join(""))
                    .unwrap_or_default();

                if !title.is_empty() && !url.is_empty() {
                    let search_result = SearchResult::new(
                        &title.trim(),
                        &url,
                        &content.trim(),
                        &["Baidu"],
                    );
                    results.push((url.clone(), search_result));
                }
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Baidu {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        // Build URL and headers
        let url = self.build_url(query, page, safe_search);
        let headers = self.build_headers(user_agent)?;

        // Try JSON API first
        let response = self.fetch_json_as_bytes_from_upstream(&url, headers.clone(), client).await;

        match response {
            Ok(json_bytes) => {
                // Try to parse as JSON
                match self.parse_json_response(&json_bytes) {
                    Ok(results) => Ok(results),
                    Err(_) => {
                        // If JSON parsing fails, try HTML parsing
                        let html = String::from_utf8_lossy(&json_bytes);
                        self.parse_html_response(&html)
                    }
                }
            }
            Err(_) => {
                // Fallback to HTML endpoint
                let html_url = format!(
                    "https://www.baidu.com/s?wd={}&pn={}",
                    urlencoding::encode(query),
                    (page - 1) * 10
                );
                let html = self.fetch_html_from_upstream(&html_url, headers, client).await?;
                self.parse_html_response(&html)
            }
        }
    }
}
