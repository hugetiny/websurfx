//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use super::aggregation::SearchResult;
use super::engine_health::EngineErrorType;
use error_stack::{Report, Result, ResultExt};
use reqwest::Client;
use std::fmt;

/// A custom error type used for handle engine associated errors.
#[derive(Debug)]
pub enum EngineError {
    /// No matching engine found
    NoSuchEngineFound(String),
    /// This variant handles all request related errors like forbidden, not found,
    /// etc.
    EmptyResultSet,
    /// This variant handles the not results found error provide by the upstream
    /// search engines.
    RequestError,
    /// Request timeout error
    Timeout,
    /// Access denied (HTTP 403, blocked, etc.)
    AccessDenied,
    /// CAPTCHA required by the search engine
    Captcha,
    /// Too many requests / rate limited (HTTP 429)
    TooManyRequests,
    /// SSL/TLS certificate error
    SslError,
    /// HTTP error with status code
    HttpError(u16),
    /// Parse error (HTML/JSON parsing failed)
    ParseError,
    /// Network connectivity error
    NetworkError,
    ///  This variant handles all the errors which are unexpected or occur rarely
    /// and are errors mostly related to failure in initialization of HeaderMap,
    /// Selector errors and all other errors occurring within the code handling
    /// the `upstream search engines`.
    UnexpectedError,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::NoSuchEngineFound(engine) => {
                write!(f, "No such engine with the name '{engine}' found")
            }
            EngineError::EmptyResultSet => {
                write!(f, "The upstream search engine returned an empty result set")
            }
            EngineError::RequestError => {
                write!(
                    f,
                    "Error occurred while requesting data from upstream search engine"
                )
            }
            EngineError::Timeout => {
                write!(f, "Request timed out while fetching data from upstream search engine")
            }
            EngineError::AccessDenied => {
                write!(f, "Access denied by the upstream search engine")
            }
            EngineError::Captcha => {
                write!(f, "CAPTCHA required by the upstream search engine")
            }
            EngineError::TooManyRequests => {
                write!(f, "Too many requests - rate limited by the upstream search engine")
            }
            EngineError::SslError => {
                write!(f, "SSL/TLS error while connecting to the upstream search engine")
            }
            EngineError::HttpError(code) => {
                write!(f, "HTTP error {} from the upstream search engine", code)
            }
            EngineError::ParseError => {
                write!(f, "Failed to parse response from the upstream search engine")
            }
            EngineError::NetworkError => {
                write!(f, "Network error while connecting to the upstream search engine")
            }
            EngineError::UnexpectedError => {
                write!(f, "An unexpected error occurred while processing the data")
            }
        }
    }
}

impl EngineError {
    /// Convert EngineError to EngineErrorType for health tracking
    pub fn to_error_type(&self) -> EngineErrorType {
        match self {
            EngineError::NoSuchEngineFound(_) => EngineErrorType::EngineNotFound,
            EngineError::EmptyResultSet => EngineErrorType::EmptyResultSet,
            EngineError::RequestError => EngineErrorType::NetworkError,
            EngineError::Timeout => EngineErrorType::Timeout,
            EngineError::AccessDenied => EngineErrorType::AccessDenied,
            EngineError::Captcha => EngineErrorType::Captcha,
            EngineError::TooManyRequests => EngineErrorType::TooManyRequests,
            EngineError::SslError => EngineErrorType::SslError,
            EngineError::HttpError(code) => EngineErrorType::HttpError(*code),
            EngineError::ParseError => EngineErrorType::ParseError,
            EngineError::NetworkError => EngineErrorType::NetworkError,
            EngineError::UnexpectedError => EngineErrorType::UnexpectedError,
        }
    }
}

impl error_stack::Context for EngineError {}

