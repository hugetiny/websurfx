//! Semantic Scholar academic search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct SemanticResponse {
    data: Option<Vec<SemanticPaper>>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct SemanticPaper {
    paperId: Option<String>,
    title: Option<String>,
    #[serde(rename = "abstract")]
    paper_abstract: Option<String>,
    year: Option<i32>,
    citationCount: Option<i32>,
}

/// Semantic Scholar search engine
pub struct SemanticScholar;

impl SemanticScholar {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 20;
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://api.semanticscholar.org/graph/v1/paper/search?query={}&offset={}&limit=20&fields=paperId,title,abstract,year,citationCount",
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
impl SearchEngine for SemanticScholar {
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

        let response: SemanticResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(papers) = response.data {
            for paper in papers {
                let paper_id = paper.paperId.unwrap_or_default();
                let title = paper.title.unwrap_or_default();
                let paper_abstract = paper.paper_abstract.unwrap_or_default();
                let year = paper.year.unwrap_or(0);
                let citations = paper.citationCount.unwrap_or(0);

                if !paper_id.is_empty() && !title.is_empty() {
                    let paper_url = format!("https://www.semanticscholar.org/paper/{}", paper_id);
                    let desc = if year > 0 {
                        format!("[{}] {} citations - {}", year, citations,
                            if paper_abstract.len() > 200 {
                                format!("{}...", &paper_abstract[..200])
                            } else {
                                paper_abstract
                            })
                    } else {
                        paper_abstract
                    };

                    results.push((
                        paper_url.clone(),
                        SearchResult {
                            title,
                            url: paper_url,
                            description: desc,
                            engine: vec!["semanticscholar".to_string()],
                            relevance_score: 0.0,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
