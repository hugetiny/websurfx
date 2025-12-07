//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use super::engine::EngineError;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[cfg(any(
    feature = "use-synonyms-search",
    feature = "use-non-static-synonyms-search"
))]
use {arc_swap::ArcSwap, rayon::iter::ParallelExtend, std::sync::Arc, thesaurus::synonyms};

/// A named struct to store the raw scraped search results scraped search results from the
/// upstream search engines before aggregating it.It derives the Clone trait which is needed
/// to write idiomatic rust using `Iterators`.
///
/// This struct is designed to be fully compatible with SearXNG's result format.
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The title of the search result.
    pub title: String,
    /// The url which is accessed when clicked on it (href url in html in simple words).
    pub url: String,
    /// The description of the search result.
    pub description: String,
    /// The names of the upstream engines from which this results were provided.
    pub engine: Vec<String>,
    /// The td-tdf score of the result in regards to the title, url and description and the
    /// user's query
    pub relevance_score: f32,

    // ============================================================================
    // SearXNG-compatible fields below
    // ============================================================================

    /// Thumbnail URL for the result (used in news, images, videos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,

    /// Full-size image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,

    /// Thumbnail source URL (alternative to thumbnail)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_src: Option<String>,

    /// Image format (e.g., "jpg", "png", "gif")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_format: Option<String>,

    /// Published date as ISO 8601 string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,

    /// Author of the content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Source website/domain name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Iframe source URL (for embedded content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iframe_src: Option<String>,

    /// Audio source URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_src: Option<String>,

    /// Video duration/length in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,

    /// View count (for videos, articles)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub views: Option<String>,

    /// Metadata string (miscellaneous info)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,

    /// Template name for special result types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Category of the result (general, images, videos, news, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Magnet link (for torrent results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnetlink: Option<String>,

    /// Torrent file URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrentfile: Option<String>,

    /// Seed count (for torrents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,

    /// Leech count (for torrents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leech: Option<u32>,

    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesize: Option<u64>,

    /// File size as human-readable string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesize_human: Option<String>,

    /// DOI (for academic papers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    /// Whether the paper is open access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access: Option<bool>,

    /// Comments URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,

    /// ISBN (for books)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,

    /// Publisher name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Journal name (for academic results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,

    /// Price (for shopping results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// Currency code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Shipping info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<String>,

    /// Latitude (for map results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,

    /// Longitude (for map results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,

    /// Address (for map/location results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Rating (e.g., "4.5/5")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,

    /// Number of reviews
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviews: Option<u32>,

    /// Programming language (for code results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_language: Option<String>,

    /// Code snippet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Package version (for package managers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Download count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<String>,

    /// License type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Tags/keywords
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl SearchResult {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    /// Uses Default for all optional SearXNG-compatible fields.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the search result.
    /// * `url` - The url which is accessed when clicked on it
    ///   (href url in html in simple words).
    /// * `description` - The description of the search result.
    /// * `engine` - The names of the upstream engines from which this results were provided.
    pub fn new(title: &str, url: &str, description: &str, engine: &[&str]) -> Self {
        SearchResult {
            title: title.to_owned(),
            url: url.to_owned(),
            description: description.to_owned(),
            relevance_score: 0.0,
            engine: engine.iter().map(|name| name.to_string()).collect(),
            ..Default::default()
        }
    }

    /// Constructs a new `SearchResult` with extended SearXNG fields.
    /// Use this builder for results that have additional metadata like thumbnails.
    pub fn with_extras(
        title: &str,
        url: &str,
        description: &str,
        engine: &[&str],
    ) -> SearchResultBuilder {
        SearchResultBuilder::new(title, url, description, engine)
    }
    /// calculates and update the relevance score of the current search.
    ///
    /// # Arguments
    ///
    /// * query -  the query string  used to obtain the results
    pub fn calculate_relevance(&mut self, query: &str) {
        use stop_words::{LANGUAGE, get};
        // when language settings can change to any of the ones supported on this crate: https://docs.rs/crate/stop-words/0.8.0
        let documents = [
            self.title.clone(),
            self.url.clone(),
            self.description.clone(),
        ];

        let stop_words = get(LANGUAGE::English);
        let punctuation = [
            ".".to_owned(),
            ",".to_owned(),
            ":".to_owned(),
            ";".to_owned(),
            "!".to_owned(),
            "?".to_owned(),
            "(".to_owned(),
            ")".to_owned(),
            "[".to_owned(),
            "]".to_owned(),
            "{".to_owned(),
            "}".to_owned(),
            "\"".to_owned(),
            "'".to_owned(),
            "<".to_owned(),
            ">".to_owned(),
        ];

        self.relevance_score = calculate_tf_idf(query, &documents, &stop_words, &punctuation);
    }

    /// A function which adds the engine name provided as a string into a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `engine` - Takes an engine name provided as a String.
    pub fn add_engines(&mut self, engine: &str) {
        self.engine.push(engine.to_owned())
    }

    /// A function which returns the engine name stored from the struct as a string.
    ///
    /// # Returns
    ///
    /// An engine name stored as a string from the struct.
    pub fn engine(&mut self) -> String {
        std::mem::take(&mut self.engine[0])
    }
}

