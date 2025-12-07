//! Internet Archive search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct ArchiveResponse {
    response: Option<ArchiveResponseInner>,
}

#[derive(Deserialize)]
struct ArchiveResponseInner {
    docs: Option<Vec<ArchiveDoc>>,
}

#[derive(Deserialize)]
struct ArchiveDoc {
    identifier: Option<String>,
    title: Option<String>,
    description: Option<Vec<String>>,
    mediatype: Option<String>,
}

/// Internet Archive search engine
pub struct InternetArchive;

impl InternetArchive {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://archive.org/advancedsearch.php?q={}&output=json&rows=20&page={}",
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
impl SearchEngine for InternetArchive {
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

        let response: ArchiveResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(inner) = response.response {
            if let Some(docs) = inner.docs {
                for doc in docs {
                    let identifier = doc.identifier.unwrap_or_default();
                    let title = doc.title.unwrap_or_default();
                    let description = doc.description
                        .and_then(|d| d.first().cloned())
                        .unwrap_or_default();
                    let mediatype = doc.mediatype.unwrap_or_default();

                    if !identifier.is_empty() && !title.is_empty() {
                        let archive_url = format!("https://archive.org/details/{}", identifier);
                        results.push((
                            archive_url.clone(),
                            SearchResult {
                                title,
                                url: archive_url,
                                description: format!("[{}] {}", mediatype, description),
                                engine: vec!["internetarchive".to_string()],
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
