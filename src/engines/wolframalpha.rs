//! Wolfram Alpha search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Wolfram Alpha search engine
pub struct WolframAlpha;

impl WolframAlpha {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://www.wolframalpha.com/input?i={}", encoded_query)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for WolframAlpha {
    async fn results(
        &self,
        query: &str,
        _page: u32,
        user_agent: &str,
        _client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query);
        let _headers = self.build_headers(user_agent);

        // For WolframAlpha, we just return a link to the query
        // as parsing the dynamic content is complex
        let results = vec![(
            url.clone(),
            SearchResult {
                title: format!("Wolfram|Alpha: {}", query),
                url: url.clone(),
                description: "Computational knowledge engine - click to see detailed results".to_string(),
                engine: vec!["wolframalpha".to_string()],
                relevance_score: 0.0,
                ..Default::default()
            },
        )];

        Ok(results)
    }
}
