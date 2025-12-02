//! OpenStreetMap search engine

use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use error_stack::{Report, Result};

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

#[derive(Deserialize)]
struct NominatimResult {
    #[allow(dead_code)]
    place_id: Option<i64>,
    display_name: Option<String>,
    lat: Option<String>,
    lon: Option<String>,
    #[serde(rename = "type")]
    place_type: Option<String>,
}

/// OpenStreetMap search engine
pub struct OpenStreetMap;

impl OpenStreetMap {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }

    fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=20",
            encoded_query
        )
    }

    fn build_headers(&self, user_agent: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        headers
    }
}

#[async_trait::async_trait]
impl SearchEngine for OpenStreetMap {
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

        let places: Vec<NominatimResult> = serde_json::from_slice(&bytes)
            .map_err(|_| Report::new(EngineError::UnexpectedError))?;

        let mut results = Vec::new();

        for place in places {
            let display_name = place.display_name.unwrap_or_default();
            let lat = place.lat.unwrap_or_default();
            let lon = place.lon.unwrap_or_default();
            let place_type = place.place_type.unwrap_or_default();

            if !display_name.is_empty() && !lat.is_empty() && !lon.is_empty() {
                let map_url = format!(
                    "https://www.openstreetmap.org/?mlat={}&mlon={}#map=15/{}/{}",
                    lat, lon, lat, lon
                );
                results.push((
                    map_url.clone(),
                    SearchResult {
                        title: display_name,
                        url: map_url,
                        description: format!("Type: {} | Coordinates: {}, {}", place_type, lat, lon),
                        engine: vec!["openstreetmap".to_string()],
                        relevance_score: 0.0,
                    },
                ));
            }
        }

        Ok(results)
    }
}
