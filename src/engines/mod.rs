//! This module provides different modules which handles the functionlity to fetch results from the
//! upstream search engines based on user requested queries. Also provides different models to
//! provide a standard functions to be implemented for all the upstream search engine handling
//! code. Moreover, it also provides a custom error for the upstream search engine handling code.

// General search engines
pub mod baidu;
pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod google;
pub mod librex;
pub mod mojeek;
pub mod searx;
pub mod sogou;
pub mod startpage;
pub mod yahoo;
pub mod qwant;
pub mod yandex;
pub mod ask;

// Chinese search engines
pub mod search360;
pub mod chinaso;
pub mod zhihu;
pub mod weibo;
pub mod douban;
pub mod taobao;
pub mod jd;

// Asian search engines
pub mod naver;

// Reference / Wikimedia
pub mod wikipedia;
pub mod wikidata;
pub mod openlibrary;
pub mod wikibooks;
pub mod wikiquote;
pub mod wikinews;
pub mod wikisource;
pub mod wikiversity;
pub mod wikivoyage;
pub mod wiktionary;
pub mod wikispecies;
pub mod wikicommons;

// Developer tools
pub mod github;
pub mod gitlab;
pub mod codeberg;
pub mod bitbucket;
pub mod stackexchange;
pub mod npm;
pub mod pypi;
pub mod crates;
pub mod dockerhub;
pub mod huggingface;

// Academic
pub mod arxiv;
pub mod pubmed;
pub mod semanticscholar;
pub mod crossref;
pub mod core;
pub mod googlescholar;
pub mod wolframalpha;

// Social/Community
pub mod hackernews;
pub mod reddit;
pub mod twitter;
pub mod mastodon;
pub mod lemmy;
pub mod lobsters;

// Media - Video
pub mod youtube;
pub mod bilibili;
pub mod vimeo;
pub mod dailymotion;

// Media - Music
pub mod soundcloud;
pub mod bandcamp;
pub mod genius;
pub mod spotify;
pub mod applemusic;

// Media - Images
pub mod unsplash;
pub mod flickr;
pub mod pixabay;
pub mod deviantart;

// Movies/TV
pub mod imdb;
pub mod rottentomatoes;

// Maps
pub mod openstreetmap;

// Archive
pub mod internetarchive;

// Shopping
pub mod amazon;
pub mod ebay;

mod search_result_parser;

/// Build a query from a list of key value pairs.
///
/// # Arguments
///
/// * `query_params` - Takes the query parameters key value a slice of tuples of type string.
///
/// # Returns
///
/// It returns the query key value pairs formatted in the url query parameter format.
fn build_query(query_params: &[(&str, &str)]) -> String {
    let mut query_params_string = String::new();
    for (k, v) in query_params {
        query_params_string.push_str(&format!("&{k}={v}"));
    }
    query_params_string
}

/// Build a cookie from a list of key value pairs.
///
/// # Arguments
///
/// * `cookie_params` - Takes the query parameters key value a slice of tuples of type string.
///
/// # Returns
///
/// It returns the query key value pairs formatted in the cookie key value format.
fn build_cookie(cookie_params: &[(&str, &str)]) -> String {
    let mut cookie_string = String::new();
    for (k, v) in cookie_params {
        cookie_string.push_str(&format!("{k}={v}; "));
    }
    cookie_string
}
