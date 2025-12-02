//! Dailymotion video search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct DailymotionResponse {
    list: Option<Vec<DailymotionVideo>>,
}

#[derive(Deserialize)]
struct DailymotionVideo {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
}

/// Dailymotion search engine
pub struct Dailymotion;

impl Dailymotion {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://api.dailymotion.com/videos?search={}&page={}&limit=20&fields=id,title,description",
            encoded_query, page
        )
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Dailymotion {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page);
        let headers = self.build_headers(user_agent);
        let bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let response: DailymotionResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(videos) = response.list {
            for video in videos {
                let id = video.id.unwrap_or_default();
                let title = video.title.unwrap_or_default();
                let description = video.description.unwrap_or_default();

                if !id.is_empty() && !title.is_empty() {
                    let url = format!("https://www.dailymotion.com/video/{}", id);
                    results.push((
                        url.clone(),
                        SearchResult {
                            title,
                            url,
                            description,
                            engine: vec!["dailymotion".to_string()],
                            relevance_score: 0.0,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
