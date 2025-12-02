//! Flickr image search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Flickr search engine
pub struct Flickr;

impl Flickr {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!("https://www.flickr.com/search/?text={}&page={}", encoded_query, page)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Flickr {
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

        if let Ok(selector) = Selector::parse("div.photo-list-photo-view") {
            for element in document.select(&selector) {
                let _href = element
                    .value()
                    .attr("style")
                    .and_then(|s| {
                        s.split("url(").nth(1)
                            .and_then(|u| u.split(')').next())
                    })
                    .unwrap_or_default();

                let title = element
                    .value()
                    .attr("title")
                    .unwrap_or("Flickr Image")
                    .to_string();

                let photo_url = element
                    .select(&Selector::parse("a").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .map(|h| format!("https://www.flickr.com{}", h))
                    .unwrap_or_default();

                if !photo_url.is_empty() {
                    results.push((
                        photo_url.clone(),
                        SearchResult {
                            title,
                            url: photo_url,
                            description: String::new(),
                            engine: vec!["flickr".to_string()],
                            relevance_score: 0.0,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
