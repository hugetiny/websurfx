//! Bandcamp music search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Bandcamp search engine
pub struct Bandcamp;

impl Bandcamp {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!("https://bandcamp.com/search?q={}&page={}", encoded_query, page)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Bandcamp {
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

        if let Ok(selector) = Selector::parse("li.searchresult") {
            for element in document.select(&selector) {
                let title = element
                    .select(&Selector::parse("div.heading a").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let href = element
                    .select(&Selector::parse("div.heading a").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .unwrap_or_default()
                    .to_string();

                let artist = element
                    .select(&Selector::parse("div.subhead").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !href.is_empty() {
                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description: artist,
                            engine: vec!["bandcamp".to_string()],
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
