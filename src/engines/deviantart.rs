//! DeviantArt art search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// DeviantArt search engine
pub struct DeviantArt;

impl DeviantArt {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let offset = (page - 1) * 24;
        let encoded_query = urlencoding::encode(query);
        format!("https://www.deviantart.com/search?q={}&offset={}", encoded_query, offset)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for DeviantArt {
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
        let html = self.fetch_html_from_upstream(&url, headers, client).await?;

        let document = Html::parse_document(&html);
        let mut results = Vec::new();

        if let Ok(selector) = Selector::parse("a[data-hook='deviation_link']") {
            for element in document.select(&selector) {
                let href = element
                    .value()
                    .attr("href")
                    .unwrap_or_default()
                    .to_string();

                let title = element
                    .value()
                    .attr("title")
                    .or_else(|| element.value().attr("aria-label"))
                    .unwrap_or("DeviantArt Artwork")
                    .to_string();

                if !href.is_empty() {
                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description: "Artwork from DeviantArt".to_string(),
                            engine: vec!["deviantart".to_string()],
                            relevance_score: 0.0,
                            ..Default::default()
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
