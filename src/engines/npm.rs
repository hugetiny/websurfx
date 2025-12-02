//! The `npm` module handles fetching search results from npm registry.
//!
//! Based on SearXNG's npm.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// npm API response structures
#[derive(Deserialize, Debug)]
struct NpmResponse {
    objects: Option<Vec<NpmObject>>,
}

#[derive(Deserialize, Debug)]
struct NpmObject {
    package: Option<NpmPackage>,
}

#[derive(Deserialize, Debug)]
struct NpmPackage {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    links: Option<NpmLinks>,
}

#[derive(Deserialize, Debug)]
struct NpmLinks {
    npm: Option<String>,
    #[allow(dead_code)]
    homepage: Option<String>,
    #[allow(dead_code)]
    repository: Option<String>,
}

/// The npm search engine struct.
pub struct Npm;

impl Npm {
    /// Creates a new npm search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for npm
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://registry.npmjs.org/-/v1/search?text={}&size=10&from={}",
            encoded_query,
            offset
        )
    }

    /// Build headers for npm requests
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
impl SearchEngine for Npm {
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

        let response: NpmResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(objects) = response.objects {
            for obj in objects {
                if let Some(package) = obj.package {
                    if let Some(name) = &package.name {
                        let url = package.links
                            .as_ref()
                            .and_then(|l| l.npm.clone())
                            .unwrap_or_else(|| format!("https://www.npmjs.com/package/{}", name));

                        let mut title = name.clone();
                        if let Some(version) = &package.version {
                            title = format!("{} v{}", name, version);
                        }

                        let content = package.description.clone().unwrap_or_default();

                        let search_result = SearchResult::new(
                            &title,
                            &url,
                            &content,
                            &["npm"],
                        );
                        results.push((url.clone(), search_result));
                    }
                }
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}
