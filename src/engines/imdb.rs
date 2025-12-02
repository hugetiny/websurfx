//! The `imdb` module handles fetching search results from IMDb.
//!
//! Based on SearXNG's imdb.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// IMDb suggestion API response
#[derive(Deserialize, Debug)]
struct IMDbResponse {
    d: Option<Vec<IMDbItem>>,
}

#[derive(Deserialize, Debug)]
struct IMDbItem {
    l: Option<String>,  // title
    id: Option<String>, // imdb id
    q: Option<String>,  // category/type
    s: Option<String>,  // starring
    y: Option<u32>,     // year
    yr: Option<String>, // year range for TV series
}

/// The IMDb search engine struct.
pub struct IMDb;

impl IMDb {
    /// Creates a new IMDb search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for IMDb
    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://v3.sg.media-imdb.com/suggestion/x/{}.json",
            encoded_query
        )
    }

    /// Build headers for IMDb requests
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
impl SearchEngine for IMDb {
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

        let response: IMDbResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(items) = response.d {
            for item in items {
                let title = item.l.clone().unwrap_or_default();
                let id = item.id.clone().unwrap_or_default();

                if title.is_empty() || id.is_empty() {
                    continue;
                }

                let url = format!("https://www.imdb.com/title/{}/", id);

                let mut content_parts = Vec::new();

                if let Some(category) = &item.q {
                    content_parts.push(category.clone());
                }
                if let Some(year) = item.y {
                    content_parts.push(year.to_string());
                } else if let Some(year_range) = &item.yr {
                    content_parts.push(year_range.clone());
                }
                if let Some(starring) = &item.s {
                    content_parts.push(starring.clone());
                }

                let content = content_parts.join(" | ");

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["IMDb"],
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
