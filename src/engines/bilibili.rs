//! The `bilibili` module handles fetching search results from Bilibili.
//!
//! Based on SearXNG's bilibili.py implementation.
//! Bilibili is the leading video platform in China.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Bilibili API response structures
#[derive(Deserialize, Debug)]
struct BilibiliResponse {
    code: Option<i32>,
    data: Option<BilibiliData>,
}

#[derive(Deserialize, Debug)]
struct BilibiliData {
    result: Option<Vec<BilibiliResult>>,
}

#[derive(Deserialize, Debug)]
struct BilibiliResult {
    title: Option<String>,
    arcurl: Option<String>,
    author: Option<String>,
    play: Option<u64>,
    description: Option<String>,
    duration: Option<String>,
    #[allow(dead_code)]
    pubdate: Option<i64>,
}

/// The Bilibili search engine struct.
pub struct Bilibili;

impl Bilibili {
    /// Creates a new Bilibili search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    /// Build the search URL for Bilibili
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "https://api.bilibili.com/x/web-interface/search/type?keyword={}&search_type=video&page={}",
            encoded_query,
            page
        )
    }

    /// Build headers for Bilibili requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));
        headers.insert(REFERER, HeaderValue::from_static("https://www.bilibili.com/"));

        Ok(headers)
    }

    /// Format play count
    fn format_plays(plays: u64) -> String {
        if plays >= 100_000_000 {
            format!("{:.1}亿播放", plays as f64 / 100_000_000.0)
        } else if plays >= 10_000 {
            format!("{:.1}万播放", plays as f64 / 10_000.0)
        } else {
            format!("{}播放", plays)
        }
    }

    /// Clean HTML tags from title
    fn clean_html(text: &str) -> String {
        text.replace("<em class=\"keyword\">", "")
            .replace("</em>", "")
    }
}

#[async_trait::async_trait]
impl SearchEngine for Bilibili {
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

        let response: BilibiliResponse = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        if response.code != Some(0) {
            return Err(Report::new(EngineError::RequestError));
        }

        let mut results = Vec::new();

        if let Some(data) = response.data {
            if let Some(result_list) = data.result {
                for item in result_list {
                    let title = item.title.clone().unwrap_or_default();
                    let url = item.arcurl.clone().unwrap_or_default();

                    if title.is_empty() || url.is_empty() {
                        continue;
                    }

                    let clean_title = Self::clean_html(&title);

                    let mut content_parts = Vec::new();

                    if let Some(author) = &item.author {
                        content_parts.push(format!("UP: {}", author));
                    }
                    if let Some(play) = item.play {
                        content_parts.push(Self::format_plays(play));
                    }
                    if let Some(duration) = &item.duration {
                        content_parts.push(duration.clone());
                    }

                    // Add description snippet
                    if let Some(desc) = &item.description {
                        if !desc.is_empty() {
                            let clean_desc = Self::clean_html(desc);
                            let snippet = if clean_desc.len() > 100 {
                                format!("{}...", &clean_desc[..100])
                            } else {
                                clean_desc
                            };
                            content_parts.push(snippet);
                        }
                    }

                    let content = content_parts.join(" | ");

                    let search_result = SearchResult::new(
                        &clean_title,
                        &url,
                        &content,
                        &["Bilibili"],
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
