//! The `soundcloud` module handles fetching search results from SoundCloud.
//!
//! Based on SearXNG's soundcloud.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The SoundCloud search engine struct.
pub struct SoundCloud;

impl SoundCloud {
    /// Creates a new SoundCloud search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for SoundCloud
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://soundcloud.com/search?q={}&offset={}",
            encoded_query,
            offset
        )
    }

    /// Build headers for SoundCloud requests
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

    /// Parse search results from SoundCloud HTML
    fn parse_results(&self, html: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        let result_selector = Selector::parse("li.searchList__item, div.searchItem")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let title_selector = Selector::parse("a.soundTitle__title, .soundTitle__titleHero")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let user_selector = Selector::parse("a.soundTitle__username, .soundTitle__usernameHero")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let link_selector = Selector::parse("a[href]")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        for element in document.select(&result_selector) {
            // Try to find title
            let title: String = element
                .select(&title_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            // Try to find URL
            let mut url = String::new();
            for link in element.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    if href.starts_with("https://soundcloud.com/") && !href.contains("/search") {
                        url = href.to_string();
                        break;
                    } else if href.starts_with("/") && !href.starts_with("/search") {
                        url = format!("https://soundcloud.com{}", href);
                        break;
                    }
                }
            }

            if url.is_empty() {
                continue;
            }

            // Get username
            let username: String = element
                .select(&user_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default();

            let content = if username.is_empty() {
                String::new()
            } else {
                format!("by {}", username.trim())
            };

            let search_result = SearchResult::new(
                &title.trim(),
                &url,
                &content,
                &["SoundCloud"],
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
impl SearchEngine for SoundCloud {
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