/// Builder for creating SearchResult with optional SearXNG-compatible fields.
/// This provides a fluent API for constructing results with additional metadata.
#[derive(Clone, Default)]
pub struct SearchResultBuilder {
    result: SearchResult,
}

impl SearchResultBuilder {
    /// Create a new builder with required fields
    pub fn new(title: &str, url: &str, description: &str, engine: &[&str]) -> Self {
        Self {
            result: SearchResult::new(title, url, description, engine),
        }
    }

    /// Set the thumbnail URL
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.result.thumbnail = Some(url.into());
        self
    }

    /// Set the full-size image URL
    pub fn img_src(mut self, url: impl Into<String>) -> Self {
        self.result.img_src = Some(url.into());
        self
    }

    /// Set the thumbnail source URL (alternative)
    pub fn thumbnail_src(mut self, url: impl Into<String>) -> Self {
        self.result.thumbnail_src = Some(url.into());
        self
    }

    /// Set the image format
    pub fn img_format(mut self, format: impl Into<String>) -> Self {
        self.result.img_format = Some(format.into());
        self
    }

    /// Set the published date (ISO 8601 format)
    pub fn published_date(mut self, date: impl Into<String>) -> Self {
        self.result.published_date = Some(date.into());
        self
    }

    /// Set the author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.result.author = Some(author.into());
        self
    }

    /// Set the source website
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.result.source = Some(source.into());
        self
    }

    /// Set the iframe source URL
    pub fn iframe_src(mut self, url: impl Into<String>) -> Self {
        self.result.iframe_src = Some(url.into());
        self
    }

    /// Set the audio source URL
    pub fn audio_src(mut self, url: impl Into<String>) -> Self {
        self.result.audio_src = Some(url.into());
        self
    }

    /// Set the duration/length in seconds
    pub fn length(mut self, seconds: u64) -> Self {
        self.result.length = Some(seconds);
        self
    }

    /// Set the view count
    pub fn views(mut self, views: impl Into<String>) -> Self {
        self.result.views = Some(views.into());
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: impl Into<String>) -> Self {
        self.result.metadata = Some(metadata.into());
        self
    }

    /// Set the template name
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.result.template = Some(template.into());
        self
    }

    /// Set the category
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.result.category = Some(category.into());
        self
    }

    /// Set magnet link (for torrents)
    pub fn magnetlink(mut self, link: impl Into<String>) -> Self {
        self.result.magnetlink = Some(link.into());
        self
    }

    /// Set torrent file URL
    pub fn torrentfile(mut self, url: impl Into<String>) -> Self {
        self.result.torrentfile = Some(url.into());
        self
    }

    /// Set seed count (for torrents)
    pub fn seed(mut self, count: u32) -> Self {
        self.result.seed = Some(count);
        self
    }

    /// Set leech count (for torrents)
    pub fn leech(mut self, count: u32) -> Self {
        self.result.leech = Some(count);
        self
    }

    /// Set file size in bytes
    pub fn filesize(mut self, size: u64) -> Self {
        self.result.filesize = Some(size);
        self
    }

    /// Set human-readable file size
    pub fn filesize_human(mut self, size: impl Into<String>) -> Self {
        self.result.filesize_human = Some(size.into());
        self
    }

    /// Set DOI (for academic papers)
    pub fn doi(mut self, doi: impl Into<String>) -> Self {
        self.result.doi = Some(doi.into());
        self
    }

    /// Set open access flag
    pub fn open_access(mut self, is_open: bool) -> Self {
        self.result.open_access = Some(is_open);
        self
    }

    /// Set comments URL
    pub fn comments(mut self, url: impl Into<String>) -> Self {
        self.result.comments = Some(url.into());
        self
    }

    /// Set ISBN
    pub fn isbn(mut self, isbn: impl Into<String>) -> Self {
        self.result.isbn = Some(isbn.into());
        self
    }

    /// Set publisher
    pub fn publisher(mut self, publisher: impl Into<String>) -> Self {
        self.result.publisher = Some(publisher.into());
        self
    }

    /// Set journal name
    pub fn journal(mut self, journal: impl Into<String>) -> Self {
        self.result.journal = Some(journal.into());
        self
    }

    /// Set price
    pub fn price(mut self, price: impl Into<String>) -> Self {
        self.result.price = Some(price.into());
        self
    }

    /// Set currency code
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.result.currency = Some(currency.into());
        self
    }

    /// Set shipping info
    pub fn shipping(mut self, shipping: impl Into<String>) -> Self {
        self.result.shipping = Some(shipping.into());
        self
    }

    /// Set coordinates
    pub fn coordinates(mut self, lat: f64, lon: f64) -> Self {
        self.result.latitude = Some(lat);
        self.result.longitude = Some(lon);
        self
    }

    /// Set address
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.result.address = Some(address.into());
        self
    }

    /// Set phone number
    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.result.phone = Some(phone.into());
        self
    }

    /// Set rating
    pub fn rating(mut self, rating: impl Into<String>) -> Self {
        self.result.rating = Some(rating.into());
        self
    }

    /// Set review count
    pub fn reviews(mut self, count: u32) -> Self {
        self.result.reviews = Some(count);
        self
    }

    /// Set programming language
    pub fn code_language(mut self, lang: impl Into<String>) -> Self {
        self.result.code_language = Some(lang.into());
        self
    }

    /// Set code snippet
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.result.code = Some(code.into());
        self
    }

    /// Set package version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.result.version = Some(version.into());
        self
    }

    /// Set download count
    pub fn downloads(mut self, downloads: impl Into<String>) -> Self {
        self.result.downloads = Some(downloads.into());
        self
    }

    /// Set license
    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.result.license = Some(license.into());
        self
    }

    /// Set tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.result.tags = Some(tags);
        self
    }

    /// Build the final SearchResult
    pub fn build(self) -> SearchResult {
        self.result
    }
}

