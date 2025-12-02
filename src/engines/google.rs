//! The `google` module handles fetching search results from the Google search engine.
//!
//! This implementation is based on the SearXNG google.py engine and includes:
//! - CAPTCHA detection
//! - Regional/language support
//! - Time range filtering
//! - Safe search
//!
//! Note: Google actively blocks automated requests, so this engine may require
//! additional measures like rotating proxies or user agents.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The Google search engine struct.
pub struct Google {
    /// Google search domain
    domain: String,
    /// Language code
    lang: String,
}

impl Google {
    /// Creates a new Google search engine instance.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language code (e.g., "en", "zh-CN")
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Google` instance or an error.
    pub fn new(lang: &str) -> Result<Self, EngineError> {
        let domain = match lang {
            "zh-CN" | "zh-TW" => "www.google.com.hk".to_string(),
            "ja" => "www.google.co.jp".to_string(),
            "ko" => "www.google.co.kr".to_string(),
            "de" => "www.google.de".to_string(),
            "fr" => "www.google.fr".to_string(),
            _ => "www.google.com".to_string(),
        };

        Ok(Self {
            domain,
            lang: lang.to_string(),
        })
    }

    /// Build the search URL for Google
    fn build_url(&self, query: &str, page: u32, safe_search: u8) -> String {
        // Ensure page is at least 1
        let page = page.max(1);
        let start = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        // Safe search mapping: 0 = off, 1 = medium, 2 = high
        let safe = match safe_search {
            0 => "off",
            1 => "medium",
            _ => "high",
        };

        format!(
            "https://{}/search?q={}&start={}&hl={}&lr=lang_{}&ie=utf8&oe=utf8&safe={}",
            self.domain,
            encoded_query,
            start,
            self.lang,
            self.lang,
            safe
        )
    }

    /// Build headers for Google requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderName, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        // Cookie to accept consent
        headers.insert(COOKIE, HeaderValue::from_static("CONSENT=YES+"));

        // Add Sec-Fetch headers to appear more like a real browser
        headers.insert(
            HeaderName::from_static("sec-fetch-dest"),
            HeaderValue::from_static("document")
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-mode"),
            HeaderValue::from_static("navigate")
        );
        headers.insert(
            HeaderName::from_static("sec-fetch-site"),
            HeaderValue::from_static("same-origin")
        );

        Ok(headers)
    }

    /// Check if response contains CAPTCHA
    fn is_captcha_response(&self, html: &str) -> bool {
        html.contains("sorry.google.com") ||
        html.contains("/sorry/") ||
        html.contains("unusual traffic") ||
        html.contains("captcha")
    }

    /// Parse search results from Google HTML
    fn parse_results(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        if self.is_captcha_response(html) {
            log::warn!("Google CAPTCHA detected");
            return Err(Report::new(EngineError::RequestError));
        }

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Google uses various selectors for results
        // Try multiple patterns to handle different Google layouts
        let result_selectors = [
            "div.g",
            "div[data-hveid]",
            "div.MjjYud",
        ];

        let title_selectors = [
            "h3",
            "a h3",
            ".LC20lb",
        ];

        let link_selectors = [
            "a[href]",
            "div.yuRUbf a",
        ];

        let content_selectors = [
            "div.VwiC3b",
            "span.aCOpRe",
            "div[data-sncf]",
            ".IsZvec",
        ];

        // Try each result selector
        for result_selector_str in &result_selectors {
            if let Ok(result_selector) = Selector::parse(result_selector_str) {
                for element in document.select(&result_selector) {
                    // Try to find title
                    let mut title = String::new();
                    for title_sel_str in &title_selectors {
                        if let Ok(title_sel) = Selector::parse(title_sel_str) {
                            if let Some(title_elem) = element.select(&title_sel).next() {
                                title = title_elem.text().collect::<Vec<_>>().join("");
                                break;
                            }
                        }
                    }

                    if title.is_empty() {
                        continue;
                    }

                    // Try to find URL
                    let mut url = String::new();
                    for link_sel_str in &link_selectors {
                        if let Ok(link_sel) = Selector::parse(link_sel_str) {
                            if let Some(link_elem) = element.select(&link_sel).next() {
                                if let Some(href) = link_elem.value().attr("href") {
                                    // Filter out Google internal links
                                    if !href.starts_with("/search") &&
                                       !href.starts_with("/url?") &&
                                       href.starts_with("http") {
                                        url = href.to_string();
                                        break;
                                    } else if href.starts_with("/url?") {
                                        // Extract actual URL from Google redirect
                                        if let Some(actual_url) = Self::extract_url_from_redirect(href) {
                                            url = actual_url;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if url.is_empty() {
                        continue;
                    }

                    // Try to find content/description
                    let mut content = String::new();
                    for content_sel_str in &content_selectors {
                        if let Ok(content_sel) = Selector::parse(content_sel_str) {
                            if let Some(content_elem) = element.select(&content_sel).next() {
                                content = content_elem.text().collect::<Vec<_>>().join(" ");
                                if !content.is_empty() {
                                    break;
                                }
                            }
                        }
                    }

                    let search_result = SearchResult::new(
                        &title.trim(),
                        &url,
                        &content.trim(),
                        &["Google"],
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

    /// Extract actual URL from Google redirect URL
    fn extract_url_from_redirect(redirect_url: &str) -> Option<String> {
        // Parse /url?q=actual_url&...
        if let Some(start) = redirect_url.find("url?q=") {
            let url_part = &redirect_url[start + 6..];
            if let Some(end) = url_part.find('&') {
                return Some(urlencoding::decode(&url_part[..end]).ok()?.to_string());
            } else {
                return Some(urlencoding::decode(url_part).ok()?.to_string());
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl SearchEngine for Google {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page, safe_search);
        let headers = self.build_headers(user_agent)?;

        let html = self.fetch_html_from_upstream(&url, headers, client).await?;

        self.parse_results(&html)
    }
}