/// A trait to define common behavior for all search engines.
#[async_trait::async_trait]
pub trait SearchEngine: Sync + Send {
    /// This helper function fetches/requests the search results from the upstream search engine in
    /// an html form.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the url of the upstream search engine with the user requested search
    /// query appended in the search parameters.
    /// * `header_map` - It takes the http request headers to be sent to the upstream engine in
    /// order to prevent being detected as a bot. It takes the header as a HeaderMap type.
    /// * `request_timeout` - It takes the request timeout value as seconds which is used to limit
    /// the amount of time for each request to remain connected when until the results can be provided
    /// by the upstream engine.
    ///
    /// # Error
    ///
    /// It returns the html data as a string if the upstream engine provides the data as expected
    /// otherwise it returns a custom `EngineError`.
    async fn fetch_html_from_upstream(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        client: &Client,
    ) -> Result<String, EngineError> {
        // fetch the html from upstream search engine
        Ok(client
            .get(url)
            .headers(header_map) // add spoofed headers to emulate human behavior
            .send()
            .await
            .change_context(EngineError::RequestError)?
            .text()
            .await
            .change_context(EngineError::RequestError)?)
    }

    /// This helper function fetches/requests the json search results from the upstream search engine as a vector of bytes.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the url of the upstream search engine with the user requested search
    /// query appended in the search parameters.
    /// * `header_map` - It takes the http request headers to be sent to the upstream engine in
    /// order to prevent being detected as a bot. It takes the header as a HeaderMap type.
    /// * `request_timeout` - It takes the request timeout value as seconds which is used to limit
    /// the amount of time for each request to remain connected when until the results can be provided
    /// by the upstream engine.
    ///
    /// # Error
    ///
    /// It returns the html data as a vector of bytes if the upstream engine provides the data as expected
    /// otherwise it returns a custom `EngineError`.
    async fn fetch_json_as_bytes_from_upstream(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        client: &Client,
    ) -> Result<Vec<u8>, EngineError> {
        // fetch the json response from upstream search engine

        Ok(client
            .get(url)
            .headers(header_map) // add spoofed headers to emulate human behavior
            .send()
            .await
            .change_context(EngineError::RequestError)?
            .bytes()
            .await
            .change_context(EngineError::RequestError)?
            .to_vec())
    }

    /// This function scrapes results from the upstream engine and puts all the scraped results like
    /// title, visiting_url (href in html),engine (from which engine it was fetched from) and description
    /// in a RawSearchResult and then adds that to HashMap whose keys are url and values are RawSearchResult
    /// struct and then returns it within a Result enum.
    ///
    /// # Arguments
    ///
    /// * `query` - Takes the user provided query to query to the upstream search engine with.
    /// * `page` - Takes an u32 as an argument.
    /// * `user_agent` - Takes a random user agent string as an argument.
    /// * `request_timeout` - Takes a time (secs) as a value which controls the server request timeout.
    ///
    /// # Errors
    ///
    /// Returns an `EngineErrorKind` if the user is not connected to the internet or if their is failure to
    /// reach the above `upstream search engine` page or if the `upstream search engine` is unable to
    /// provide results for the requested search query and also returns error if the scraping selector
    /// or HeaderMap fails to initialize.
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError>;
}

/// A named struct which stores the engine struct with the name of the associated engine.
pub struct EngineHandler {
    /// It stores the engine struct wrapped in a box smart pointer as the engine struct implements
    /// the `SearchEngine` trait.
    engine: Box<dyn SearchEngine>,
    /// It stores the name of the engine to which the struct is associated to.
    name: &'static str,
}

impl Clone for EngineHandler {
    fn clone(&self) -> Self {
        Self::new(self.name).unwrap()
    }
}

