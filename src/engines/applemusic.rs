//! Apple Music search

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Apple Music search engine
pub struct AppleMusic;

impl AppleMusic {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://music.apple.com/search?term={}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for AppleMusic {
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
                title: format!("Apple Music: {}", query),
                url,
                description: "Search music on Apple Music".to_string(),
                engine: vec!["applemusic".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
