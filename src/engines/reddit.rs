//! The `reddit` module handles fetching search results from Reddit.
//!
//! Based on SearXNG's reddit.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Reddit API response structures
#[derive(Deserialize, Debug)]
struct RedditResponse {
    data: Option<RedditData>,
}

#[derive(Deserialize, Debug)]
struct RedditData {
    children: Option<Vec<RedditChild>>,
}

#[derive(Deserialize, Debug)]
struct RedditChild {
    data: Option<RedditPost>,
}

#[derive(Deserialize, Debug)]
struct RedditPost {
    title: Option<String>,
    permalink: Option<String>,
    #[allow(dead_code)]
    url: Option<String>,
    selftext: Option<String>,
    subreddit: Option<String>,
    score: Option<i32>,
    num_comments: Option<i32>,
    author: Option<String>,
}

/// The Reddit search engine struct.
pub struct Reddit;

impl Reddit {
    /// Creates a new Reddit search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Reddit
    fn build_url(&self, query: &str, _page: u32) -> String {
        let _page = _page.max(1);
        let encoded_query = urlencoding::encode(query);

        // Reddit uses 'after' for pagination, but for simplicity we'll use time-based
        format!(
            "https://www.reddit.com/search.json?q={}&limit=10&sort=relevance",
            encoded_query
        )
    }

    /// Build headers for Reddit requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        // Reddit requires a descriptive user agent
        let reddit_ua = format!("{} (websurfx search engine)", user_agent);
        headers.insert(USER_AGENT, HeaderValue::from_str(&reddit_ua)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Ok(headers)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Reddit {
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

        let response: RedditResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(data) = response.data {
            if let Some(children) = data.children {
                for child in children {
                    if let Some(post) = child.data {
                        let title = post.title.clone().unwrap_or_default();
                        if title.is_empty() {
                            continue;
                        }

                        let url = post.permalink
                            .as_ref()
                            .map(|p| format!("https://www.reddit.com{}", p))
                            .unwrap_or_default();

                        if url.is_empty() {
                            continue;
                        }

                        let mut content_parts = Vec::new();

                        if let Some(subreddit) = &post.subreddit {
                            content_parts.push(format!("r/{}", subreddit));
                        }
                        if let Some(author) = &post.author {
                            content_parts.push(format!("u/{}", author));
                        }
                        if let Some(score) = post.score {
                            content_parts.push(format!("{} points", score));
                        }
                        if let Some(comments) = post.num_comments {
                            content_parts.push(format!("{} comments", comments));
                        }

                        // Add selftext snippet if available
                        if let Some(selftext) = &post.selftext {
                            if !selftext.is_empty() {
                                let snippet = if selftext.len() > 150 {
                                    format!("{}...", &selftext[..150])
                                } else {
                                    selftext.clone()
                                };
                                content_parts.push(snippet);
                            }
                        }

                        let content = content_parts.join(" | ");

                        let search_result = SearchResult::new(
                            &title,
                            &url,
                            &content,
                            &["Reddit"],
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