impl EngineHandler {
    /// Parses an engine name into an engine handler.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - It takes the name of the engine to which the struct was associated to.
    ///
    /// # Returns
    ///
    /// It returns an option either containing the value or a none if the engine is unknown
    pub fn new(engine_name: &str) -> Result<Self, EngineError> {
        let engine: (&'static str, Box<dyn SearchEngine>) =
            match engine_name.to_lowercase().as_str() {
                "duckduckgo" => {
                    let engine = crate::engines::duckduckgo::DuckDuckGo::new()?;
                    ("duckduckgo", Box::new(engine))
                }
                "searx" => {
                    let engine = crate::engines::searx::Searx::new()?;
                    ("searx", Box::new(engine))
                }
                "brave" => {
                    let engine = crate::engines::brave::Brave::new()?;
                    ("brave", Box::new(engine))
                }
                "startpage" => {
                    let engine = crate::engines::startpage::Startpage::new()?;
                    ("startpage", Box::new(engine))
                }
                "librex" => {
                    let engine = crate::engines::librex::LibreX::new()?;
                    ("librex", Box::new(engine))
                }
                "mojeek" => {
                    let engine = crate::engines::mojeek::Mojeek::new()?;
                    ("mojeek", Box::new(engine))
                }
                "bing" => {
                    let engine = crate::engines::bing::Bing::new()?;
                    ("bing", Box::new(engine))
                }
                "wikipedia" => {
                    let engine = crate::engines::wikipedia::Wikipedia::new("en")?;
                    ("wikipedia", Box::new(engine))
                }
                "yahoo" => {
                    let engine = crate::engines::yahoo::Yahoo::new()?;
                    ("yahoo", Box::new(engine))
                }
                "baidu" => {
                    let engine = crate::engines::baidu::Baidu::new()?;
                    ("baidu", Box::new(engine))
                }
                "google" => {
                    let engine = crate::engines::google::Google::new("en")?;
                    ("google", Box::new(engine))
                }
                "sogou" => {
                    let engine = crate::engines::sogou::Sogou::new()?;
                    ("sogou", Box::new(engine))
                }
                // New general search engines
                "qwant" => {
                    let engine = crate::engines::qwant::Qwant::new()?;
                    ("qwant", Box::new(engine))
                }
                "yandex" => {
                    let engine = crate::engines::yandex::Yandex::new()?;
                    ("yandex", Box::new(engine))
                }
                "ask" => {
                    let engine = crate::engines::ask::Ask::new()?;
                    ("ask", Box::new(engine))
                }
                // Chinese search engines
                "360search" | "search360" => {
                    let engine = crate::engines::search360::Search360::new()?;
                    ("360search", Box::new(engine))
                }
                "chinaso" => {
                    let engine = crate::engines::chinaso::Chinaso::new()?;
                    ("chinaso", Box::new(engine))
                }
                // Asian search engines
                "naver" => {
                    let engine = crate::engines::naver::Naver::new()?;
                    ("naver", Box::new(engine))
                }
                // Reference
                "wikidata" => {
                    let engine = crate::engines::wikidata::Wikidata::new()?;
                    ("wikidata", Box::new(engine))
                }
                "openlibrary" => {
                    let engine = crate::engines::openlibrary::OpenLibrary::new()?;
                    ("openlibrary", Box::new(engine))
                }
                // Developer tools
                "github" => {
                    let engine = crate::engines::github::GitHub::new()?;
                    ("github", Box::new(engine))
                }
                "stackexchange" | "stackoverflow" => {
                    let engine = crate::engines::stackexchange::StackExchange::new()?;
                    ("stackexchange", Box::new(engine))
                }
                "npm" => {
                    let engine = crate::engines::npm::Npm::new()?;
                    ("npm", Box::new(engine))
                }
                "pypi" => {
                    let engine = crate::engines::pypi::PyPI::new()?;
                    ("pypi", Box::new(engine))
                }
                "crates" | "crates.io" => {
                    let engine = crate::engines::crates::Crates::new()?;
                    ("crates", Box::new(engine))
                }
                "dockerhub" | "docker" => {
                    let engine = crate::engines::dockerhub::DockerHub::new()?;
                    ("dockerhub", Box::new(engine))
                }
                "huggingface" | "hf" => {
                    let engine = crate::engines::huggingface::HuggingFace::new()?;
                    ("huggingface", Box::new(engine))
                }
                // Academic
                "arxiv" => {
                    let engine = crate::engines::arxiv::Arxiv::new()?;
                    ("arxiv", Box::new(engine))
                }
                "pubmed" => {
                    let engine = crate::engines::pubmed::PubMed::new()?;
                    ("pubmed", Box::new(engine))
                }
                // Social/Community
                "hackernews" | "hn" => {
                    let engine = crate::engines::hackernews::HackerNews::new()?;
                    ("hackernews", Box::new(engine))
                }
                "reddit" => {
                    let engine = crate::engines::reddit::Reddit::new()?;
                    ("reddit", Box::new(engine))
                }
                // Media
                "youtube" => {
                    let engine = crate::engines::youtube::YouTube::new()?;
                    ("youtube", Box::new(engine))
                }
                "bilibili" => {
                    let engine = crate::engines::bilibili::Bilibili::new()?;
                    ("bilibili", Box::new(engine))
                }
                "soundcloud" => {
                    let engine = crate::engines::soundcloud::SoundCloud::new()?;
                    ("soundcloud", Box::new(engine))
                }
                "imdb" => {
                    let engine = crate::engines::imdb::IMDb::new()?;
                    ("imdb", Box::new(engine))
                }
                "unsplash" => {
                    let engine = crate::engines::unsplash::Unsplash::new()?;
                    ("unsplash", Box::new(engine))
                }
                // New Video engines
                "vimeo" => {
                    let engine = crate::engines::vimeo::Vimeo::new()?;
                    ("vimeo", Box::new(engine))
                }
                "dailymotion" => {
                    let engine = crate::engines::dailymotion::Dailymotion::new()?;
                    ("dailymotion", Box::new(engine))
                }
                // New Image engines
                "flickr" => {
                    let engine = crate::engines::flickr::Flickr::new()?;
                    ("flickr", Box::new(engine))
                }
                "pixabay" => {
                    let engine = crate::engines::pixabay::Pixabay::new()?;
                    ("pixabay", Box::new(engine))
                }
                "deviantart" => {
                    let engine = crate::engines::deviantart::DeviantArt::new()?;
                    ("deviantart", Box::new(engine))
                }
                // New Music engines
                "bandcamp" => {
                    let engine = crate::engines::bandcamp::Bandcamp::new()?;
                    ("bandcamp", Box::new(engine))
                }
                "genius" => {
                    let engine = crate::engines::genius::Genius::new()?;
                    ("genius", Box::new(engine))
                }
                "spotify" => {
                    let engine = crate::engines::spotify::Spotify::new()?;
                    ("spotify", Box::new(engine))
                }
                "applemusic" => {
                    let engine = crate::engines::applemusic::AppleMusic::new()?;
                    ("applemusic", Box::new(engine))
                }
                // New Code hosting engines
                "gitlab" => {
                    let engine = crate::engines::gitlab::GitLab::new()?;
                    ("gitlab", Box::new(engine))
                }
                "codeberg" => {
                    let engine = crate::engines::codeberg::Codeberg::new()?;
                    ("codeberg", Box::new(engine))
                }
                "bitbucket" => {
                    let engine = crate::engines::bitbucket::Bitbucket::new()?;
                    ("bitbucket", Box::new(engine))
                }
                // Maps
                "openstreetmap" | "osm" => {
                    let engine = crate::engines::openstreetmap::OpenStreetMap::new()?;
                    ("openstreetmap", Box::new(engine))
                }
                // Wikimedia family
                "wikibooks" => {
                    let engine = crate::engines::wikibooks::Wikibooks::new()?;
                    ("wikibooks", Box::new(engine))
                }
                "wikiquote" => {
                    let engine = crate::engines::wikiquote::Wikiquote::new()?;
                    ("wikiquote", Box::new(engine))
                }
                "wikinews" => {
                    let engine = crate::engines::wikinews::Wikinews::new()?;
                    ("wikinews", Box::new(engine))
                }
                "wikisource" => {
                    let engine = crate::engines::wikisource::Wikisource::new()?;
                    ("wikisource", Box::new(engine))
                }
                "wikiversity" => {
                    let engine = crate::engines::wikiversity::Wikiversity::new()?;
                    ("wikiversity", Box::new(engine))
                }
                "wikivoyage" => {
                    let engine = crate::engines::wikivoyage::Wikivoyage::new()?;
                    ("wikivoyage", Box::new(engine))
                }
                "wiktionary" => {
                    let engine = crate::engines::wiktionary::Wiktionary::new()?;
                    ("wiktionary", Box::new(engine))
                }
                "wikispecies" => {
                    let engine = crate::engines::wikispecies::Wikispecies::new()?;
                    ("wikispecies", Box::new(engine))
                }
                "wikicommons" | "wikimediacommons" => {
                    let engine = crate::engines::wikicommons::Wikicommons::new()?;
                    ("wikicommons", Box::new(engine))
                }
                // Archive
                "internetarchive" | "archive" => {
                    let engine = crate::engines::internetarchive::InternetArchive::new()?;
                    ("internetarchive", Box::new(engine))
                }
                // Academic
                "semanticscholar" => {
                    let engine = crate::engines::semanticscholar::SemanticScholar::new()?;
                    ("semanticscholar", Box::new(engine))
                }
                "crossref" => {
                    let engine = crate::engines::crossref::CrossRef::new()?;
                    ("crossref", Box::new(engine))
                }
                "core" => {
                    let engine = crate::engines::core::Core::new()?;
                    ("core", Box::new(engine))
                }
                "googlescholar" => {
                    let engine = crate::engines::googlescholar::GoogleScholar::new()?;
                    ("googlescholar", Box::new(engine))
                }
                "wolframalpha" | "wolfram" => {
                    let engine = crate::engines::wolframalpha::WolframAlpha::new()?;
                    ("wolframalpha", Box::new(engine))
                }
                // Shopping
                "ebay" => {
                    let engine = crate::engines::ebay::Ebay::new()?;
                    ("ebay", Box::new(engine))
                }
                "amazon" => {
                    let engine = crate::engines::amazon::Amazon::new()?;
                    ("amazon", Box::new(engine))
                }
                "taobao" => {
                    let engine = crate::engines::taobao::Taobao::new()?;
                    ("taobao", Box::new(engine))
                }
                "jd" | "jingdong" => {
                    let engine = crate::engines::jd::Jd::new()?;
                    ("jd", Box::new(engine))
                }
                // Social
                "twitter" | "x" => {
                    let engine = crate::engines::twitter::Twitter::new()?;
                    ("twitter", Box::new(engine))
                }
                "mastodon" => {
                    let engine = crate::engines::mastodon::Mastodon::new()?;
                    ("mastodon", Box::new(engine))
                }
                "lemmy" => {
                    let engine = crate::engines::lemmy::Lemmy::new()?;
                    ("lemmy", Box::new(engine))
                }
                "lobsters" => {
                    let engine = crate::engines::lobsters::Lobsters::new()?;
                    ("lobsters", Box::new(engine))
                }
                // Chinese social/content
                "zhihu" => {
                    let engine = crate::engines::zhihu::Zhihu::new()?;
                    ("zhihu", Box::new(engine))
                }
                "weibo" => {
                    let engine = crate::engines::weibo::Weibo::new()?;
                    ("weibo", Box::new(engine))
                }
                "douban" => {
                    let engine = crate::engines::douban::Douban::new()?;
                    ("douban", Box::new(engine))
                }
                // Movies
                "rottentomatoes" => {
                    let engine = crate::engines::rottentomatoes::RottenTomatoes::new()?;
                    ("rottentomatoes", Box::new(engine))
                }
                _ => {
                    return Err(Report::from(EngineError::NoSuchEngineFound(
                        engine_name.to_string(),
                    )));
                }
            };

        Ok(Self {
            engine: engine.1,
            name: engine.0,
        })
    }

    /// This function converts the EngineHandler type into a tuple containing the engine name and
    /// the associated engine struct.
    pub fn into_name_engine(self) -> (&'static str, Box<dyn SearchEngine>) {
        (self.name, self.engine)
    }
}
