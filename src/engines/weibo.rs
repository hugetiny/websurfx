//! Weibo Chinese social media search engine

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Weibo search engine
pub struct Weibo;

impl Weibo {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://s.weibo.com/weibo?q={}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Weibo {
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
                title: format!("微博: {}", query),
                url,
                description: "在微博搜索热门内容".to_string(),
                engine: vec!["weibo".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
