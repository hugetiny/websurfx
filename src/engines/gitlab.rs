//! GitLab code search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct GitLabProject {
    #[allow(dead_code)]
    id: Option<i64>,
    name: Option<String>,
    description: Option<String>,
    web_url: Option<String>,
    star_count: Option<i64>,
}

/// GitLab search engine
pub struct GitLab;

impl GitLab {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://gitlab.com/api/v4/projects?search={}&page={}&per_page=20&order_by=stars",
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
impl SearchEngine for GitLab {
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

        let projects: Vec<GitLabProject> = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        for project in projects {
            let name = project.name.unwrap_or_default();
            let web_url = project.web_url.unwrap_or_default();
            let description = project.description.unwrap_or_default();
            let stars = project.star_count.unwrap_or(0);

            if !name.is_empty() && !web_url.is_empty() {
                results.push((
                    web_url.clone(),
                    SearchResult {
                        title: format!("{} ⭐{}", name, stars),
                        url: web_url,
                        description,
                        engine: vec!["gitlab".to_string()],
                        relevance_score: 0.0,
                    },
                ));
            }
        }

        Ok(results)
    }
}
