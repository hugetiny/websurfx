//! The `dockerhub` module handles fetching search results from Docker Hub.
//!
//! Based on SearXNG's docker_hub.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Docker Hub API response structures
#[derive(Deserialize, Debug)]
struct DockerHubResponse {
    results: Option<Vec<DockerHubImage>>,
}

#[derive(Deserialize, Debug)]
struct DockerHubImage {
    name: Option<String>,
    slug: Option<String>,
    description: Option<String>,
    star_count: Option<u64>,
    pull_count: Option<String>,
    is_official: Option<bool>,
}

/// The Docker Hub search engine struct.
pub struct DockerHub;

impl DockerHub {
    /// Creates a new Docker Hub search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Docker Hub
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://hub.docker.com/api/content/v1/products/search?q={}&page={}&page_size=10",
            encoded_query,
            page
        )
    }

    /// Build headers for Docker Hub requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            reqwest::header::HeaderName::from_static("search-version"),
            HeaderValue::from_static("v3")
        );

        Ok(headers)
    }

    /// Format pull count
    fn format_pulls(pulls: &str) -> String {
        if let Ok(num) = pulls.replace(",", "").parse::<u64>() {
            if num >= 1_000_000_000 {
                format!("{:.1}B pulls", num as f64 / 1_000_000_000.0)
            } else if num >= 1_000_000 {
                format!("{:.1}M pulls", num as f64 / 1_000_000.0)
            } else if num >= 1_000 {
                format!("{:.1}K pulls", num as f64 / 1_000.0)
            } else {
                format!("{} pulls", num)
            }
        } else {
            format!("{} pulls", pulls)
        }
    }
}

#[async_trait::async_trait]
impl SearchEngine for DockerHub {
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

        let response: DockerHubResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(images) = response.results {
            for image in images {
                let name = image.name.clone().or(image.slug.clone()).unwrap_or_default();

                if name.is_empty() {
                    continue;
                }

                let url = format!("https://hub.docker.com/_/{}", name);

                let mut title = name.clone();
                if image.is_official == Some(true) {
                    title = format!("✓ {} (Official)", name);
                }

                let mut content_parts = Vec::new();

                if let Some(desc) = &image.description {
                    if !desc.is_empty() {
                        let snippet = if desc.len() > 150 {
                            format!("{}...", &desc[..150])
                        } else {
                            desc.clone()
                        };
                        content_parts.push(snippet);
                    }
                }

                if let Some(stars) = image.star_count {
                    content_parts.push(format!("⭐ {}", stars));
                }

                if let Some(pulls) = &image.pull_count {
                    content_parts.push(Self::format_pulls(pulls));
                }

                let content = content_parts.join(" | ");

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["DockerHub"],
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
