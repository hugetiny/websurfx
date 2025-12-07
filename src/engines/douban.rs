//! Douban Chinese media search engine

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Douban search engine
pub struct Douban;

impl Douban {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://www.douban.com/search?q={}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Douban {
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
                title: format!("豆瓣: {}", query),
                url,
                description: "在豆瓣搜索电影、图书、音乐".to_string(),
                engine: vec!["douban".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
