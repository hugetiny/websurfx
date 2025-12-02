//! Lemmy search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct LemmyResponse {
    posts: Option<Vec<LemmyPostWrapper>>,
}

#[derive(Deserialize)]
struct LemmyPostWrapper {
    post: Option<LemmyPost>,
    community: Option<LemmyCommunity>,
}

#[derive(Deserialize)]
struct LemmyPost {
    #[allow(dead_code)]
    id: Option<i64>,
    name: Option<String>,
    body: Option<String>,
    url: Option<String>,
    ap_id: Option<String>,
}

#[derive(Deserialize)]
struct LemmyCommunity {
    name: Option<String>,
}

/// Lemmy search engine
pub struct Lemmy;

impl Lemmy {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://lemmy.ml/api/v3/search?q={}&page={}&limit=20&type_=Posts",
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
impl SearchEngine for Lemmy {
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

        let response: LemmyResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(posts) = response.posts {
            for wrapper in posts {
                if let Some(post) = wrapper.post {
                    let title = post.name.unwrap_or_default();
                    let body = post.body.unwrap_or_default();
                    let post_url = post.ap_id.or(post.url).unwrap_or_default();
                    let community = wrapper.community
                        .and_then(|c| c.name)
                        .unwrap_or_default();

                    if !title.is_empty() && !post_url.is_empty() {
                        results.push((
                            post_url.clone(),
                            SearchResult {
                                title,
                                url: post_url,
                                description: format!("[{}] {}", community,
                                    if body.len() > 200 { format!("{}...", &body[..200]) } else { body }
                                ),
                                engine: vec!["lemmy".to_string()],
                                relevance_score: 0.0,
                            },
                        ));
                    }
                }
            }
        }

        Ok(results)
    }
}
