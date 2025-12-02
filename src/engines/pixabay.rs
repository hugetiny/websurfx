//! Pixabay image search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Pixabay search engine
pub struct Pixabay;

impl Pixabay {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!("https://pixabay.com/images/search/{}/?pagi={}", encoded_query, page)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Pixabay {
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

        if let Ok(selector) = Selector::parse("div.item a") {
            for element in document.select(&selector) {
                let href = element
                    .value()
                    .attr("href")
                    .map(|h| {
                        if h.starts_with('/') {
                            format!("https://pixabay.com{}", h)
                        } else {
                            h.to_string()
                        }
                    })
                    .unwrap_or_default();

                let title = element
                    .select(&Selector::parse("img").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("alt"))
                    .unwrap_or("Pixabay Image")
                    .to_string();

                if !href.is_empty() {
                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description: "Free image from Pixabay".to_string(),
                            engine: vec!["pixabay".to_string()],
                            relevance_score: 0.0,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
