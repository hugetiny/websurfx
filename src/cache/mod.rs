//! This module provides the modules which provide the functionality to cache the aggregated
//! results fetched and aggregated from the upstream search engines in a json format.

use crate::models::aggregation::SearchResults;
use crate::models::parser::CacheBackend;
use crate::parser::Config;
use arc_swap::ArcSwap;
use error::CacheError;
use error_stack::Report;
use memory::InMemoryCache;
use redis::RedisCache;
use std::convert::TryInto;
use std::sync::Arc;

pub mod error;
pub mod memory;
pub mod redis;

#[cfg(any(feature = "encrypt-cache-results", feature = "cec-cache-results"))]
/// encryption module contains encryption utils such the cipher and key
pub mod encryption;

#[cfg(any(feature = "encrypt-cache-results", feature = "cec-cache-results"))]
use encryption::*;

/// Abstraction trait for common methods provided by a cache backend.
#[allow(dead_code)]
#[async_trait::async_trait]
trait Cacher: Send + Sync {
    // A function that builds the cache from the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - It takes the config struct as an argument.
    ///
    /// # Returns
    ///
    /// It returns a newly initialized backend based on the feature enabled by the user.
    async fn build(config: &Config) -> Self
    where
        Self: Sized;

    /// A function which fetches the cached json results as json string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    ///
    /// # Error
    ///
    /// Returns the `SearchResults` from the cache if the program executes normally otherwise
    /// returns a `CacheError` if the results cannot be retrieved from the cache.
    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>>;

