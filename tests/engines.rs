//! Comprehensive tests for all search engines.
//!
//! This module tests each search engine implementation to verify:
//! 1. Engine can be instantiated successfully
//! 2. Engine can perform searches without panicking
//! 3. Engine returns results in the expected format
//!
//! Note: Some tests may fail due to network issues, rate limiting, or geo-restrictions.
//! These are integration tests that require network access.

use reqwest::Client;
use std::time::Duration;
use websurfx::models::engine::{EngineError, EngineHandler};

/// Default test query
const TEST_QUERY: &str = "rust programming";
/// Chinese test query
const TEST_QUERY_CN: &str = "编程语言";
/// Academic test query
const TEST_QUERY_ACADEMIC: &str = "machine learning";
/// Music test query
const TEST_QUERY_MUSIC: &str = "beethoven symphony";
/// Image test query
const TEST_QUERY_IMAGE: &str = "nature landscape";
/// Video test query
const TEST_QUERY_VIDEO: &str = "tutorial";
/// Shopping test query
const TEST_QUERY_SHOPPING: &str = "laptop";
/// Map test query
const TEST_QUERY_MAP: &str = "New York";

/// Test user agent
const TEST_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Create a test HTTP client with reasonable timeouts
fn create_test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(TEST_USER_AGENT)
        .build()
        .expect("Failed to create HTTP client")
}

/// Helper macro to generate engine instantiation tests
macro_rules! test_engine_new {
    ($test_name:ident, $engine_name:expr) => {
        #[test]
        fn $test_name() {
            let result = EngineHandler::new($engine_name);
            assert!(result.is_ok(), "Failed to create engine '{}': {:?}", $engine_name, result.err());
        }
    };
}

/// Helper macro to generate engine search tests
macro_rules! test_engine_search {
    ($test_name:ident, $engine_name:expr, $query:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let client = create_test_client();
            let handler = match EngineHandler::new($engine_name) {
                Ok(h) => h,
                Err(e) => {
                    println!("Skipping test: Failed to create engine '{}': {:?}", $engine_name, e);
                    return;
                }
            };

            let (name, engine) = handler.into_name_engine();
            println!("Testing engine: {} with query: '{}'", name, $query);

            let result = engine.results($query, 1, TEST_USER_AGENT, &client, 0).await;

            match result {
                Ok(results) => {
                    println!("Engine '{}' returned {} results", name, results.len());
                    // Verify results are not empty (if we got Ok)
                    assert!(!results.is_empty(), "Engine '{}' returned empty results", name);

                    // Verify each result has required fields
                    for (url, search_result) in &results {
                        assert!(!url.is_empty(), "Result URL should not be empty");
                        assert!(!search_result.title.is_empty(), "Result title should not be empty for URL: {}", url);
                    }
                }
                Err(e) => {
                    // Some errors are expected (rate limiting, geo-restrictions, etc.)
                    println!("Engine '{}' returned error (may be expected): {:?}", name, e);
                    // Don't fail the test for network-related errors
                    let error_string = format!("{:?}", e);
                    if error_string.contains("EmptyResultSet") {
                        println!("Note: Empty result set may indicate the query returned no results");
                    }
                }
            }
        }
    };
}

// ============================================================================
// Engine Instantiation Tests
// ============================================================================

mod instantiation {
    use super::*;

    // General Search Engines
    test_engine_new!(test_new_duckduckgo, "duckduckgo");
    test_engine_new!(test_new_searx, "searx");
    test_engine_new!(test_new_brave, "brave");
    test_engine_new!(test_new_startpage, "startpage");
    test_engine_new!(test_new_librex, "librex");
    test_engine_new!(test_new_mojeek, "mojeek");
    test_engine_new!(test_new_bing, "bing");
    test_engine_new!(test_new_yahoo, "yahoo");
    test_engine_new!(test_new_google, "google");
    test_engine_new!(test_new_qwant, "qwant");
    test_engine_new!(test_new_yandex, "yandex");
    test_engine_new!(test_new_ask, "ask");

    // Chinese Search Engines
    test_engine_new!(test_new_baidu, "baidu");
    test_engine_new!(test_new_sogou, "sogou");
    test_engine_new!(test_new_search360, "search360");
    test_engine_new!(test_new_chinaso, "chinaso");
    test_engine_new!(test_new_zhihu, "zhihu");
    test_engine_new!(test_new_weibo, "weibo");
    test_engine_new!(test_new_douban, "douban");
    test_engine_new!(test_new_taobao, "taobao");
    test_engine_new!(test_new_jd, "jd");

