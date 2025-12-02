//! The `openlibrary` module handles fetching search results from Open Library.
//!
//! Based on SearXNG's openlibrary.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Open Library API response structures
#[derive(Deserialize, Debug)]
struct OpenLibraryResponse {
    docs: Option<Vec<OpenLibraryDoc>>,
}

#[derive(Deserialize, Debug)]
struct OpenLibraryDoc {
    title: Option<String>,
    key: Option<String>,
    author_name: Option<Vec<String>>,
    first_publish_year: Option<u32>,
    publisher: Option<Vec<String>>,
    #[allow(dead_code)]
    isbn: Option<Vec<String>>,
    subject: Option<Vec<String>>,
}

/// The Open Library search engine struct.
pub struct OpenLibrary;

impl OpenLibrary {
    /// Creates a new Open Library search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Open Library
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://openlibrary.org/search.json?q={}&page={}&limit=10",
            encoded_query,
            page
        )
    }

    /// Build headers for Open Library requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Ok(headers)
    }
}

#[async_trait::async_trait]
impl SearchEngine for OpenLibrary {
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

        let response_bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let response: OpenLibraryResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(docs) = response.docs {
            for doc in docs {
                let title = doc.title.clone().unwrap_or_default();
                let key = doc.key.clone().unwrap_or_default();

                if title.is_empty() || key.is_empty() {
                    continue;
                }

                let url = format!("https://openlibrary.org{}", key);

                let mut content_parts = Vec::new();

                if let Some(authors) = &doc.author_name {
                    if !authors.is_empty() {
                        content_parts.push(format!("by {}", authors.join(", ")));
                    }
                }

                if let Some(year) = doc.first_publish_year {
                    content_parts.push(format!("({})", year));
                }

                if let Some(publishers) = &doc.publisher {
                    if !publishers.is_empty() {
                        content_parts.push(publishers[0].clone());
                    }
                }

                if let Some(subjects) = &doc.subject {
                    let display_subjects: Vec<&str> = subjects.iter()
                        .take(3)
                        .map(|s| s.as_str())
                        .collect();
                    if !display_subjects.is_empty() {
                        content_parts.push(display_subjects.join(", "));
                    }
                }

                let content = content_parts.join(" | ");

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["OpenLibrary"],
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
