//! The `youtube` module handles fetching search results from YouTube (via Invidious).
//!
//! Based on SearXNG's invidious.py and youtube_noapi.py implementations.
//! Uses Invidious instances to avoid YouTube API restrictions.

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// Invidious API response structures
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum InvidiousResult {
    Video(InvidiousVideo),
    #[allow(dead_code)]
    Other(serde_json::Value),
}

#[derive(Deserialize, Debug)]
struct InvidiousVideo {
    #[serde(rename = "type")]
    result_type: Option<String>,
    title: Option<String>,
    #[serde(rename = "videoId")]
    video_id: Option<String>,
    author: Option<String>,
    #[serde(rename = "viewCount")]
    view_count: Option<u64>,
    #[serde(rename = "lengthSeconds")]
    length_seconds: Option<u64>,
    description: Option<String>,
    #[serde(rename = "publishedText")]
    published_text: Option<String>,
}

/// The YouTube search engine struct.
pub struct YouTube {
    invidious_instance: String,
}

impl YouTube {
    /// Creates a new YouTube search engine instance.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            // Use a reliable Invidious instance
            invidious_instance: "https://vid.puffyan.us".to_string(),
        })
    }

    /// Build the search URL for YouTube via Invidious
    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);

        format!(
            "{}/api/v1/search?q={}&page={}&type=video",
            self.invidious_instance,
            encoded_query,
            page
        )
    }

    /// Build headers for YouTube requests
    fn build_headers(&self, user_agent: &str) -> Result<HeaderMap, EngineError> {
        use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
            .change_context(EngineError::UnexpectedError)?);

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    /// Format video duration
    fn format_duration(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{}:{:02}", minutes, secs)
        }
    }

    /// Format view count
    fn format_views(views: u64) -> String {
        if views >= 1_000_000 {
            format!("{:.1}M views", views as f64 / 1_000_000.0)
        } else if views >= 1_000 {
            format!("{:.1}K views", views as f64 / 1_000.0)
        } else {
            format!("{} views", views)
        }
    }
}

#[async_trait::async_trait]
impl SearchEngine for YouTube {
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

        let response: Vec<InvidiousResult> = serde_json::from_slice(&response_bytes)
            .change_context(EngineError::UnexpectedError)?;

        let mut results = Vec::new();

        for item in response {
            if let InvidiousResult::Video(video) = item {
                // Only process video results
                if video.result_type.as_deref() != Some("video") {
                    continue;
                }

                let title = video.title.clone().unwrap_or_default();
                let video_id = video.video_id.clone().unwrap_or_default();

                if title.is_empty() || video_id.is_empty() {
                    continue;
                }

                let url = format!("https://www.youtube.com/watch?v={}", video_id);

                let mut content_parts = Vec::new();

                if let Some(author) = &video.author {
                    content_parts.push(author.clone());
                }
                if let Some(views) = video.view_count {
                    content_parts.push(Self::format_views(views));
                }
                if let Some(length) = video.length_seconds {
                    content_parts.push(Self::format_duration(length));
                }
                if let Some(published) = &video.published_text {
                    content_parts.push(published.clone());
                }

                // Add description snippet
                if let Some(desc) = &video.description {
                    if !desc.is_empty() {
                        let snippet = if desc.len() > 100 {
                            format!("{}...", &desc[..100])
                        } else {
                            desc.clone()
                        };
                        content_parts.push(snippet);
                    }
                }

                let content = content_parts.join(" | ");

                let search_result = SearchResult::new(
                    &title,
                    &url,
                    &content,
                    &["YouTube"],
                );
                results.push((url.clone(), search_result));
            }
        }

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}
