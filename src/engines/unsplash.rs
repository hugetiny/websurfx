//! The `unsplash` module handles fetching search results from Unsplash.
//!
//! Based on SearXNG's unsplash.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The Unsplash search engine struct.
pub struct Unsplash;

impl Unsplash {
    /// Creates a new Unsplash search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Unsplash
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://unsplash.com/s/photos/{}?page={}",
            encoded_query,
            page
        )
    }

    /// Build headers for Unsplash requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        ));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        Ok(headers)
    }

    /// Parse search results from Unsplash HTML
    fn parse_results(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        let figure_selector = Selector::parse("figure[itemprop='image']")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let link_selector = Selector::parse("a[itemprop='contentUrl']")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let img_selector = Selector::parse("img")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        for figure in document.select(&figure_selector) {
            // Get link
            if let Some(link_elem) = figure.select(&link_selector).next() {
                let href = link_elem.value().attr("href").unwrap_or("");
                if href.is_empty() {
                    continue;
                }

                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://unsplash.com{}", href)
                };

                // Get image alt text as title
                let title: String = figure
                    .select(&img_selector)
                    .next()
                    .and_then(|img| img.value().attr("alt"))
                    .unwrap_or("Unsplash Image")
                    .to_string();

                let content = "Free high-resolution photo from Unsplash".to_string();

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["Unsplash"],
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
impl SearchEngine for Unsplash {
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
