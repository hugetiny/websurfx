//! Genius lyrics search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct GeniusResponse {
    response: Option<GeniusResponseInner>,
}

#[derive(Deserialize)]
struct GeniusResponseInner {
    hits: Option<Vec<GeniusHit>>,
}

#[derive(Deserialize)]
struct GeniusHit {
    result: Option<GeniusSong>,
}

#[derive(Deserialize)]
struct GeniusSong {
    title: Option<String>,
    url: Option<String>,
    primary_artist: Option<GeniusArtist>,
}

#[derive(Deserialize)]
struct GeniusArtist {
    name: Option<String>,
}

/// Genius search engine
pub struct Genius;

impl Genius {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!("https://genius.com/api/search/multi?q={}", encoded_query)
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for Genius {
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
        let bytes = self.fetch_json_as_bytes_from_upstream(&url, headers, client).await?;

        let response: GeniusResponse = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        if let Some(inner) = response.response {
            if let Some(hits) = inner.hits {
                for hit in hits {
                    if let Some(song) = hit.result {
                        let title = song.title.unwrap_or_default();
                        let url = song.url.unwrap_or_default();
                        let artist = song.primary_artist
                            .and_then(|a| a.name)
                            .unwrap_or_default();

                        if !title.is_empty() && !url.is_empty() {
                            results.push((
                                url.clone(),
                                SearchResult {
                                    title: format!("{} - {}", artist, title),
                                    url,
                                    description: format!("Lyrics by {}", artist),
                                    engine: vec!["genius".to_string()],
                                    relevance_score: 0.0,
                                },
                            ));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