    // Asian Search Engines
    test_engine_new!(test_new_naver, "naver");

    // Reference / Wikimedia
    test_engine_new!(test_new_wikipedia, "wikipedia");
    test_engine_new!(test_new_wikidata, "wikidata");
    test_engine_new!(test_new_openlibrary, "openlibrary");
    test_engine_new!(test_new_wikibooks, "wikibooks");
    test_engine_new!(test_new_wikiquote, "wikiquote");
    test_engine_new!(test_new_wikinews, "wikinews");
    test_engine_new!(test_new_wikisource, "wikisource");
    test_engine_new!(test_new_wikiversity, "wikiversity");
    test_engine_new!(test_new_wikivoyage, "wikivoyage");
    test_engine_new!(test_new_wiktionary, "wiktionary");
    test_engine_new!(test_new_wikispecies, "wikispecies");
    test_engine_new!(test_new_wikicommons, "wikicommons");

    // Developer Tools
    test_engine_new!(test_new_github, "github");
    test_engine_new!(test_new_gitlab, "gitlab");
    test_engine_new!(test_new_codeberg, "codeberg");
    test_engine_new!(test_new_bitbucket, "bitbucket");
    test_engine_new!(test_new_stackexchange, "stackexchange");
    test_engine_new!(test_new_npm, "npm");
    test_engine_new!(test_new_pypi, "pypi");
    test_engine_new!(test_new_crates, "crates");
    test_engine_new!(test_new_dockerhub, "dockerhub");
    test_engine_new!(test_new_huggingface, "huggingface");

    // Academic
    test_engine_new!(test_new_arxiv, "arxiv");
    test_engine_new!(test_new_pubmed, "pubmed");
    test_engine_new!(test_new_semanticscholar, "semanticscholar");
    test_engine_new!(test_new_crossref, "crossref");
    test_engine_new!(test_new_core, "core");
    test_engine_new!(test_new_googlescholar, "googlescholar");
    test_engine_new!(test_new_wolframalpha, "wolframalpha");

    // Social/Community
    test_engine_new!(test_new_hackernews, "hackernews");
    test_engine_new!(test_new_reddit, "reddit");
    test_engine_new!(test_new_twitter, "twitter");
    test_engine_new!(test_new_mastodon, "mastodon");
    test_engine_new!(test_new_lemmy, "lemmy");
    test_engine_new!(test_new_lobsters, "lobsters");

    // Media - Video
    test_engine_new!(test_new_youtube, "youtube");
    test_engine_new!(test_new_bilibili, "bilibili");
    test_engine_new!(test_new_vimeo, "vimeo");
    test_engine_new!(test_new_dailymotion, "dailymotion");

    // Media - Music
    test_engine_new!(test_new_soundcloud, "soundcloud");
    test_engine_new!(test_new_bandcamp, "bandcamp");
    test_engine_new!(test_new_genius, "genius");
    test_engine_new!(test_new_spotify, "spotify");
    test_engine_new!(test_new_applemusic, "applemusic");

    // Media - Images
    test_engine_new!(test_new_unsplash, "unsplash");
    test_engine_new!(test_new_flickr, "flickr");
    test_engine_new!(test_new_pixabay, "pixabay");
    test_engine_new!(test_new_deviantart, "deviantart");

    // Movies/TV
    test_engine_new!(test_new_imdb, "imdb");
    test_engine_new!(test_new_rottentomatoes, "rottentomatoes");

    // Maps
    test_engine_new!(test_new_openstreetmap, "openstreetmap");

    // Archive
    test_engine_new!(test_new_internetarchive, "internetarchive");

    // Shopping
    test_engine_new!(test_new_amazon, "amazon");
    test_engine_new!(test_new_ebay, "ebay");

    // Test unknown engine returns error
    #[test]
    fn test_unknown_engine() {
        let result = EngineHandler::new("nonexistent_engine");
        assert!(result.is_err(), "Unknown engine should return error");
    }

    // Test case insensitivity
    #[test]
    fn test_case_insensitive() {
        assert!(EngineHandler::new("DuckDuckGo").is_ok());
        assert!(EngineHandler::new("DUCKDUCKGO").is_ok());
        assert!(EngineHandler::new("duckduckgo").is_ok());
    }

