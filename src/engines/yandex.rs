//! The `yandex` module handles fetching search results from the Yandex search engine.
//!
//! Based on SearXNG's yandex.py implementation.
//! Yandex is a major Russian search engine.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The Yandex search engine struct.
pub struct Yandex;

impl Yandex {
    /// Creates a new Yandex search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Yandex
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://yandex.com/search/?text={}&p={}",
            encoded_query,
            page - 1
        )
    }

    /// Build headers for Yandex requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        ));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        // Cookie to disable safe search
        headers.insert(COOKIE, HeaderValue::from_static(
            "yp=1716337604.sp.family%3A0"
        ));

        Ok(headers)
    }

    /// Parse search results from Yandex HTML
    fn parse_results(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Yandex uses various selectors
        let result_selectors = [
            "li.serp-item",
            "div.organic",
            "div[data-cid]",
        ];

        let title_selectors = [
            "h2 a",
            ".organic__title-wrapper a",
            ".OrganicTitle-LinkText",
        ];

        let content_selectors = [
            ".organic__text",
            ".OrganicText",
            ".text-container",
        ];

        for result_sel_str in &result_selectors {
            if let Ok(result_selector) = Selector::parse(result_sel_str) {
                for element in document.select(&result_selector) {
                    let mut title = String::new();
                    let mut url = String::new();

                    for title_sel_str in &title_selectors {
                        if let Ok(title_sel) = Selector::parse(title_sel_str) {
                            if let Some(title_elem) = element.select(&title_sel).next() {
                                title = title_elem.text().collect::<Vec<_>>().join("");
                                if let Some(href) = title_elem.value().attr("href") {
                                    if href.starts_with("http") {
                                        url = href.to_string();
                                    }
                                }
                                break;
                            }
                        }
                    }

                    if title.is_empty() || url.is_empty() {
                        continue;
                    }

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
                        &["Yandex"],
                    );
                    results.push((url.clone(), search_result));
                }
            }

            if !results.is_empty() {
                break;
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Yandex {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page);
        let headers = self.build_headers(user_agent)?;

        let html = self.fetch_html_from_upstream(&url, headers, client).await?;

        self.parse_results(&html)
    }
}
