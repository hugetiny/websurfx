//! Spotify music search (via web)

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Spotify search engine
pub struct Spotify;

impl Spotify {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://open.spotify.com/search/{}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Spotify {
    async fn results(
        &self,
        query: &str,
        _page: u32,
        _user_agent: &str,
        _client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query);

        let results = vec![(
            url.clone(),
            SearchResult {
                title: format!("Spotify: {}", query),
                url,
                description: "Search music on Spotify".to_string(),
                engine: vec!["spotify".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