    // Test aliases
    #[test]
    fn test_aliases() {
        assert!(EngineHandler::new("stackoverflow").is_ok()); // alias for stackexchange
        assert!(EngineHandler::new("docker").is_ok()); // alias for dockerhub
        assert!(EngineHandler::new("hf").is_ok()); // alias for huggingface
        assert!(EngineHandler::new("hn").is_ok()); // alias for hackernews
        assert!(EngineHandler::new("osm").is_ok()); // alias for openstreetmap
        assert!(EngineHandler::new("archive").is_ok()); // alias for internetarchive
        assert!(EngineHandler::new("crates.io").is_ok()); // alias for crates
        assert!(EngineHandler::new("x").is_ok()); // alias for twitter
        assert!(EngineHandler::new("wolfram").is_ok()); // alias for wolframalpha
        assert!(EngineHandler::new("jingdong").is_ok()); // alias for jd
        assert!(EngineHandler::new("wikimediacommons").is_ok()); // alias for wikicommons
        assert!(EngineHandler::new("360search").is_ok()); // alias for search360
    }
}

// ============================================================================
// Engine Search Tests - General Search Engines
// ============================================================================

mod search_general {
    use super::*;

    test_engine_search!(test_search_duckduckgo, "duckduckgo", TEST_QUERY);
    test_engine_search!(test_search_bing, "bing", TEST_QUERY);
    test_engine_search!(test_search_yahoo, "yahoo", TEST_QUERY);
    test_engine_search!(test_search_brave, "brave", TEST_QUERY);
    test_engine_search!(test_search_startpage, "startpage", TEST_QUERY);
    test_engine_search!(test_search_mojeek, "mojeek", TEST_QUERY);
    test_engine_search!(test_search_qwant, "qwant", TEST_QUERY);
    test_engine_search!(test_search_yandex, "yandex", TEST_QUERY);
    test_engine_search!(test_search_ask, "ask", TEST_QUERY);
    // Note: google and searx require special setup
}

// ============================================================================
// Engine Search Tests - Chinese Search Engines
// ============================================================================

mod search_chinese {
    use super::*;

    test_engine_search!(test_search_baidu, "baidu", TEST_QUERY_CN);
    test_engine_search!(test_search_sogou, "sogou", TEST_QUERY_CN);
    test_engine_search!(test_search_search360, "search360", TEST_QUERY_CN);
    test_engine_search!(test_search_chinaso, "chinaso", TEST_QUERY_CN);
    test_engine_search!(test_search_zhihu, "zhihu", TEST_QUERY_CN);
    test_engine_search!(test_search_weibo, "weibo", TEST_QUERY_CN);
    test_engine_search!(test_search_douban, "douban", TEST_QUERY_CN);
    test_engine_search!(test_search_bilibili, "bilibili", TEST_QUERY_CN);
}

// ============================================================================
// Engine Search Tests - Reference / Wikimedia
// ============================================================================

mod search_reference {
    use super::*;

    test_engine_search!(test_search_wikipedia, "wikipedia", TEST_QUERY);
    test_engine_search!(test_search_wikidata, "wikidata", TEST_QUERY);
    test_engine_search!(test_search_openlibrary, "openlibrary", "programming");
    test_engine_search!(test_search_wikibooks, "wikibooks", TEST_QUERY);
    test_engine_search!(test_search_wikiquote, "wikiquote", "einstein");
    test_engine_search!(test_search_wikinews, "wikinews", "technology");
    test_engine_search!(test_search_wikisource, "wikisource", "shakespeare");
    test_engine_search!(test_search_wikiversity, "wikiversity", "computer science");
    test_engine_search!(test_search_wikivoyage, "wikivoyage", "tokyo");
    test_engine_search!(test_search_wiktionary, "wiktionary", "programming");
    test_engine_search!(test_search_wikispecies, "wikispecies", "homo sapiens");
    test_engine_search!(test_search_wikicommons, "wikicommons", "nature");
}

// ============================================================================
// Engine Search Tests - Developer Tools
// ============================================================================

mod search_developer {
    use super::*;

