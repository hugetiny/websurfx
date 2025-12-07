//! Taobao Chinese shopping search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Taobao search engine
pub struct Taobao;

impl Taobao {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://s.taobao.com/search?q={}&s={}",
            encoded_query, (page - 1) * 44
        )
    }

    #[allow(dead_code)]
    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Taobao {
    async fn results(
        &self,
        query: &str,
        page: u32,
        _user_agent: &str,
        _client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page);

        // Taobao requires login for API access, so we return a search link
        let results = vec![(
            url.clone(),
            SearchResult {
                title: format!("淘宝搜索: {}", query),
                url,
                description: "点击在淘宝上搜索商品".to_string(),
                engine: vec!["taobao".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
