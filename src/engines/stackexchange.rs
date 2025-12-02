//! The `stackexchange` module handles fetching search results from Stack Exchange sites.
//!
//! Based on SearXNG's stackexchange.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Stack Exchange API response structures
#[derive(Deserialize, Debug)]
struct StackExchangeResponse {
    items: Option<Vec<StackExchangeItem>>,
}

#[derive(Deserialize, Debug)]
struct StackExchangeItem {
    title: Option<String>,
    link: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    is_answered: Option<bool>,
    answer_count: Option<u32>,
    score: Option<i32>,
}

/// The Stack Exchange search engine struct.
pub struct StackExchange {
    site: String,
}

impl StackExchange {
    /// Creates a new Stack Exchange search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            site: "stackoverflow".to_string(),
        })
    }

    /// Creates a new Stack Exchange search engine instance for a specific site.
    #[allow(dead_code)]
    pub fn with_site(site: &str) -> Result<Self, EngineError> {
        Ok(Self {
            site: site.to_string(),
        })
    }

    /// Build the search URL for Stack Exchange
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://api.stackexchange.com/2.3/search/advanced?order=desc&sort=relevance&q={}&site={}&page={}&pagesize=10",
            encoded_query,
            self.site,
            page
        )
    }

    /// Build headers for Stack Exchange requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, ACCEPT_ENCODING, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));

        Ok(headers)
    }
}

#[async_trait::async_trait]
impl SearchEngine for StackExchange {
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

        let response: StackExchangeResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(items) = response.items {
            for item in items {
                if let (Some(title), Some(url)) = (&item.title, &item.link) {
                    let mut content = String::new();

                    // Add tags
                    if !item.tags.is_empty() {
                        content.push_str(&format!("[{}] ", item.tags.join(", ")));
                    }

                    // Add answer info
                    if let Some(answered) = item.is_answered {
                        if answered {
                            content.push_str("✓ Answered ");
                        }
                    }
                    if let Some(answers) = item.answer_count {
                        content.push_str(&format!("({} answers) ", answers));
                    }
                    if let Some(score) = item.score {
                        content.push_str(&format!("Score: {}", score));
                    }

                    // Decode HTML entities in title
                    let decoded_title = html_escape::decode_html_entities(title);

                    let search_result = SearchResult::new(
                        &decoded_title,
                        url,
                        &content,
                        &["StackExchange"],
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
