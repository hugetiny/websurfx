//! Bitbucket code search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct BitbucketResponse {
    values: Option<Vec<BitbucketRepo>>,
}

#[derive(Deserialize)]
struct BitbucketRepo {
    name: Option<String>,
    description: Option<String>,
    links: Option<BitbucketLinks>,
}

#[derive(Deserialize)]
struct BitbucketLinks {
    html: Option<BitbucketHref>,
}

#[derive(Deserialize)]
struct BitbucketHref {
    href: Option<String>,
}

/// Bitbucket search engine
pub struct Bitbucket;

impl Bitbucket {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://api.bitbucket.org/2.0/repositories?q=name~\"{}\"&page={}&pagelen=20",
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
impl SearchEngine for Bitbucket {
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

        let response: BitbucketResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(repos) = response.values {
            for repo in repos {
                let name = repo.name.unwrap_or_default();
                let html_url = repo.links
                    .and_then(|l| l.html)
                    .and_then(|h| h.href)
                    .unwrap_or_default();
                let description = repo.description.unwrap_or_default();

                if !name.is_empty() && !html_url.is_empty() {
                    results.push((
                        html_url.clone(),
                        SearchResult {
                            title: name,
                            url: html_url,
                            description,
                            engine: vec!["bitbucket".to_string()],
                            relevance_score: 0.0,
                            ..Default::default()
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
