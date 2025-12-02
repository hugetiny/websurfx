//! The `huggingface` module handles fetching search results from Hugging Face.
//!
//! Based on SearXNG's huggingface.py implementation.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Hugging Face API response structures
#[derive(Deserialize, Debug)]
struct HuggingFaceModel {
    id: Option<String>,
    #[serde(rename = "modelId")]
    model_id: Option<String>,
    downloads: Option<u64>,
    likes: Option<u64>,
    pipeline_tag: Option<String>,
    tags: Option<Vec<String>>,
}

/// The Hugging Face search engine struct.
pub struct HuggingFace;

impl HuggingFace {
    /// Creates a new Hugging Face search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Hugging Face
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://huggingface.co/api/models?search={}&limit=10&offset={}&sort=downloads&direction=-1",
            encoded_query,
            offset
        )
    }

    /// Build headers for Hugging Face requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    /// Format download count
    fn format_downloads(downloads: u64) -> String {
        if downloads >= 1_000_000 {
            format!("{:.1}M downloads", downloads as f64 / 1_000_000.0)
        } else if downloads >= 1_000 {
            format!("{:.1}K downloads", downloads as f64 / 1_000.0)
        } else {
            format!("{} downloads", downloads)
        }
    }
}

#[async_trait::async_trait]
impl SearchEngine for HuggingFace {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page);
        let headers = self.build_headers(user_agent)?;

        let response_bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let models: Vec<HuggingFaceModel> = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        for model in models {
            let model_id = model.model_id.clone().or(model.id.clone()).unwrap_or_default();

            if model_id.is_empty() {
                continue;
            }

            let url = format!("https://huggingface.co/{}", model_id);

            let mut content_parts = Vec::new();

            if let Some(pipeline) = &model.pipeline_tag {
                content_parts.push(format!("[{}]", pipeline));
            }

            if let Some(downloads) = model.downloads {
                content_parts.push(Self::format_downloads(downloads));
            }

            if let Some(likes) = model.likes {
                content_parts.push(format!("❤️ {}", likes));
            }

            // Add some tags
            if let Some(tags) = &model.tags {
                let display_tags: Vec<&str> = tags.iter()
                    .filter(|t| !t.starts_with("arxiv:") && !t.starts_with("license:"))
                    .take(3)
                    .map(|s| s.as_str())
                    .collect();
                if !display_tags.is_empty() {
                    content_parts.push(display_tags.join(", "));
                }
            }

            let content = content_parts.join(" | ");

            let search_result = SearchResult::new(
                &model_id,
                &url,
                &content,
                &["HuggingFace"],
            );
            results.push((url.clone(), search_result));
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}
