//! The `github` module handles fetching search results from GitHub.
//!
//! Based on SearXNG's github.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// GitHub API response structures
#[derive(Deserialize, Debug)]
struct GitHubResponse {
    items: Option<Vec<GitHubItem>>,
}

#[derive(Deserialize, Debug)]
struct GitHubItem {
    #[allow(dead_code)]
    name: Option<String>,
    full_name: Option<String>,
    html_url: Option<String>,
    description: Option<String>,
    stargazers_count: Option<u32>,
    language: Option<String>,
}

/// The GitHub search engine struct.
pub struct GitHub;

impl GitHub {
    /// Creates a new GitHub search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for GitHub
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://api.github.com/search/repositories?q={}&page={}&per_page=10",
            encoded_query,
            page
        )
    }

    /// Build headers for GitHub requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));

        Ok(headers)
    }
}

#[async_trait::async_trait]
impl SearchEngine for GitHub {
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

        let response: GitHubResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(items) = response.items {
            for item in items {
                if let (Some(name), Some(url)) = (&item.full_name, &item.html_url) {
                    let mut content = item.description.clone().unwrap_or_default();

                    // Add metadata
                    if let Some(lang) = &item.language {
                        content = format!("[{}] {}", lang, content);
                    }
                    if let Some(stars) = item.stargazers_count {
                        content = format!("{} ⭐ {}", content, stars);
                    }

                    let search_result = SearchResult::new(
                        name,
                        url,
                        &content,
                        &["GitHub"],
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
