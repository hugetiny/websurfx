//! The `qwant` module handles fetching search results from the Qwant search engine.
//!
//! Based on SearXNG's qwant.py implementation.
//! Qwant is a privacy-focused search engine based in France.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Qwant API response structures
#[derive(Deserialize, Debug)]
struct QwantResponse {
    status: String,
    data: Option<QwantData>,
}

#[derive(Deserialize, Debug)]
struct QwantData {
    result: Option<QwantResult>,
}

#[derive(Deserialize, Debug)]
struct QwantResult {
    items: Option<QwantItems>,
}

#[derive(Deserialize, Debug)]
struct QwantItems {
    mainline: Option<Vec<QwantMainline>>,
}

#[derive(Deserialize, Debug)]
struct QwantMainline {
    #[serde(rename = "type")]
    result_type: String,
    items: Option<Vec<QwantItem>>,
}

#[derive(Deserialize, Debug)]
struct QwantItem {
    title: Option<String>,
    url: Option<String>,
    desc: Option<String>,
}

/// The Qwant search engine struct.
pub struct Qwant {
    locale: String,
}

impl Qwant {
    /// Creates a new Qwant search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            locale: "en_US".to_string(),
        })
    }

    /// Build the search URL for Qwant
    fn build_url(&self, query: &str, page: u32, safe_search: u8) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://api.qwant.com/v3/search/web?q={}&count=10&locale={}&offset={}&safesearch={}",
            encoded_query,
            self.locale,
            offset,
            safe_search
        )
    }

    /// Build headers for Qwant requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        Ok(headers)
    }
}

#[async_trait::async_trait]
impl SearchEngine for Qwant {
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query, page, safe_search);
        let headers = self.build_headers(user_agent)?;

        let response_bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let response: QwantResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        if response.status != "success" {
            return Err(Report::new(EngineError::RequestError));
        }

        let mut results = Vec::new();

        if let Some(data) = response.data {
            if let Some(result) = data.result {
                if let Some(items) = result.items {
                    if let Some(mainline) = items.mainline {
                        for line in mainline {
                            if line.result_type == "web" {
                                if let Some(web_items) = line.items {
                                    for item in web_items {
                                        if let (Some(title), Some(url)) = (&item.title, &item.url) {
                                            let content = item.desc.clone().unwrap_or_default();
                                            let search_result = SearchResult::new(
                                                title,
                                                url,
                                                &content,
                                                &["Qwant"],
                                            );
                                            results.push((url.clone(), search_result));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}
