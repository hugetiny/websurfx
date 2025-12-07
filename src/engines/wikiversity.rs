//! Wikiversity search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct WikiResponse {
    query: Option<WikiQuery>,
}

#[derive(Deserialize)]
struct WikiQuery {
    search: Option<Vec<WikiSearchResult>>,
}

#[derive(Deserialize)]
struct WikiSearchResult {
    title: Option<String>,
    snippet: Option<String>,
}

/// Wikiversity search engine
pub struct Wikiversity;

impl Wikiversity {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 20;
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://en.wikiversity.org/w/api.php?action=query&list=search&srsearch={}&format=json&sroffset={}&srlimit=20",
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
impl SearchEngine for Wikiversity {
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

        let response: WikiResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(q) = response.query {
            if let Some(search_results) = q.search {
                for result in search_results {
                    let title = result.title.unwrap_or_default();
                    let snippet = result.snippet.unwrap_or_default();
                    let snippet_clean = html_escape::decode_html_entities(&snippet)
                        .to_string()
                        .replace("<span class=\"searchmatch\">", "")
                        .replace("</span>", "");

                    if !title.is_empty() {
                        let encoded_title = urlencoding::encode(&title);
                        let wiki_url = format!("https://en.wikiversity.org/wiki/{}", encoded_title);
                        results.push((
                            wiki_url.clone(),
                            SearchResult {
                                title,
                                url: wiki_url,
                                description: snippet_clean,
                                engine: vec!["wikiversity".to_string()],
                                relevance_score: 0.0,
                                ..Default::default()
                            },
                        ));
                    }
                }
            }
        }

        Ok(results)
    }
}
