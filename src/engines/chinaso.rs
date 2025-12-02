//! The `chinaso` module handles fetching search results from Chinaso (中国搜索).
//!
//! Based on SearXNG's chinaso.py implementation.
//! Chinaso is the Chinese national search engine.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The Chinaso search engine struct.
pub struct Chinaso;

impl Chinaso {
    /// Creates a new Chinaso search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Chinaso
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://www.chinaso.com/newssearch/all/allResults?q={}&pn={}",
            encoded_query,
            page
        )
    }

    /// Build headers for Chinaso requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        ));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));

        Ok(headers)
    }

    /// Parse search results from Chinaso HTML
    fn parse_results(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        let result_selector = Selector::parse("div.result-item, div.news-item")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let title_selector = Selector::parse("h3 a, .title a")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let content_selector = Selector::parse(".desc, .content, p")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        for element in document.select(&result_selector) {
            if let Some(title_element) = element.select(&title_selector).next() {
                let title: String = title_element.text().collect::<Vec<_>>().join("");
                let url = title_element.value().attr("href").unwrap_or("").to_string();

                if title.is_empty() || url.is_empty() {
                    continue;
                }

                let content: String = element
                    .select(&content_selector)
                    .next()
                    .map(|e| e.text().collect::<Vec<_>>().join(""))
                    .unwrap_or_default();

                let search_result = SearchResult::new(
                    &title.trim(),
                    &url,
                    &content.trim(),
                    &["Chinaso"],
                );
                results.push((url.clone(), search_result));
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Chinaso {
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
