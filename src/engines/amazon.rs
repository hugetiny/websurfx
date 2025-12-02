//! Amazon shopping search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Amazon search engine
pub struct Amazon;

impl Amazon {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://www.amazon.com/s?k={}&page={}",
            encoded_query, page
        )
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Amazon {
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

        if let Ok(selector) = Selector::parse("div[data-component-type='s-search-result']") {
            for element in document.select(&selector) {
                let title = element
                    .select(&Selector::parse("h2 span").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let asin = element.value().attr("data-asin").unwrap_or_default();
                let href = if !asin.is_empty() {
                    format!("https://www.amazon.com/dp/{}", asin)
                } else {
                    String::new()
                };

                let price = element
                    .select(&Selector::parse("span.a-price span.a-offscreen").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let rating = element
                    .select(&Selector::parse("span.a-icon-alt").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !href.is_empty() {
                    let description = if !price.is_empty() && !rating.is_empty() {
                        format!("{} - {}", price, rating)
                    } else if !price.is_empty() {
                        price
                    } else {
                        "Amazon product".to_string()
                    };

                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description,
                            engine: vec!["amazon".to_string()],
                            relevance_score: 0.0,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
