//! Google Scholar academic search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Google Scholar search engine
pub struct GoogleScholar;

impl GoogleScholar {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str, page: u32) -> String {
        let page = page.max(1);
        let start = (page - 1) * 10;
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://scholar.google.com/scholar?q={}&start={}&hl=en",
            encoded_query, start
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
impl SearchEngine for GoogleScholar {
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

        if let Ok(selector) = Selector::parse("div.gs_ri") {
            for element in document.select(&selector) {
                let title = element
                    .select(&Selector::parse("h3.gs_rt a").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let href = element
                    .select(&Selector::parse("h3.gs_rt a").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .unwrap_or_default()
                    .to_string();

                let description = element
                    .select(&Selector::parse("div.gs_rs").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let info = element
                    .select(&Selector::parse("div.gs_a").unwrap())
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !href.is_empty() {
                    results.push((
                        href.clone(),
                        SearchResult {
                            title,
                            url: href,
                            description: format!("{} - {}", info, description),
                            engine: vec!["googlescholar".to_string()],
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
