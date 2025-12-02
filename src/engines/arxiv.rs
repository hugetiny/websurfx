//! The `arxiv` module handles fetching search results from arXiv.
//!
//! Based on SearXNG's arxiv.py implementation.
//! arXiv is a free distribution service and an open-access archive for scholarly articles.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// The arXiv search engine struct.
pub struct Arxiv;

impl Arxiv {
    /// Creates a new arXiv search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for arXiv
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let start = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://export.arxiv.org/api/query?search_query=all:{}&start={}&max_results=10",
            encoded_query,
            start
        )
    }

    /// Build headers for arXiv requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/xml"));

        Ok(headers)
    }

    /// Parse search results from arXiv XML
    fn parse_results(&self, xml: &str) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let document = Html::parse_document(xml);
        let mut results = Vec::new();

        let entry_selector = Selector::parse("entry")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let title_selector = Selector::parse("title")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let id_selector = Selector::parse("id")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let summary_selector = Selector::parse("summary")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let author_selector = Selector::parse("author name")
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        for entry in document.select(&entry_selector) {
            let title: String = entry
                .select(&title_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default()
                .replace('\n', " ")
                .trim()
                .to_string();

            let url: String = entry
                .select(&id_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default();

            if title.is_empty() || url.is_empty() {
                continue;
            }

            let summary: String = entry
                .select(&summary_selector)
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .unwrap_or_default()
                .replace('\n', " ")
                .trim()
                .to_string();

            let authors: Vec<String> = entry
                .select(&author_selector)
                .map(|e| e.text().collect::<Vec<_>>().join(""))
                .collect();

            let mut content = summary.clone();
            if !authors.is_empty() {
                content = format!("Authors: {} | {}", authors.join(", "), content);
            }

            // Truncate content if too long
            if content.len() > 300 {
                content = format!("{}...", &content[..300]);
            }

            let search_result = SearchResult::new(
                &title,
                &url,
                &content,
                &["arXiv"],
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
impl SearchEngine for Arxiv {
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

        let xml = self.fetch_html_from_upstream(&url, headers, client).await?;

        self.parse_results(&xml)
    }
}
