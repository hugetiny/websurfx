//! The `pubmed` module handles fetching search results from PubMed.
//!
//! Based on SearXNG's pubmed.py implementation.
//! PubMed is a free search engine accessing primarily the MEDLINE database.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// PubMed API response structures
#[derive(Deserialize, Debug)]
struct PubMedSearchResponse {
    esearchresult: Option<ESearchResult>,
}

#[derive(Deserialize, Debug)]
struct ESearchResult {
    idlist: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct PubMedSummaryResponse {
    result: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// The PubMed search engine struct.
pub struct PubMed;

impl PubMed {
    /// Creates a new PubMed search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for PubMed
    fn build_search_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let retstart = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term={}&retstart={}&retmax=10&retmode=json",
            encoded_query,
            retstart
        )
    }

    /// Build the summary URL for fetching article details
    fn build_summary_url(&self, ids: &[String]) -> String {
        format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&id={}&retmode=json",
            ids.join(",")
        )
    }

    /// Build headers for PubMed requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Ok(headers)
    }
}

#[async_trait::async_trait]
impl SearchEngine for PubMed {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let search_url = self.build_search_url(query, page);
        let headers = self.build_headers(user_agent)?;

        // First, search for article IDs
        let search_bytes = self.fetch_json_as_bytes_from_upstream(&search_url, headers.clone(), client).await?;

        let search_response: PubMedSearchResponse = serde_json::from_slice(&search_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let ids = search_response
            .esearchresult
            .and_then(|r| r.idlist)
            .unwrap_or_default();

        if ids.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // Then, fetch article summaries
        let summary_url = self.build_summary_url(&ids);
        let summary_bytes = self.fetch_json_as_bytes_from_upstream(&summary_url, headers, client).await?;

        let summary_response: PubMedSummaryResponse = serde_json::from_slice(&summary_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        if let Some(result_map) = summary_response.result {
            for id in &ids {
                if let Some(article) = result_map.get(id) {
                    let title = article.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if title.is_empty() {
                        continue;
                    }

                    let url = format!("https://pubmed.ncbi.nlm.nih.gov/{}/", id);

                    let mut content_parts = Vec::new();

                    // Authors
                    if let Some(authors) = article.get("authors").and_then(|v| v.as_array()) {
                        let author_names: Vec<&str> = authors
                            .iter()
                            .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                            .take(3)
                            .collect();
                        if !author_names.is_empty() {
                            content_parts.push(format!("Authors: {}", author_names.join(", ")));
                        }
                    }

                    // Source/Journal
                    if let Some(source) = article.get("source").and_then(|v| v.as_str()) {
                        content_parts.push(source.to_string());
                    }

                    // Publication date
                    if let Some(pubdate) = article.get("pubdate").and_then(|v| v.as_str()) {
                        content_parts.push(pubdate.to_string());
                    }

                    let content = content_parts.join(" | ");

                    let search_result = SearchResult::new(
                        &title,
                        &url,
                        &content,
                        &["PubMed"],
                    );
                    results.push((url.clone(), search_result));
                }
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}
