//! Rottentomatoes movie reviews

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{Html, Selector};
use error_stack::Result;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

/// Rotten Tomatoes search engine
pub struct RottenTomatoes;

impl RottenTomatoes {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://www.rottentomatoes.com/search?search={}", encoded_query)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for RottenTomatoes {
    async fn results(
        &self,
        query: &str,
        _page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let url = self.build_url(query);
        let headers = self.build_headers(user_agent);
        let html = self.fetch_html_from_upstream(&url, headers, client).await?;

        let document = Html::parse_document(&html);
        let mut results = Vec::new();

        if let Ok(selector) = Selector::parse("search-page-media-row") {
            for element in document.select(&selector) {
                let _title = element
                    .value()
                    .attr("releaseyear")
                    .map(|y| format!("{} ({})",
                        element.value().attr("tomatometerscore").unwrap_or(""),
                        y
                    ))
                    .unwrap_or_default();

                // Fallback to basic search result
                if results.is_empty() {
                    results.push((
                        url.clone(),
                        SearchResult {
                            title: format!("Rotten Tomatoes: {}", query),
                            url: url.clone(),
                            description: "Search movies and TV shows on Rotten Tomatoes".to_string(),
                            engine: vec!["rottentomatoes".to_string()],
                            relevance_score: 0.0,
                            ..Default::default()
                        },
                    ));
                }
            }
        }

        if results.is_empty() {
            results.push((
                url.clone(),
                SearchResult {
                    title: format!("Rotten Tomatoes: {}", query),
                    url,
                    description: "Search movies and TV shows on Rotten Tomatoes".to_string(),
                    engine: vec!["rottentomatoes".to_string()],
                    relevance_score: 0.0,
                    ..Default::default()
                },
            ));
        }

        Ok(results)
    }
}
