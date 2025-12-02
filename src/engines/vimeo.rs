//! Vimeo video search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Vimeo search engine
pub struct Vimeo;

impl Vimeo {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let encoded_query = urlencoding::encode(query);
        format!("https://vimeo.com/search?q={}&page={}", encoded_query, page)
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
impl SearchEngine for Vimeo {
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

        if let Ok(selector) = Selector::parse("div.iris_video-vital") {
            for element in document.select(&selector) {
                let title = element
                    .select(&Selector::parse("a.iris_link-header").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let href = element
                    .select(&Selector::parse("a.iris_link-header").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .map(|h| {
                        if h.starts_with('/') {
                            format!("https://vimeo.com{}", h)
                        } else {
                            h.to_string()
                        }
                    })
                    .unwrap_or_default();

                let description = element
                    .select(&Selector::parse("p").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !href.is_empty() {
                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description,
                            engine: vec!["vimeo".to_string()],
                            relevance_score: 0.0,
                        },
                    ));
                }
            }
        }

        Ok(results)
    }
}
