//! Lobsters tech news search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct LobsterStory {
    short_id: Option<String>,
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    score: Option<i32>,
    comment_count: Option<i32>,
}

/// Lobsters search engine
pub struct Lobsters;

impl Lobsters {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!("https://lobste.rs/search.json?q={}&page={}", encoded_query, page)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Lobsters {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page);
        let headers = self.build_headers(user_agent);
        let bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let stories: Vec<LobsterStory> = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        for story in stories {
            let title = story.title.unwrap_or_default();
            let story_url = story.url.unwrap_or_else(|| {
                story.short_id.map(|id| format!("https://lobste.rs/s/{}", id)).unwrap_or_default()
            });
            let description = story.description.unwrap_or_default();
            let score = story.score.unwrap_or(0);
            let comments = story.comment_count.unwrap_or(0);

            if !title.is_empty() && !story_url.is_empty() {
                results.push((
                    story_url.clone(),
                    SearchResult {
                        title,
                        url: story_url,
                        description: format!("⬆{} 💬{} - {}", score, comments, description),
                        engine: vec!["lobsters".to_string()],
                        relevance_score: 0.0,
                    },
                ));
            }
        }

        Ok(results)
    }
}