    /// A function that checks if the cached results exists in the cache server or not.
    ///
    /// # Arguments
    ///
    /// * `urls` - It takes the hashed search urls as an argument which will be used as the key to
    ///   check whether the cache exists or not.
    ///
    /// # Error
    ///
    /// Returns a Vector containing booleans for each respective url if nothing goes wrong otherwise
    /// returns a `CacheError`.
    async fn cached_results_exists(
        &mut self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>>;

    /// A function which caches the results by using the `url` as the key and
    /// `json results` as the value and stores it in the cache
    ///
    /// # Arguments
    ///
    /// * `json_results` - It takes the json results string as an argument.
    /// * `url` - It takes the url as a String.
    ///
    /// # Error
    ///
    /// Returns a unit type if the program caches the given search results without a failure
    /// otherwise it returns a `CacheError` if the search results cannot be cached due to a
    /// failure.
    async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>>;

    /// A helper function that returns  either encrypted or decrypted results.
    ///  Feature flags (**encrypt-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    /// * `encrypt` - A boolean to choose whether to encrypt or decrypt the bytes
    ///
    /// # Error
    ///
    /// Returns  either encrypted or decrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(
      //  feature = "compress-cache-results",
        feature = "encrypt-cache-results",
        feature = "cec-cache-results"
    ))]
    async fn encrypt_or_decrypt_results(
        &mut self,
        mut bytes: Vec<u8>,
        encrypt: bool,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        use chacha20poly1305::{
            ChaCha20Poly1305,
            aead::{Aead, AeadCore, KeyInit, OsRng},
        };

        Ok(
            tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Report<CacheError>> {
                let cipher = CIPHER.get_or_init(|| {
                    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                    ChaCha20Poly1305::new(&key)
                });

                let encryption_key = ENCRYPTION_KEY.get_or_init(
                    || ChaCha20Poly1305::generate_nonce(&mut OsRng), // 96-bits; unique per message
                );

                bytes = if encrypt {
                    cipher
                        .encrypt(encryption_key, bytes.as_ref())
                        .map_err(|_| CacheError::EncryptionError)?
                } else {
                    cipher
                        .decrypt(encryption_key, bytes.as_ref())
                        .map_err(|_| CacheError::EncryptionError)?
                };

                Ok(bytes)
            })
            .await
            .map_err(|_| CacheError::EncryptionError)??,
        )
    }

    /// A helper function that returns compressed results.
    /// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the compressed bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
    async fn compress_results(
        &mut self,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        use tokio::io::AsyncWriteExt;
        let mut writer = async_compression::tokio::write::BrotliEncoder::new(Vec::new());
        writer
            .write_all(&bytes)
            .await
            .map_err(|_| CacheError::CompressionError)?;
        writer
            .shutdown()
            .await
            .map_err(|_| CacheError::CompressionError)?;
        bytes = writer.into_inner();
        Ok(bytes)
    }

    /// A helper function that returns compressed-encrypted results.
    /// Feature flag (**cec-cache-results**) is required  for this to work.
    ///
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the compressed and encrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(feature = "cec-cache-results")]
    async fn compress_encrypt_compress_results(
        &mut self,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        // compress first
        bytes = self.compress_results(bytes).await?;
        // encrypt
        bytes = self.encrypt_or_decrypt_results(bytes, true).await?;

        // compress again;
        bytes = self.compress_results(bytes).await?;

        Ok(bytes)
    }

    /// A helper function that returns compressed results.
    /// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
    /// If bytes where
    /// # Arguments
    ///
    /// * `bytes` - It takes a slice of bytes as an argument.
    ///
    /// # Error
    ///
    /// Returns the uncompressed bytes on success otherwise it returns a CacheError
    /// on failure.
    #[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
    async fn decompress_results(&mut self, bytes: &[u8]) -> Result<Vec<u8>, Report<CacheError>> {
        cfg_if::cfg_if! {
             if #[cfg(feature = "compress-cache-results")]
            {
               decompress_util(bytes).await

            }
            else if  #[cfg(feature = "cec-cache-results")]
            {
                let decompressed = decompress_util(bytes)?;
                let decrypted = self.encrypt_or_decrypt_results(decompressed, false)?;

                decompress_util(&decrypted).await

            }
        }
    }

    /// A helper function that compresses or encrypts search results before they're inserted into a cache store
    /// # Arguments
    ///
    /// * `search_results` - A reference to the search_Results to process.
    ///
    /// # Error
    ///
    /// Returns a Vec of compressed or encrypted bytes on success otherwise it returns a CacheError
    /// on failure.
    async fn pre_process_search_results(
        &mut self,
        search_results: &SearchResults,
    ) -> Result<Vec<u8>, Report<CacheError>> {
        #[allow(unused_mut)] // needs to be mutable when any of the features is enabled
        let mut bytes: Vec<u8> = search_results.try_into()?;
        #[cfg(feature = "compress-cache-results")]
        {
            let compressed = self.compress_results(bytes).await?;
            bytes = compressed;
        }

        #[cfg(feature = "encrypt-cache-results")]
        {
            let encrypted = self.encrypt_or_decrypt_results(bytes, true).await?;
            bytes = encrypted;
        }

        #[cfg(feature = "cec-cache-results")]
        {
            let compressed_encrypted_compressed =
                self.compress_encrypt_compress_results(bytes).await?;
            bytes = compressed_encrypted_compressed;
        }

        Ok(bytes)
    }

    /// A helper function that decompresses or decrypts search results after they're fetched from the cache-store
    /// # Arguments
    ///
    /// * `bytes` - A Vec of bytes stores in the cache.
    ///
    /// # Error
    ///
    /// Returns the SearchResults struct on success otherwise it returns a CacheError
    /// on failure.
    #[allow(unused_mut)] // needs to be mutable when any of the features is enabled
    async fn post_process_search_results(
        &mut self,
        mut bytes: Vec<u8>,
    ) -> Result<SearchResults, Report<CacheError>> {
        #[cfg(feature = "compress-cache-results")]
        {
            let decompressed = self.decompress_results(&bytes).await?;
            bytes = decompressed
        }

        #[cfg(feature = "encrypt-cache-results")]
        {
            let decrypted = self.encrypt_or_decrypt_results(bytes, false).await?;
            bytes = decrypted
        }

        #[cfg(feature = "cec-cache-results")]
        {
            let decompressed_decrypted = self.decompress_results(&bytes).await?;
            bytes = decompressed_decrypted;
        }

        Ok(bytes.try_into()?)
    }
}

/// A helper function that returns compressed results.
/// Feature flags (**compress-cache-results or cec-cache-results**) are required  for this to work.
/// If bytes where
///
/// # Arguments
///
/// * `bytes` - It takes a slice of bytes as an argument.
///
/// # Error
///
/// Returns the uncompressed bytes on success otherwise it returns a CacheError
/// on failure.
#[cfg(any(feature = "compress-cache-results", feature = "cec-cache-results"))]
async fn decompress_util(input: &[u8]) -> Result<Vec<u8>, Report<CacheError>> {
    use tokio::io::AsyncWriteExt;
    let mut writer = async_compression::tokio::write::BrotliDecoder::new(Vec::new());

    writer
        .write_all(input)
        .await
        .map_err(|_| CacheError::CompressionError)?;
    writer
        .shutdown()
        .await
        .map_err(|_| CacheError::CompressionError)?;
    let bytes = writer.into_inner();
    Ok(bytes)
}

