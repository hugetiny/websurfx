//! eBay shopping search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// eBay search engine
pub struct Ebay;

impl Ebay {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://www.ebay.com/sch/i.html?_nkw={}&_pgn={}",
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
impl SearchEngine for Ebay {
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

        if let Ok(selector) = Selector::parse("li.s-item") {
            for element in document.select(&selector) {
                let title = element
                    .select(&Selector::parse("div.s-item__title span").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let href = element
                    .select(&Selector::parse("a.s-item__link").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .unwrap_or_default()
                    .to_string();

                let price = element
                    .select(&Selector::parse("span.s-item__price").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !href.is_empty() && !title.contains("Shop on eBay") {
                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description: format!("Price: {}", price),
                            engine: vec!["ebay".to_string()],
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