/// A named struct that stores the error info related to the upstream search engines.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EngineErrorInfo {
    /// It stores the error type which occured while fetching the result from a particular search
    /// engine.
    pub error: String,
    /// It stores the name of the engine that failed to provide the requested search results.
    pub engine: String,
    /// It stores the name of the color to indicate whether how severe the particular error is (In
    /// other words it indicates the severity of the error/issue).
    pub severity_color: String,
    /// Whether the engine was suspended due to this error
    pub suspended: bool,
    /// Remaining suspension time in seconds (0 if not suspended)
    pub suspension_remaining_secs: u64,
}

impl EngineErrorInfo {
    /// Constructs a new `EngineErrorInfo` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `error` - It takes the error type which occured while fetching the result from a particular
    ///   search engine.
    /// * `engine` - It takes the name of the engine that failed to provide the requested search results.
    pub fn new(error: &EngineError, engine: &str) -> Self {
        Self {
            error: match error {
                EngineError::NoSuchEngineFound(_) => "EngineNotFound".to_owned(),
                EngineError::RequestError => "RequestError".to_owned(),
                EngineError::EmptyResultSet => "EmptyResultSet".to_owned(),
                EngineError::Timeout => "Timeout".to_owned(),
                EngineError::AccessDenied => "AccessDenied".to_owned(),
                EngineError::Captcha => "Captcha".to_owned(),
                EngineError::TooManyRequests => "TooManyRequests".to_owned(),
                EngineError::SslError => "SslError".to_owned(),
                EngineError::HttpError(code) => format!("HttpError_{}", code),
                EngineError::ParseError => "ParseError".to_owned(),
                EngineError::NetworkError => "NetworkError".to_owned(),
                EngineError::UnexpectedError => "UnexpectedError".to_owned(),
            },
            engine: engine.to_owned(),
            severity_color: match error {
                EngineError::NoSuchEngineFound(_) => "red".to_owned(),
                EngineError::RequestError => "yellow".to_owned(),
                EngineError::EmptyResultSet => "blue".to_owned(),
                EngineError::Timeout => "yellow".to_owned(),
                EngineError::AccessDenied => "red".to_owned(),
                EngineError::Captcha => "red".to_owned(),
                EngineError::TooManyRequests => "orange".to_owned(),
                EngineError::SslError => "red".to_owned(),
                EngineError::HttpError(_) => "orange".to_owned(),
                EngineError::ParseError => "yellow".to_owned(),
                EngineError::NetworkError => "yellow".to_owned(),
                EngineError::UnexpectedError => "red".to_owned(),
            },
            suspended: false,
            suspension_remaining_secs: 0,
        }
    }

