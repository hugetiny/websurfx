//! The `wikidata` module handles fetching search results from Wikidata.
//!
//! Based on SearXNG's wikidata.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Wikidata API response structures
#[derive(Deserialize, Debug)]
struct WikidataResponse {
    search: Option<Vec<WikidataItem>>,
}

#[derive(Deserialize, Debug)]
struct WikidataItem {
    id: Option<String>,
    label: Option<String>,
    description: Option<String>,
    concepturi: Option<String>,
}

/// The Wikidata search engine struct.
pub struct Wikidata {
    language: String,
}

impl Wikidata {
    /// Creates a new Wikidata search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            language: "en".to_string(),
        })
    }

    /// Creates a new Wikidata search engine instance with a specific language.
    #[allow(dead_code)]
    pub fn with_language(language: &str) -> Result<Self, EngineError> {
        Ok(Self {
            language: language.to_string(),
        })
    }

    /// Build the search URL for Wikidata
    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language={}&format=json&limit=10",
            encoded_query,
            self.language
        )
    }

    /// Build headers for Wikidata requests
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
impl SearchEngine for Wikidata {
    async fn results(
        &self,
        query: &str,
        _page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query);
        let headers = self.build_headers(user_agent)?;

        let response_bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let response: WikidataResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(items) = response.search {
            for item in items {
                let label = item.label.clone().unwrap_or_default();
                let id = item.id.clone().unwrap_or_default();

                if label.is_empty() || id.is_empty() {
                    continue;
                }

                let url = item.concepturi.clone()
                    .unwrap_or_else(|| format!("https://www.wikidata.org/wiki/{}", id));

                let content = item.description.clone().unwrap_or_default();

                let title = format!("{} ({})", label, id);

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["Wikidata"],
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
