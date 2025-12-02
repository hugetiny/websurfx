//! This module provides the memory cache structures for enabling the use of memory caching.

use super::error::CacheError;
use crate::models::aggregation::SearchResults;
use crate::parser::Config;
use error_stack::Report;
use futures::future::join_all;
use moka::future::Cache as MokaCache;
use std::convert::TryInto;
use std::sync::Arc;
use tokio::time::Duration;

/// Memory based cache backend.
pub struct InMemoryCache {
    /// The backend cache which stores data.
    cache: Arc<MokaCache<String, Vec<u8>>>,
}

impl Clone for InMemoryCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
        }
    }
}

impl InMemoryCache {
    /// Builds a new InMemoryCache instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration struct containing cache settings.
    ///
    /// # Returns
    ///
    /// Returns a new InMemoryCache instance.
    pub async fn build(config: &Config) -> Self {
        log::info!("Initialising in-memory cache with TTL {} seconds", config.cache_expiry_time);

        InMemoryCache {
            cache: Arc::new(
                MokaCache::builder()
                    .time_to_live(Duration::from_secs(config.cache_expiry_time.into()))
                    .build(),
            ),
        }
    }

    /// Checks if cached results exist for the given URLs.
    pub async fn cached_results_exists(
        &mut self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>> {
        Ok(urls
            .iter()
            .map(|url| self.cache.contains_key(url))
            .collect())
    }

    /// Fetches cached results for the given URL.
    pub async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        if let Some(res) = self.cache.get(url).await {
            let search_results: SearchResults = res.try_into()?;
            return Ok(search_results);
        }

        Err(Report::new(CacheError::MissingValue))
    }

    /// Caches search results with their corresponding URLs.
    pub async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        let mut tasks: Vec<_> = Vec::with_capacity(urls.len());

        for (hashed_url_string, search_result) in urls.iter().cloned().zip(search_results.iter()) {
            let bytes: Vec<u8> = bincode::serialize(search_result)
                .map_err(|_| CacheError::SerializationError)?;
            let new_self = self.clone();
            tasks.push(tokio::spawn(async move {
                new_self.cache.insert(hashed_url_string, bytes).await
            }));
        }

        join_all(tasks).await;

        Ok(())
    }
}