    /// Create error info with suspension details
    pub fn with_suspension(error: &EngineError, engine: &str, suspended: bool, remaining_secs: u64) -> Self {
        let mut info = Self::new(error, engine);
        info.suspended = suspended;
        info.suspension_remaining_secs = remaining_secs;
        info
    }
}

/// A named struct to store, serialize, deserialize the all the search results scraped and
/// aggregated from the upstream search engines.
/// `SearchResult` structs.
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// Stores the individual serializable `SearchResult` struct into a vector of
    pub results: Box<[SearchResult]>,
    /// Stores the information on which engines failed with their engine name
    /// and the type of error that caused it.
    pub engine_errors_info: Box<[EngineErrorInfo]>,
    /// Stores the flag option which holds the check value that the following
    /// search query was disallowed when the safe search level set to 4 and it
    /// was present in the `Blocklist` file.
    pub disallowed: bool,
    /// Stores the flag option which holds the check value that the following
    /// search query was filtered when the safe search level set to 3 and it
    /// was present in the `Blocklist` file.
    pub filtered: bool,
    /// Stores the safe search level `safesearch` provided in the search url.
    pub safe_search_level: u8,
    /// Stores the flag option which holds the check value that whether any search engines were
    /// selected or not.
    pub no_engines_selected: bool,
}

impl SearchResults {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `results` - Takes an argument of individual serializable `SearchResult` struct
    ///   and stores it into a vector of `SearchResult` structs.
    /// * `page_query` - Takes an argument of current page`s search query `q` provided in
    ///   the search url.
    /// * `engine_errors_info` - Takes an array of structs which contains information regarding
    ///   which engines failed with their names, reason and their severity color name.
    pub fn new(results: Box<[SearchResult]>, engine_errors_info: Box<[EngineErrorInfo]>) -> Self {
        Self {
            results,
            engine_errors_info,
            disallowed: Default::default(),
            filtered: Default::default(),
            safe_search_level: Default::default(),
            no_engines_selected: Default::default(),
        }
    }

