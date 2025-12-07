//! Zhihu Chinese Q&A search engine

use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Zhihu search engine
pub struct Zhihu;

impl Zhihu {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://www.zhihu.com/search?type=content&q={}", encoded_query)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Zhihu {
    async fn results(
        &self,
        query: &str,
        _page: u32,
        _user_agent: &str,
        _client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query);

        // Zhihu requires complex JS rendering
        let results = vec![(
            url.clone(),
            SearchResult {
                title: format!("知乎: {}", query),
                url,
                description: "在知乎搜索问答内容".to_string(),
                engine: vec!["zhihu".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
