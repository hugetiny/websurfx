//! The `pypi` module handles fetching search results from PyPI.
//!
//! Based on SearXNG's pypi.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The PyPI search engine struct.
pub struct PyPI;

impl PyPI {
    /// Creates a new PyPI search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for PyPI
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://pypi.org/search/?q={}&page={}",
            encoded_query,
            page
        )
    }

    /// Build headers for PyPI requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        ));

        Ok(headers)
    }

    /// Parse search results from PyPI HTML
    fn parse_results(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        let result_selector = Selector::parse("a.package-snippet")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let name_selector = Selector::parse("span.package-snippet__name")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let version_selector = Selector::parse("span.package-snippet__version")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let description_selector = Selector::parse("p.package-snippet__description")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        for element in document.select(&result_selector) {
            let href = element.value().attr("href").unwrap_or("");
            if href.is_empty() {
                continue;
            }

            let url = format!("https://pypi.org{}", href);

            let name: String = element
                .select(&name_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default();

            let version: String = element
                .select(&version_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default();

            let description: String = element
                .select(&description_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default();

            if name.is_empty() {
                continue;
            }

            let title = if version.is_empty() {
                name.clone()
            } else {
                format!("{} {}", name.trim(), version.trim())
            };

            let search_result = SearchResult::new(
                &title,
                &url,
                &description.trim(),
                &["PyPI"],
            );
            results.push((url.clone(), search_result));
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl SearchEngine for PyPI {
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
