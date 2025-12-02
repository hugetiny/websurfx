//! This module provides Infobox functionality similar to SearXNG.
//! Infoboxes display summary information about search results (e.g., Wikipedia snippets).

#![allow(dead_code)]
#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An infobox displayed alongside search results
/// Similar to SearXNG's infobox structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Infobox {
    /// The title of the infobox
    pub infobox: String,

    /// Unique identifier
    pub id: String,

    /// Short summary/content
    pub content: Option<String>,

    /// Source engine name
    pub engine: String,

    /// URLs for more information
    pub urls: Vec<InfoboxUrl>,

    /// Image URL if available
    pub img_src: Option<String>,

    /// Key-value attributes
    pub attributes: Vec<InfoboxAttribute>,

    /// Related topics/entities
    pub relatedTopics: Vec<RelatedTopic>,
}

/// URL entry in an infobox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoboxUrl {
    pub title: String,
    pub url: String,
    pub official: bool,
}

/// Attribute entry in an infobox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoboxAttribute {
    pub label: String,
    pub value: String,
    pub entity: Option<String>,
}

/// Related topic in an infobox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedTopic {
    pub name: String,
    pub url: Option<String>,
}

impl Infobox {
    /// Create a new empty infobox
    pub fn new(title: &str, engine: &str) -> Self {
        Self {
            infobox: title.to_string(),
            id: format!("{}-{}", engine, title.to_lowercase().replace(' ', "-")),
            content: None,
            engine: engine.to_string(),
            urls: Vec::new(),
            img_src: None,
            attributes: Vec::new(),
            relatedTopics: Vec::new(),
        }
    }

    /// Add content to the infobox
    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    /// Add an image to the infobox
    pub fn with_image(mut self, img_src: &str) -> Self {
        self.img_src = Some(img_src.to_string());
        self
    }

    /// Add a URL to the infobox
    pub fn add_url(&mut self, title: &str, url: &str, official: bool) {
        self.urls.push(InfoboxUrl {
            title: title.to_string(),
            url: url.to_string(),
            official,
        });
    }

    /// Add an attribute to the infobox
    pub fn add_attribute(&mut self, label: &str, value: &str, entity: Option<&str>) {
        self.attributes.push(InfoboxAttribute {
            label: label.to_string(),
            value: value.to_string(),
            entity: entity.map(String::from),
        });
    }

    /// Add a related topic
    pub fn add_related_topic(&mut self, name: &str, url: Option<&str>) {
        self.relatedTopics.push(RelatedTopic {
            name: name.to_string(),
            url: url.map(String::from),
        });
    }
}

/// Suggestion for alternative search queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub suggestion: String,
    pub url: Option<String>,
}

/// Correction for misspelled queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub correction: String,
    pub url: Option<String>,
}

/// Answer box for direct answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub answer: String,
    pub url: Option<String>,
}

/// Aggregated infobox results from multiple engines
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfoboxAggregator {
    /// All collected infoboxes
    infoboxes: Vec<Infobox>,

    /// Search suggestions
    suggestions: Vec<Suggestion>,

    /// Spelling corrections
    corrections: Vec<Correction>,

    /// Direct answers
    answers: Vec<Answer>,

    /// Number of results by engine
    engine_results: HashMap<String, usize>,
}

impl InfoboxAggregator {
    /// Create a new infobox aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an infobox
    pub fn add_infobox(&mut self, infobox: Infobox) {
        // Check for duplicates by id
        if !self.infoboxes.iter().any(|ib| ib.id == infobox.id) {
            self.infoboxes.push(infobox);
        }
    }

    /// Add a suggestion
    pub fn add_suggestion(&mut self, suggestion: &str, url: Option<&str>) {
        // Check for duplicates
        if !self.suggestions.iter().any(|s| s.suggestion == suggestion) {
            self.suggestions.push(Suggestion {
                suggestion: suggestion.to_string(),
                url: url.map(String::from),
            });
        }
    }

    /// Add a correction
    pub fn add_correction(&mut self, correction: &str, url: Option<&str>) {
        if !self.corrections.iter().any(|c| c.correction == correction) {
            self.corrections.push(Correction {
                correction: correction.to_string(),
                url: url.map(String::from),
            });
        }
    }

    /// Add an answer
    pub fn add_answer(&mut self, answer: &str, url: Option<&str>) {
        self.answers.push(Answer {
            answer: answer.to_string(),
            url: url.map(String::from),
        });
    }

    /// Record results from an engine
    pub fn record_engine_results(&mut self, engine: &str, count: usize) {
        *self.engine_results.entry(engine.to_string()).or_insert(0) += count;
    }

    /// Get all infoboxes
    pub fn get_infoboxes(&self) -> &[Infobox] {
        &self.infoboxes
    }

    /// Get all suggestions
    pub fn get_suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    /// Get all corrections
    pub fn get_corrections(&self) -> &[Correction] {
        &self.corrections
    }

    /// Get all answers
    pub fn get_answers(&self) -> &[Answer] {
        &self.answers
    }

    /// Get engine result counts
    pub fn get_engine_results(&self) -> &HashMap<String, usize> {
        &self.engine_results
    }

    /// Merge another aggregator into this one
    pub fn merge(&mut self, other: InfoboxAggregator) {
        for infobox in other.infoboxes {
            self.add_infobox(infobox);
        }
        for suggestion in other.suggestions {
            self.add_suggestion(&suggestion.suggestion, suggestion.url.as_deref());
        }
        for correction in other.corrections {
            self.add_correction(&correction.correction, correction.url.as_deref());
        }
        for answer in other.answers {
            self.add_answer(&answer.answer, answer.url.as_deref());
        }
        for (engine, count) in other.engine_results {
            *self.engine_results.entry(engine).or_insert(0) += count;
        }
    }
}

/// Helper to create a Wikipedia-style infobox
pub fn create_wikipedia_infobox(
    title: &str,
    summary: &str,
    url: &str,
    image_url: Option<&str>,
) -> Infobox {
    let mut infobox = Infobox::new(title, "wikipedia");
    infobox.content = Some(summary.to_string());
    infobox.add_url("Wikipedia", url, true);
    if let Some(img) = image_url {
        infobox.img_src = Some(img.to_string());
    }
    infobox
}

/// Helper to create a Wikidata-style infobox
pub fn create_wikidata_infobox(
    title: &str,
    description: Option<&str>,
    attributes: &[(String, String)],
    urls: &[(String, String)],
) -> Infobox {
    let mut infobox = Infobox::new(title, "wikidata");
    if let Some(desc) = description {
        infobox.content = Some(desc.to_string());
    }
    for (label, value) in attributes {
        infobox.add_attribute(label, value, None);
    }
    for (title, url) in urls {
        infobox.add_url(title, url, false);
    }
    infobox
}
