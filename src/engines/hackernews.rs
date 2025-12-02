//! The `hackernews` module handles fetching search results from Hacker News via Algolia.
//!
//! Based on SearXNG's hackernews.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Hacker News API response structures
#[derive(Deserialize, Debug)]
struct HNResponse {
    hits: Option<Vec<HNHit>>,
}

#[derive(Deserialize, Debug)]
struct HNHit {
    title: Option<String>,
    url: Option<String>,
    #[serde(rename = "objectID")]
    object_id: Option<String>,
    author: Option<String>,
    points: Option<i32>,
    num_comments: Option<i32>,
    #[serde(rename = "story_text")]
    story_text: Option<String>,
}

/// The Hacker News search engine struct.
pub struct HackerNews;

impl HackerNews {
    /// Creates a new Hacker News search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Hacker News
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://hn.algolia.com/api/v1/search?query={}&page={}&hitsPerPage=10",
            encoded_query,
            page - 1
        )
    }

    /// Build headers for Hacker News requests
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
impl SearchEngine for HackerNews {
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

        let response: HNResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(hits) = response.hits {
            for hit in hits {
                let title = hit.title.clone().unwrap_or_default();
                if title.is_empty() {
                    continue;
                }

                // Use the original URL if available, otherwise link to HN discussion
                let url = hit.url.clone().unwrap_or_else(|| {
                    hit.object_id
                        .as_ref()
                        .map(|id| format!("https://news.ycombinator.com/item?id={}", id))
                        .unwrap_or_default()
                });

                if url.is_empty() {
                    continue;
                }

                let mut content_parts = Vec::new();

                if let Some(author) = &hit.author {
                    content_parts.push(format!("by {}", author));
                }
                if let Some(points) = hit.points {
                    content_parts.push(format!("{} points", points));
                }
                if let Some(comments) = hit.num_comments {
                    content_parts.push(format!("{} comments", comments));
                }

                // Add story text snippet if available
                if let Some(text) = &hit.story_text {
                    let snippet = if text.len() > 200 {
                        format!("{}...", &text[..200])
                    } else {
                        text.clone()
                    };
                    // Clean HTML tags
                    let clean_snippet = snippet
                        .replace("<p>", " ")
                        .replace("</p>", " ")
                        .replace("<br>", " ");
                    content_parts.push(clean_snippet);
                }

                let content = content_parts.join(" | ");

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["HackerNews"],
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