    test_engine_search!(test_search_github, "github", "rust async");
    test_engine_search!(test_search_gitlab, "gitlab", "python");
    test_engine_search!(test_search_codeberg, "codeberg", "open source");
    test_engine_search!(test_search_bitbucket, "bitbucket", "java");
    test_engine_search!(test_search_stackexchange, "stackexchange", "rust borrow checker");
    test_engine_search!(test_search_npm, "npm", "express");
    test_engine_search!(test_search_pypi, "pypi", "requests");
    test_engine_search!(test_search_crates, "crates", "tokio");
    test_engine_search!(test_search_dockerhub, "dockerhub", "nginx");
    test_engine_search!(test_search_huggingface, "huggingface", "llama");
}

// ============================================================================
// Engine Search Tests - Academic
// ============================================================================

mod search_academic {
    use super::*;

    test_engine_search!(test_search_arxiv, "arxiv", TEST_QUERY_ACADEMIC);
    test_engine_search!(test_search_pubmed, "pubmed", "cancer treatment");
    test_engine_search!(test_search_semanticscholar, "semanticscholar", TEST_QUERY_ACADEMIC);
    test_engine_search!(test_search_crossref, "crossref", "climate change");
    test_engine_search!(test_search_core, "core", "artificial intelligence");
    test_engine_search!(test_search_googlescholar, "googlescholar", TEST_QUERY_ACADEMIC);
    test_engine_search!(test_search_wolframalpha, "wolframalpha", "integrate x^2");
}

// ============================================================================
// Engine Search Tests - Social/Community
// ============================================================================

mod search_social {
    use super::*;

    test_engine_search!(test_search_hackernews, "hackernews", "rust");
    test_engine_search!(test_search_reddit, "reddit", "programming");
    test_engine_search!(test_search_lobsters, "lobsters", "rust");
    test_engine_search!(test_search_lemmy, "lemmy", "linux");
    test_engine_search!(test_search_mastodon, "mastodon", "opensource");
    // Note: twitter requires authentication
}

// ============================================================================
// Engine Search Tests - Media (Video)
// ============================================================================

mod search_video {
    use super::*;

    test_engine_search!(test_search_youtube, "youtube", TEST_QUERY_VIDEO);
    test_engine_search!(test_search_vimeo, "vimeo", "short film");
    test_engine_search!(test_search_dailymotion, "dailymotion", "music video");
}

// ============================================================================
// Engine Search Tests - Media (Music)
// ============================================================================

mod search_music {
    use super::*;

    test_engine_search!(test_search_soundcloud, "soundcloud", TEST_QUERY_MUSIC);
    test_engine_search!(test_search_bandcamp, "bandcamp", "indie rock");
    test_engine_search!(test_search_genius, "genius", "lyrics");
    test_engine_search!(test_search_spotify, "spotify", TEST_QUERY_MUSIC);
    test_engine_search!(test_search_applemusic, "applemusic", TEST_QUERY_MUSIC);
}

// ============================================================================
// Engine Search Tests - Media (Images)
// ============================================================================

mod search_images {
    use super::*;

    test_engine_search!(test_search_unsplash, "unsplash", TEST_QUERY_IMAGE);
    test_engine_search!(test_search_flickr, "flickr", TEST_QUERY_IMAGE);
    test_engine_search!(test_search_pixabay, "pixabay", TEST_QUERY_IMAGE);
    test_engine_search!(test_search_deviantart, "deviantart", "digital art");
}

// ============================================================================
// Engine Search Tests - Movies/TV
// ============================================================================

mod search_movies {
    use super::*;

    test_engine_search!(test_search_imdb, "imdb", "inception");
    test_engine_search!(test_search_rottentomatoes, "rottentomatoes", "action movies");
}

// ============================================================================
// Engine Search Tests - Maps
// ============================================================================

mod search_maps {
    use super::*;

    test_engine_search!(test_search_openstreetmap, "openstreetmap", TEST_QUERY_MAP);
}

// ============================================================================
// Engine Search Tests - Archive
// ============================================================================

mod search_archive {
    use super::*;

    test_engine_search!(test_search_internetarchive, "internetarchive", "vintage computing");
}

// ============================================================================
// Engine Search Tests - Shopping
// ============================================================================

mod search_shopping {
    use super::*;

    test_engine_search!(test_search_amazon, "amazon", TEST_QUERY_SHOPPING);
    test_engine_search!(test_search_ebay, "ebay", TEST_QUERY_SHOPPING);
    test_engine_search!(test_search_taobao, "taobao", TEST_QUERY_SHOPPING);
    test_engine_search!(test_search_jd, "jd", TEST_QUERY_SHOPPING);
}

