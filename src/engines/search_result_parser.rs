//! This modules provides helper functionalities for parsing a html document into internal SearchResult.

use crate::models::{aggregation::SearchResult, engine::EngineError};
use error_stack::{Report, Result};
use scraper::{ElementRef, Html, Selector, html::Select};

/// A html search result parser, based on a predefined CSS selectors.
pub struct SearchResultParser {
    /// selector to locate the element which is displayed, if there were nothing found.
    no_result: Selector,
    /// selector to locate the element which contains one item from the search result.
    results: Selector,
    /// selector to locate the title relative to the search result item.
    result_title: Selector,
    /// selector to locate the url relative to the search result item.
    result_url: Selector,
    /// selector to locate the description relative to the search result item.
    result_desc: Selector,
    /// Optional selector to locate the thumbnail image
    result_thumbnail: Option<Selector>,
    /// Optional selector to locate the full-size image
    result_img: Option<Selector>,
    /// Optional selector to locate the published date
    result_date: Option<Selector>,
    /// Optional selector to locate the author
    result_author: Option<Selector>,
    /// Optional selector to locate the source/domain
    result_source: Option<Selector>,
    /// Optional selector to locate view count
    result_views: Option<Selector>,
    /// Optional selector to locate duration/length
    result_length: Option<Selector>,
    /// Optional selector to locate additional metadata
    result_metadata: Option<Selector>,
}

impl SearchResultParser {
    /// Creates a new parser with basic selectors only
    pub fn new(
        no_result_selector: &str,
        results_selector: &str,
        result_title_selector: &str,
        result_url_selector: &str,
        result_desc_selector: &str,
    ) -> Result<SearchResultParser, EngineError> {
        Ok(SearchResultParser {
            no_result: new_selector(no_result_selector)?,
            results: new_selector(results_selector)?,
            result_title: new_selector(result_title_selector)?,
            result_url: new_selector(result_url_selector)?,
            result_desc: new_selector(result_desc_selector)?,
            result_thumbnail: None,
            result_img: None,
            result_date: None,
            result_author: None,
            result_source: None,
            result_views: None,
            result_length: None,
            result_metadata: None,
        })
    }

    /// Creates a new parser with extended selectors for SearXNG compatibility
    pub fn new_extended(
        no_result_selector: &str,
        results_selector: &str,
        result_title_selector: &str,
        result_url_selector: &str,
        result_desc_selector: &str,
        result_thumbnail_selector: Option<&str>,
        result_img_selector: Option<&str>,
        result_date_selector: Option<&str>,
        result_author_selector: Option<&str>,
        result_source_selector: Option<&str>,
    ) -> Result<SearchResultParser, EngineError> {
        Ok(SearchResultParser {
            no_result: new_selector(no_result_selector)?,
            results: new_selector(results_selector)?,
            result_title: new_selector(result_title_selector)?,
            result_url: new_selector(result_url_selector)?,
            result_desc: new_selector(result_desc_selector)?,
            result_thumbnail: result_thumbnail_selector.map(new_selector).transpose()?,
            result_img: result_img_selector.map(new_selector).transpose()?,
            result_date: result_date_selector.map(new_selector).transpose()?,
            result_author: result_author_selector.map(new_selector).transpose()?,
            result_source: result_source_selector.map(new_selector).transpose()?,
            result_views: None,
            result_length: None,
            result_metadata: None,
        })
    }

    /// Creates a builder for constructing a parser with custom selectors
    pub fn builder(
        no_result_selector: &str,
        results_selector: &str,
        result_title_selector: &str,
        result_url_selector: &str,
        result_desc_selector: &str,
    ) -> Result<SearchResultParserBuilder, EngineError> {
        SearchResultParserBuilder::new(
            no_result_selector,
            results_selector,
            result_title_selector,
            result_url_selector,
            result_desc_selector,
        )
    }