    /// A setter function that sets disallowed to true.
    pub fn set_disallowed(&mut self) {
        self.disallowed = true;
    }

    /// A setter function that sets the filtered to true.
    pub fn set_filtered(&mut self, filtered: bool) {
        self.filtered = filtered;
    }

    /// A getter function that gets the value of `engine_errors_info`.
    pub fn engine_errors_info(&mut self) -> Box<[EngineErrorInfo]> {
        std::mem::take(&mut self.engine_errors_info)
    }
    /// A getter function that gets the value of `results`.
    pub fn results(&mut self) -> Box<[SearchResult]> {
        self.results.clone()
    }

    /// A setter function to set the current page safe search level.
    pub fn set_safe_search_level(&mut self, safe_search_level: u8) {
        self.safe_search_level = safe_search_level;
    }

    /// A getter function that gets the value of `no_engines_selected`.
    pub fn no_engines_selected(&self) -> bool {
        self.no_engines_selected
    }

    /// A setter function to set the `no_engines_selected` to true.
    pub fn set_no_engines_selected(&mut self) {
        self.no_engines_selected = true;
    }
}

/// Helper function to calculate the tf-idf for the search query.
/// <br> The approach is  as [`as`](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
///  <br> Find a sample article about TF-IDF [`here`](https://medium.com/analytics-vidhya/tf-idf-term-frequency-technique-easiest-explanation-for-text-classification-in-nlp-with-code-8ca3912e58c3)
/// ### Arguments
/// * `query` -  a user's search query
/// * `documents` -  a list of text used for comparision (url, title, description)
/// * `stop_words` - A list of language specific stop words.
/// * `punctuation` - list of punctuation symbols.
/// ### Returns
/// * `score` - The average tf-idf score of the word tokens (and synonyms) in the query
fn calculate_tf_idf(
    query: &str,
    documents: &[String],
    stop_words: &[String],
    punctuation: &[String],
) -> f32 {
    use keyword_extraction::{
        tf_idf::{TfIdf, TfIdfParams},
        tokenizer::Tokenizer,
    };

    let params = TfIdfParams::UnprocessedDocuments(documents, stop_words, Some(punctuation));
    let tf_idf = TfIdf::new(params);
    let tokener = Tokenizer::new(query, stop_words, Some(punctuation));
    let query_tokens = tokener.split_into_words();

    #[cfg(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    ))]
    let extra_tokens = ArcSwap::from_pointee(Vec::new());

    let total_score: f32 = query_tokens
        .par_iter()
        .map(|token| {
            #[cfg(any(
                feature = "use-synonyms-search",
                feature = "use-non-static-synonyms-search"
            ))]
            {
                let mut extra_tokens_owned = (*extra_tokens.load_full()).clone();

                // find some synonyms and add them to the search  (from wordnet or moby if feature is enabled)
                extra_tokens_owned.par_extend(synonyms(token));

                extra_tokens.store(Arc::new(extra_tokens_owned));
            }

            tf_idf.get_score(token)
        })
        .sum();

    let result: f32;

    #[cfg(not(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    )))]
    {
        result = total_score / (query_tokens.len() as f32);
    }

    #[cfg(any(
        feature = "use-synonyms-search",
        feature = "use-non-static-synonyms-search"
    ))]
    {
        let extra_tokens_owned = (*extra_tokens.load_full()).clone();
        let extra_total_score: f32 = extra_tokens_owned
            .par_iter()
            .map(|token| tf_idf.get_score(token))
            .sum();

        result = (extra_total_score + total_score)
            / ((query_tokens.len() + extra_tokens_owned.len()) as f32);
    }

    f32::from(!result.is_nan()) * result
}