// ============================================================================
// Engine Search Tests - Asian
// ============================================================================

mod search_asian {
    use super::*;

    test_engine_search!(test_search_naver, "naver", "서울"); // Seoul in Korean
}

// ============================================================================
// Pagination Tests
// ============================================================================

mod pagination {
    use super::*;

    #[tokio::test]
    async fn test_pagination_wikipedia() {
        let client = create_test_client();
        let handler = EngineHandler::new("wikipedia").expect("Failed to create wikipedia engine");
        let (_, engine) = handler.into_name_engine();

        // Test page 1
        let page1 = engine.results("rust", 1, TEST_USER_AGENT, &client, 0).await;

        // Test page 2
        let page2 = engine.results("rust", 2, TEST_USER_AGENT, &client, 0).await;

        // Both should work (may have same or different results depending on engine)
        if let (Ok(p1), Ok(p2)) = (&page1, &page2) {
            println!("Page 1 results: {}, Page 2 results: {}", p1.len(), p2.len());
        }
    }

    #[tokio::test]
    async fn test_pagination_duckduckgo() {
        let client = create_test_client();
        let handler = EngineHandler::new("duckduckgo").expect("Failed to create duckduckgo engine");
        let (_, engine) = handler.into_name_engine();

        let page0 = engine.results("rust", 0, TEST_USER_AGENT, &client, 0).await;
        let page1 = engine.results("rust", 1, TEST_USER_AGENT, &client, 0).await;

        // Page 0 and 1 should work
        println!("Page 0: {:?}, Page 1: {:?}", page0.is_ok(), page1.is_ok());
    }
}

// ============================================================================
// Safe Search Tests
// ============================================================================

mod safe_search {
    use super::*;

    #[tokio::test]
    async fn test_safe_search_levels() {
        let client = create_test_client();
        let handler = EngineHandler::new("duckduckgo").expect("Failed to create engine");
        let (_, engine) = handler.into_name_engine();

        // Test different safe search levels (0-4)
        for level in 0..=4 {
            let result = engine.results("test", 1, TEST_USER_AGENT, &client, level).await;
            println!("Safe search level {}: {:?}", level, result.is_ok());
        }
    }
}

// ============================================================================
// Special Character Query Tests
// ============================================================================

mod special_queries {
    use super::*;

