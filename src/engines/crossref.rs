//! CrossRef academic search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct CrossRefResponse {
    message: Option<CrossRefMessage>,
}

#[derive(Deserialize)]
struct CrossRefMessage {
    items: Option<Vec<CrossRefItem>>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct CrossRefItem {
    DOI: Option<String>,
    title: Option<Vec<String>>,
    #[serde(rename = "abstract")]
    item_abstract: Option<String>,
    published: Option<CrossRefDate>,
}

#[derive(Deserialize)]
struct CrossRefDate {
    #[serde(rename = "date-parts")]
    date_parts: Option<Vec<Vec<i32>>>,
}

/// CrossRef search engine
pub struct CrossRef;

impl CrossRef {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 20;
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://api.crossref.org/works?query={}&offset={}&rows=20",
            encoded_query, offset
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
impl SearchEngine for CrossRef {
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

        let response: CrossRefResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(message) = response.message {
            if let Some(items) = message.items {
                for item in items {
                    let doi = item.DOI.unwrap_or_default();
                    let title = item.title
                        .and_then(|t| t.first().cloned())
                        .unwrap_or_default();
                    let item_abstract = item.item_abstract.unwrap_or_default();
                    let year = item.published
                        .and_then(|p| p.date_parts)
                        .and_then(|d| d.first().cloned())
                        .and_then(|y| y.first().cloned())
                        .unwrap_or(0);

                    if !doi.is_empty() && !title.is_empty() {
                        let doi_url = format!("https://doi.org/{}", doi);
                        let desc = if year > 0 && !item_abstract.is_empty() {
                            format!("[{}] {}", year,
                                if item_abstract.len() > 200 {
                                    format!("{}...", &item_abstract[..200])
                                } else {
                                    item_abstract
                                })
                        } else if year > 0 {
                            format!("[{}] DOI: {}", year, doi)
                        } else {
                            format!("DOI: {}", doi)
                        };

                        results.push((
                            doi_url.clone(),
                            SearchResult {
                                title,
                                url: doi_url,
                                description: desc,
                                engine: vec!["crossref".to_string()],
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
