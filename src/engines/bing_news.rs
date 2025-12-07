//! The `bing_news` module handles the scraping of results from Bing News
//! by querying the upstream Bing news search with user provided query.
//! This engine extracts thumbnails and other metadata for SearXNG compatibility.

use std::collections::HashMap;

use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use log::info;

use crate::models::aggregation::SearchResult;
use crate::models::engine::{EngineError, SearchEngine};

use error_stack::{Report, Result, ResultExt};

/// Bing News engine for fetching news articles with thumbnails
pub struct BingNews;

impl BingNews {
    /// Creates a new BingNews engine instance
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self)
    }
}

#[async_trait::async_trait]
impl SearchEngine for BingNews {
    async fn results(
        &self,
        query: &str,
        page: u32,
        _user_agent: &str,  // Ignore passed user agent, use our own
        _client: &reqwest::Client,  // We'll create our own client to control redirects
        _safe_search: u8,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        // Use a fixed Chrome user agent that works well with Bing News
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        // Bing News uses the infinitescrollajax endpoint (same as SearXNG)
        // This returns HTML fragments with newsitem divs
        // Note: Use global.bing.com to try to get international news
        // If in China, this may redirect to cn.bing.com which has no news
        let first = page * 10 + 1;
        let url = format!(
            "https://www.bing.com/news/infinitescrollajax?q={}&InfiniteScroll=1&first={}&SFX={}&form=PTFTNR&setlang=en&mkt=en-US",
            urlencoding::encode(query),
            first,
            page
        );

        info!("[bing_news] Fetching URL: {}", url);

        // Set comprehensive cookies to try to force US market
        // Include MUIDB to help with geolocation override
        let cookie_string = "_EDGE_CD=m=en-US&u=en; _EDGE_S=mkt=en-US&ui=en; MUID=0; SRCHD=AF=NOFORM; SRCHHPGUSR=SRCHLANG=en&BRW=XW&BRH=M&CW=800&CH=600&SCW=800&SCH=600&DPR=1&UTC=480&DM=0; _EDGE_V=1; MUIDB=0";

        let header_map = HeaderMap::try_from(&HashMap::from([
            ("User-Agent".to_string(), user_agent.to_string()),
            ("Referer".to_string(), "https://www.bing.com/news".to_string()),
            (
                "Accept".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string(),
            ),
            ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
            ("Cookie".to_string(), cookie_string.to_string()),
            // Try to override geographic detection
            ("X-Forwarded-For".to_string(), "8.8.8.8".to_string()),
            ("CF-Connecting-IP".to_string(), "8.8.8.8".to_string()),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        // Create a custom client that follows redirects (to handle geographic redirects)
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .change_context(EngineError::UnexpectedError)?;

        // Make the request
        let response = client
            .get(&url)
            .headers(header_map)
            .send()
            .await
            .change_context(EngineError::RequestError)?;

        info!("[bing_news] Response status: {}, final URL: {}", response.status(), response.url());

        let html = response
            .text()
            .await
            .change_context(EngineError::RequestError)?;

        info!("[bing_news] Fetched HTML length: {}, contains newsitem: {}", html.len(), html.contains("newsitem"));

        // Debug: check first 100 chars
        if !html.is_empty() {
            let sample = if html.len() > 200 { &html[..200] } else { &html };
            info!("[bing_news] HTML start: {}", sample.replace('\n', " "));
        }

        let document = Html::parse_document(&html);

        // For the infinitescrollajax endpoint, the structure is:
        // <div class="newsitem"> or <div class="news-card newsitem">
        //   <a class="title" href="...">Title</a>
        //   <div class="snippet">Description...</div>
        //   <a class="imagelink"><img src="..." /></a>
        // </div>

        let newsitem_selector = Selector::parse("div.newsitem, div.news-card").ok();
        let title_selector = Selector::parse("a.title").ok();
        let snippet_selector = Selector::parse(".snippet").ok();
        let source_selector = Selector::parse(".source a").ok();
        let time_selector = Selector::parse("span[aria-label]").ok();
        // Multiple possible image locations
        let img_selector = Selector::parse("img").ok();

        let mut results: Vec<(String, SearchResult)> = Vec::new();

        if let Some(ref item_sel) = newsitem_selector {
            let item_count = document.select(item_sel).count();
            info!("[bing_news] Found {} news items", item_count);

            for item in document.select(item_sel) {
                // Get title and URL from title link
                let title_el = title_selector.as_ref()
                    .and_then(|sel| item.select(sel).next());

                let title = title_el.as_ref()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string());

                let href = title_el.as_ref()
                    .and_then(|el| el.value().attr("href"))
                    .map(|s| s.to_string());

                // Also try to get author from data-author attribute on title link
                let author_from_attr = title_el.as_ref()
                    .and_then(|el| el.value().attr("data-author"))
                    .map(|s| html_escape::decode_html_entities(s).to_string());

                let description = snippet_selector.as_ref()
                    .and_then(|sel| item.select(sel).next())
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                // Extract thumbnail - try multiple sources
                let thumbnail = img_selector.as_ref()
                    .and_then(|sel| {
                        for img in item.select(sel) {
                            // Try different attributes in order of preference
                            if let Some(src) = img.value().attr("data-src-hq")
                                .or_else(|| img.value().attr("data-src"))
                                .or_else(|| img.value().attr("src"))
                            {
                                let url = src.to_string();
                                // Skip data URIs and empty strings
                                if !url.is_empty() && !url.starts_with("data:") {
                                    // Convert protocol-relative URLs to HTTPS
                                    let final_url = if url.starts_with("//") {
                                        format!("https:{}", url)
                                    } else if !url.starts_with("http") && !url.starts_with("/") {
                                        format!("https://www.bing.com/{}", url)
                                    } else if url.starts_with("/") {
                                        format!("https://www.bing.com{}", url)
                                    } else {
                                        url
                                    };
                                    return Some(final_url);
                                }
                            }
                        }
                        None
                    });

                let source = author_from_attr.or_else(|| {
                    source_selector.as_ref()
                        .and_then(|sel| item.select(sel).next())
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                });

                let published_date = time_selector.as_ref()
                    .and_then(|sel| item.select(sel).next())
                    .and_then(|el| el.value().attr("aria-label").map(|s| s.to_string()));

                if let (Some(title), Some(href)) = (title, href) {
                    if !title.is_empty() && !href.is_empty() && href.starts_with("http") {
                        info!("[bing_news] Adding result: {} (thumbnail: {})",
                            title.chars().take(50).collect::<String>(),
                            thumbnail.is_some()
                        );

                        let mut result = SearchResult::new(
                            &title,
                            &href,
                            &description,
                            &["bing news"],
                        );
                        result.thumbnail = thumbnail;
                        result.source = source;
                        result.published_date = published_date;
                        result.category = Some("news".to_string());
                        result.template = Some("default.html".to_string());

                        results.push((href, result));
                    }
                }
            }
        }

        info!("[bing_news] Total results collected: {}", results.len());

        if results.is_empty() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        Ok(results)
    }
}