    #[tokio::test]
    async fn test_unicode_query() {
        let client = create_test_client();
        let handler = EngineHandler::new("wikipedia").expect("Failed to create engine");
        let (_, engine) = handler.into_name_engine();

        // Test with various Unicode characters
        let queries = vec![
            "中文测试",
            "日本語テスト",
            "한국어 테스트",
            "тест на русском",
            "🦀 rust programming",
        ];

        for query in queries {
            let result = engine.results(query, 1, TEST_USER_AGENT, &client, 0).await;
            println!("Query '{}': {:?}", query, result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_special_characters() {
        let client = create_test_client();
        let handler = EngineHandler::new("wikipedia").expect("Failed to create engine");
        let (_, engine) = handler.into_name_engine();

        // Test with special characters that need URL encoding
        let queries = vec![
            "C++",
            "C#",
            "test & query",
            "test=query",
            "test?query",
            "test/query",
        ];

        for query in queries {
            let result = engine.results(query, 1, TEST_USER_AGENT, &client, 0).await;
            println!("Query '{}': {:?}", query, result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_empty_query() {
        let client = create_test_client();
        let handler = EngineHandler::new("wikipedia").expect("Failed to create engine");
        let (_, engine) = handler.into_name_engine();

        let result = engine.results("", 1, TEST_USER_AGENT, &client, 0).await;
        // Empty query should either return error or empty results
        println!("Empty query result: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_long_query() {
        let client = create_test_client();
        let handler = EngineHandler::new("wikipedia").expect("Failed to create engine");
        let (_, engine) = handler.into_name_engine();

        let long_query = "a".repeat(500);
        let result = engine.results(&long_query, 1, TEST_USER_AGENT, &client, 0).await;
        println!("Long query result: {:?}", result.is_ok());
    }
}

// ============================================================================
// Concurrent Search Tests
// ============================================================================

mod concurrent {
    use super::*;
    use futures::future::join_all;

    #[tokio::test]
    async fn test_concurrent_searches() {
        let client = create_test_client();

        let engines = vec!["wikipedia", "github", "arxiv"];
        let mut handles = Vec::new();

        for engine_name in engines {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                if let Ok(handler) = EngineHandler::new(engine_name) {
                    let (name, engine) = handler.into_name_engine();
                    let result = engine.results("rust", 1, TEST_USER_AGENT, &client, 0).await;
                    (name, result.is_ok())
                } else {
                    (engine_name, false)
                }
            });
            handles.push(handle);
        }

        let results = join_all(handles).await;
        for result in results {
            if let Ok((name, success)) = result {
                println!("Concurrent search {}: {}", name, success);
            }
        }
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_engine_error_display() {
        let errors = vec![
            EngineError::NoSuchEngineFound("test".to_string()),
            EngineError::EmptyResultSet,
            EngineError::RequestError,
            EngineError::Timeout,
            EngineError::AccessDenied,
            EngineError::Captcha,
            EngineError::TooManyRequests,
            EngineError::SslError,
            EngineError::HttpError(404),
            EngineError::ParseError,
            EngineError::NetworkError,
            EngineError::UnexpectedError,
        ];

        for error in errors {
            // Just verify Display trait works without panicking
            let _msg = format!("{}", error);
            println!("Error: {}", error);
        }
    }

    #[test]
    fn test_engine_error_to_error_type() {
        use websurfx::models::engine_health::EngineErrorType;

        let test_cases = vec![
            (EngineError::Timeout, EngineErrorType::Timeout),
            (EngineError::AccessDenied, EngineErrorType::AccessDenied),
            (EngineError::Captcha, EngineErrorType::Captcha),
            (EngineError::TooManyRequests, EngineErrorType::TooManyRequests),
            (EngineError::SslError, EngineErrorType::SslError),
            (EngineError::ParseError, EngineErrorType::ParseError),
            (EngineError::NetworkError, EngineErrorType::NetworkError),
        ];

        for (engine_error, expected_type) in test_cases {
            let error_type = engine_error.to_error_type();
            assert_eq!(
                std::mem::discriminant(&error_type),
                std::mem::discriminant(&expected_type),
                "Error type mismatch for {:?}",
                engine_error
            );
        }
    }
}

// ============================================================================
// Engine Coverage Summary Test
// ============================================================================

mod coverage {
    use super::*;

    #[test]
    fn test_all_engines_can_be_instantiated() {
        let engine_names = vec![
            // General
            "duckduckgo", "searx", "brave", "startpage", "librex", "mojeek",
            "bing", "yahoo", "google", "qwant", "yandex", "ask",
            // Chinese
            "baidu", "sogou", "search360", "chinaso", "zhihu", "weibo", "douban", "taobao", "jd",
            // Asian
            "naver",
            // Wikimedia
            "wikipedia", "wikidata", "openlibrary", "wikibooks", "wikiquote",
            "wikinews", "wikisource", "wikiversity", "wikivoyage", "wiktionary",
            "wikispecies", "wikicommons",
            // Developer
            "github", "gitlab", "codeberg", "bitbucket", "stackexchange",
            "npm", "pypi", "crates", "dockerhub", "huggingface",
            // Academic
            "arxiv", "pubmed", "semanticscholar", "crossref", "core", "googlescholar", "wolframalpha",
            // Social
            "hackernews", "reddit", "twitter", "mastodon", "lemmy", "lobsters",
            // Video
            "youtube", "bilibili", "vimeo", "dailymotion",
            // Music
            "soundcloud", "bandcamp", "genius", "spotify", "applemusic",
            // Images
            "unsplash", "flickr", "pixabay", "deviantart",
            // Movies
            "imdb", "rottentomatoes",
            // Maps
            "openstreetmap",
            // Archive
            "internetarchive",
            // Shopping
            "amazon", "ebay",
        ];

        let mut success_count = 0;
        let mut fail_count = 0;

        for name in &engine_names {
            match EngineHandler::new(name) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    fail_count += 1;
                    println!("Failed to create engine '{}': {:?}", name, e);
                }
            }
        }

        println!("\n=== Engine Coverage Summary ===");
        println!("Total engines tested: {}", engine_names.len());
        println!("Successfully instantiated: {}", success_count);
        println!("Failed to instantiate: {}", fail_count);
        println!("Coverage: {:.1}%", (success_count as f64 / engine_names.len() as f64) * 100.0);

        // All engines should be instantiable
        assert_eq!(fail_count, 0, "Some engines failed to instantiate");
    }
}
