//! JD.com Chinese shopping search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// JD search engine
pub struct Jd;

impl Jd {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://search.jd.com/Search?keyword={}&page={}",
            encoded_query, page * 2 - 1
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
impl SearchEngine for Jd {
    async fn results(
        &self,
        query: &str,
        page: u32,
        _user_agent: &str,
        _client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page);

        // JD requires complex JS rendering, so we return a search link
        let results = vec![(
            url.clone(),
            SearchResult {
                title: format!("京东搜索: {}", query),
                url,
                description: "点击在京东上搜索商品".to_string(),
                engine: vec!["jd".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
