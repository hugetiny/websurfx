//! Mastodon search engine

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Mastodon search engine
pub struct Mastodon;

impl Mastodon {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://mastodon.social/tags/{}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Mastodon {
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
                title: format!("Mastodon: #{}", query),
                url,
                description: "Search hashtag on Mastodon".to_string(),
                engine: vec!["mastodon".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