    /// Parse the html and returns element representing the 'no result found' response.
    pub fn parse_for_no_results<'a>(&'a self, document: &'a Html) -> Select<'a, 'a> {
        document.select(&self.no_result)
    }

    /// Parse the html, and convert the results to SearchResult with the help of the builder function
    pub fn parse_for_results(
        &self,
        document: &Html,
        builder: impl Fn(&ElementRef<'_>, &ElementRef<'_>, &ElementRef<'_>) -> Option<SearchResult>,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let res = document
            .select(&self.results)
            .filter_map(|result| {
                let title = result.select(&self.result_title).next();
                let url = result.select(&self.result_url).next();
                let desc = result.select(&self.result_desc).next();

                if let (Some(ref t), Some(ref u), Some(ref d)) = (title, url, desc) {
                    return builder(t, u, d);
                }

                None
            })
            .map(|search_result| (search_result.url.clone(), search_result))
            .collect();
        Ok(res)
    }

    /// Parse the html with extended fields (thumbnail, date, author, etc.)
    /// Returns results with all available SearXNG-compatible fields populated
    pub fn parse_for_results_extended(
        &self,
        document: &Html,
        builder: impl Fn(ParsedResult<'_>) -> Option<SearchResult>,
    ) -> Result<Vec<(String, SearchResult)>, EngineError> {
        let res = document
            .select(&self.results)
            .filter_map(|result| {
                let title = result.select(&self.result_title).next();
                let url = result.select(&self.result_url).next();
                let desc = result.select(&self.result_desc).next();

                // Extract optional fields
                let thumbnail = self.result_thumbnail.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let img = self.result_img.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let date = self.result_date.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let author = self.result_author.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let source = self.result_source.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let views = self.result_views.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let length = self.result_length.as_ref()
                    .and_then(|sel| result.select(sel).next());
                let metadata = self.result_metadata.as_ref()
                    .and_then(|sel| result.select(sel).next());

                if let (Some(ref t), Some(ref u), Some(ref d)) = (title, url, desc) {
                    let parsed = ParsedResult {
                        title: t,
                        url: u,
                        description: d,
                        thumbnail,
                        img,
                        date,
                        author,
                        source,
                        views,
                        length,
                        metadata,
                        result_element: &result,
                    };
                    return builder(parsed);
                }

                None
            })
            .map(|search_result| (search_result.url.clone(), search_result))
            .collect();
        Ok(res)
    }
}

/// Parsed result containing all available elements from a search result
pub struct ParsedResult<'a> {
    pub title: &'a ElementRef<'a>,
    pub url: &'a ElementRef<'a>,
    pub description: &'a ElementRef<'a>,
    pub thumbnail: Option<ElementRef<'a>>,
    pub img: Option<ElementRef<'a>>,
    pub date: Option<ElementRef<'a>>,
    pub author: Option<ElementRef<'a>>,
    pub source: Option<ElementRef<'a>>,
    pub views: Option<ElementRef<'a>>,
    pub length: Option<ElementRef<'a>>,
    pub metadata: Option<ElementRef<'a>>,
    pub result_element: &'a ElementRef<'a>,
}

impl<'a> ParsedResult<'a> {
    /// Get the inner HTML of the title element, trimmed
    pub fn title_text(&self) -> String {
        self.title.inner_html().trim().to_string()
    }

    /// Get the inner HTML of the description element, trimmed
    pub fn description_text(&self) -> String {
        self.description.inner_html().trim().to_string()
    }

    /// Get the URL text, trimmed
    pub fn url_text(&self) -> String {
        self.url.inner_html().trim().to_string()
    }

    /// Get an attribute from the thumbnail element
    pub fn thumbnail_attr(&self, attr: &str) -> Option<String> {
        self.thumbnail.as_ref()
            .and_then(|el| el.value().attr(attr))
            .map(|s| s.to_string())
    }

    /// Get an attribute from the image element
    pub fn img_attr(&self, attr: &str) -> Option<String> {
        self.img.as_ref()
            .and_then(|el| el.value().attr(attr))
            .map(|s| s.to_string())
    }

    /// Get the date text, trimmed
    pub fn date_text(&self) -> Option<String> {
        self.date.as_ref()
            .map(|el| el.inner_html().trim().to_string())
    }

    /// Get the author text, trimmed
    pub fn author_text(&self) -> Option<String> {
        self.author.as_ref()
            .map(|el| el.inner_html().trim().to_string())
    }

    /// Get the source text, trimmed
    pub fn source_text(&self) -> Option<String> {
        self.source.as_ref()
            .map(|el| el.inner_html().trim().to_string())
    }

    /// Get the views text
    pub fn views_text(&self) -> Option<String> {
        self.views.as_ref()
            .map(|el| el.inner_html().trim().to_string())
    }

    /// Get the length/duration text
    pub fn length_text(&self) -> Option<String> {
        self.length.as_ref()
            .map(|el| el.inner_html().trim().to_string())
    }

    /// Get an attribute from the result element itself
    pub fn result_attr(&self, attr: &str) -> Option<String> {
        self.result_element.value().attr(attr).map(|s| s.to_string())
    }
}

/// Builder for SearchResultParser with fluent API
pub struct SearchResultParserBuilder {
    parser: SearchResultParser,
}

impl SearchResultParserBuilder {
    fn new(
        no_result_selector: &str,
        results_selector: &str,
        result_title_selector: &str,
        result_url_selector: &str,
        result_desc_selector: &str,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            parser: SearchResultParser::new(
                no_result_selector,
                results_selector,
                result_title_selector,
                result_url_selector,
                result_desc_selector,
            )?,
        })
    }

    /// Set the thumbnail selector
    pub fn thumbnail(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_thumbnail = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the full image selector
    pub fn img(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_img = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the date selector
    pub fn date(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_date = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the author selector
    pub fn author(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_author = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the source selector
    pub fn source(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_source = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the views selector
    pub fn views(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_views = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the length/duration selector
    pub fn length(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_length = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Set the metadata selector
    pub fn metadata(mut self, selector: &str) -> Result<Self, EngineError> {
        self.parser.result_metadata = Some(new_selector(selector)?);
        Ok(self)
    }

    /// Build the parser
    pub fn build(self) -> SearchResultParser {
        self.parser
    }
}

/// Create a Selector struct, if the given parameter is a valid css expression, otherwise convert it into an EngineError.
fn new_selector(selector: &str) -> Result<Selector, EngineError> {
    Selector::parse(selector).map_err(|err| {
        Report::new(EngineError::UnexpectedError).attach_printable(format!(
            "invalid CSS selector: {}, err: {:?}",
            selector, err
        ))
    })
}