/// An enum that wraps different cache backend implementations for runtime selection.
#[derive(Clone)]
pub enum CacheWrapper {
    /// In-memory cache using moka
    Memory(InMemoryCache),
    /// Redis cache
    Redis(RedisCache),
}

impl CacheWrapper {
    /// Creates a new cache wrapper based on the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration struct containing cache settings.
    ///
    /// # Returns
    ///
    /// Returns a new CacheWrapper instance.
    async fn build(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        match config.cache_backend {
            CacheBackend::Memory => {
                log::info!("Initializing in-memory cache backend");
                Ok(CacheWrapper::Memory(InMemoryCache::build(config).await))
            }
            CacheBackend::Redis => {
                log::info!("Initializing Redis cache backend at {}", config.redis_url);
                Ok(CacheWrapper::Redis(RedisCache::build(config).await))
            }
        }
    }

    /// Checks if cached results exist.
    async fn cached_results_exists(
        &mut self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>> {
        match self {
            CacheWrapper::Memory(cache) => cache.cached_results_exists(urls).await,
            CacheWrapper::Redis(cache) => cache.cached_results_exists(urls).await,
        }
    }

    /// Fetches cached results.
    async fn cached_results(&mut self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        match self {
            CacheWrapper::Memory(cache) => cache.cached_results(url).await,
            CacheWrapper::Redis(cache) => cache.cached_results(url).await,
        }
    }

    /// Caches results.
    async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        match self {
            CacheWrapper::Memory(cache) => cache.cache_results(search_results, urls).await,
            CacheWrapper::Redis(cache) => cache.cache_results(search_results, urls).await,
        }
    }
}

/// TryInto implementation for SearchResults from Vec<u8>
impl TryInto<SearchResults> for Vec<u8> {
    type Error = CacheError;

    fn try_into(self) -> Result<SearchResults, Self::Error> {
        bincode::deserialize_from(self.as_slice()).map_err(|_| CacheError::SerializationError)
    }
}

impl TryInto<Vec<u8>> for &SearchResults {
    type Error = CacheError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(self).map_err(|_| CacheError::SerializationError)
    }
}

/// A structure to efficiently share the cache between threads - as it is protected by a lock-free
/// ArcSwap structure.
pub struct SharedCache(ArcSwap<CacheWrapper>);

impl SharedCache {
    /// A function that creates a new `SharedCache` from a Cache implementation.
    ///
    /// # Arguments
    ///
    /// * `config` - It takes the config struct as an argument.
    ///
    /// Returns a newly constructed `SharedCache` struct.
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self(ArcSwap::from_pointee(
            CacheWrapper::build(config).await?,
        )))
    }

    /// A lock-free, wait-free get operation.
    ///
    /// # Returns
    ///
    /// returns an owned copy of the `CacheWrapper` enum.
    fn cache(&self) -> CacheWrapper {
        (*self.0.load_full()).clone()
    }

    /// A function that checks if the cached results exists in the cache server or not.
    ///
    /// # Arguments
    ///
    /// * `urls` - It takes the hashed search urls as an argument which will be used as the key to
    ///   check whether the cache exists or not.
    ///
    /// # Error
    ///
    /// Returns a Vector containing booleans for each respective url if nothing goes wrong otherwise
    /// returns a `CacheError`.
    pub async fn cached_results_exists(
        &self,
        urls: &[String],
    ) -> Result<Vec<bool>, Report<CacheError>> {
        self.cache().cached_results_exists(urls).await
    }

    /// A getter function which retrieves the cached SearchResults from the internal cache.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the search url as an argument which will be used as the key to fetch the
    ///   cached results from the cache.
    ///
    /// # Error
    ///
    /// Returns a `SearchResults` struct containing the search results from the cache if nothing
    /// goes wrong otherwise returns a `CacheError`.
    pub async fn cached_results(&self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        self.cache().cached_results(url).await
    }

    /// A setter function which caches the results by using the `url` as the key and
    /// `SearchResults` as the value.
    ///
    /// # Arguments
    ///
    /// * `search_results` - It takes the `SearchResults` as an argument which are results that
    ///   needs to be cached.
    /// * `url` - It takes the search url as an argument which will be used as the key for storing
    ///   results in the cache.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached successfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_results(
        &self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> Result<(), Report<CacheError>> {
        let mut mut_cache = self.cache();
        let cache_results = mut_cache.cache_results(search_results, urls).await;
        self.0.store(Arc::new(mut_cache));
        cache_results
    }
}
