//! The `crates` module handles fetching search results from crates.io.
//!
//! Based on SearXNG's crates.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// crates.io API response structures
#[derive(Deserialize, Debug)]
struct CratesResponse {
    crates: Option<Vec<Crate>>,
}

#[derive(Deserialize, Debug)]
struct Crate {
    name: Option<String>,
    max_version: Option<String>,
    description: Option<String>,
    downloads: Option<u64>,
    #[allow(dead_code)]
    repository: Option<String>,
}

/// The crates.io search engine struct.
pub struct Crates;

impl Crates {
    /// Creates a new crates.io search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for crates.io
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://crates.io/api/v1/crates?q={}&page={}&per_page=10",
            encoded_query,
            page
        )
    }

    /// Build headers for crates.io requests
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
impl SearchEngine for Crates {
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

        let response: CratesResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(crates) = response.crates {
            for crate_info in crates {
                if let Some(name) = &crate_info.name {
                    let url = format!("https://crates.io/crates/{}", name);

                    let mut title = name.clone();
                    if let Some(version) = &crate_info.max_version {
                        title = format!("{} v{}", name, version);
                    }

                    let mut content = crate_info.description.clone().unwrap_or_default();
                    if let Some(downloads) = crate_info.downloads {
                        content = format!("{} (Downloads: {})", content, downloads);
                    }

                    let search_result = SearchResult::new(
                        &title,
                        &url,
                        &content,
                        &["crates.io"],
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
