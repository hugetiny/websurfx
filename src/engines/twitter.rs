//! Twitter/X search engine

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Twitter search engine (via Nitter)
pub struct Twitter;

impl Twitter {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://nitter.net/search?q={}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Twitter {
    async fn results(
        &self,
        query: &str,
        _page: u32,
        _user_agent: &str,
        _client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query);

        // Twitter/X requires authentication, return search link
        let results = vec![(
            url.clone(),
            SearchResult {
                title: format!("Twitter/X: {}", query),
                url,
                description: "Search on Twitter/X via Nitter".to_string(),
                engine: vec!["twitter".to_string()],
                relevance_score: 0.0,
            },
        )];

        Ok(results)
    }
}
