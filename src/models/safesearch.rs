//! This module provides SafeSearch filtering functionality.
//! Similar to SearXNG's safe search implementation.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// SafeSearch levels (0 = off, 1 = moderate, 2 = strict)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SafeSearchLevel {
    /// No filtering
    Off = 0,
    /// Moderate filtering
    Moderate = 1,
    /// Strict filtering
    Strict = 2,
}

impl Default for SafeSearchLevel {
    fn default() -> Self {
        SafeSearchLevel::Moderate
    }
}

impl From<u8> for SafeSearchLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => SafeSearchLevel::Off,
            1 => SafeSearchLevel::Moderate,
            _ => SafeSearchLevel::Strict,
        }
    }
}

impl From<SafeSearchLevel> for u8 {
    fn from(level: SafeSearchLevel) -> u8 {
        level as u8
    }
}

impl SafeSearchLevel {
    /// Parse from string (for API compatibility)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "off" | "0" | "none" | "false" => SafeSearchLevel::Off,
            "moderate" | "1" | "medium" => SafeSearchLevel::Moderate,
            "strict" | "2" | "on" | "true" => SafeSearchLevel::Strict,
            _ => SafeSearchLevel::Moderate,
        }
    }

    /// Convert to display string
    pub fn to_string(&self) -> &'static str {
        match self {
            SafeSearchLevel::Off => "off",
            SafeSearchLevel::Moderate => "moderate",
            SafeSearchLevel::Strict => "strict",
        }
    }

    /// Get DuckDuckGo safe search parameter
    pub fn duckduckgo_param(&self) -> &'static str {
        match self {
            SafeSearchLevel::Off => "-2",
            SafeSearchLevel::Moderate => "-1",
            SafeSearchLevel::Strict => "1",
        }
    }

    /// Get Google safe search parameter
    pub fn google_param(&self) -> &'static str {
        match self {
            SafeSearchLevel::Off => "off",
            SafeSearchLevel::Moderate => "medium",
            SafeSearchLevel::Strict => "active",
        }
    }

    /// Get Bing safe search parameter
    pub fn bing_param(&self) -> &'static str {
        match self {
            SafeSearchLevel::Off => "Off",
            SafeSearchLevel::Moderate => "Moderate",
            SafeSearchLevel::Strict => "Strict",
        }
    }

    /// Get Yahoo safe search parameter (1=off, 2=moderate, strict doesn't filter more)
    pub fn yahoo_param(&self) -> &'static str {
        match self {
            SafeSearchLevel::Off => "r",
            SafeSearchLevel::Moderate => "i",
            SafeSearchLevel::Strict => "p",
        }
    }

    /// Get Qwant safe search parameter
    pub fn qwant_param(&self) -> u8 {
        match self {
            SafeSearchLevel::Off => 0,
            SafeSearchLevel::Moderate => 1,
            SafeSearchLevel::Strict => 2,
        }
    }

    /// Get Brave safe search parameter
    pub fn brave_param(&self) -> &'static str {
        match self {
            SafeSearchLevel::Off => "off",
            SafeSearchLevel::Moderate => "moderate",
            SafeSearchLevel::Strict => "strict",
        }
    }
}

/// List of known adult/NSFW keywords for content filtering
static ADULT_KEYWORDS: &[&str] = &[
    "porn", "xxx", "adult", "nsfw", "sex", "nude", "naked",
    "erotic", "hentai", "18+", "mature",
];

/// Check if content might be adult content
pub fn might_be_adult_content(text: &str) -> bool {
    let lower = text.to_lowercase();
    ADULT_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// Filter results based on safe search level
pub fn filter_results<T, F>(results: Vec<T>, level: SafeSearchLevel, get_content: F) -> Vec<T>
where
    F: Fn(&T) -> &str,
{
    match level {
        SafeSearchLevel::Off => results,
        SafeSearchLevel::Moderate | SafeSearchLevel::Strict => {
            results.into_iter()
                .filter(|r| !might_be_adult_content(get_content(r)))
                .collect()
        }
    }
}

/// SafeSearch configuration for the instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeSearchConfig {
    /// Default safe search level
    pub default_level: SafeSearchLevel,

    /// Minimum allowed safe search level (can't go below this)
    pub minimum_level: SafeSearchLevel,

    /// Whether users can change safe search settings
    pub user_configurable: bool,
}

impl Default for SafeSearchConfig {
    fn default() -> Self {
        Self {
            default_level: SafeSearchLevel::Moderate,
            minimum_level: SafeSearchLevel::Off,
            user_configurable: true,
        }
    }
}

impl SafeSearchConfig {
    /// Get the effective safe search level considering user preference and config
    pub fn effective_level(&self, user_preference: Option<u8>) -> SafeSearchLevel {
        let user_level = user_preference
            .map(SafeSearchLevel::from)
            .unwrap_or(self.default_level);

        // Ensure we don't go below minimum
        if (user_level as u8) < (self.minimum_level as u8) {
            self.minimum_level
        } else {
            user_level
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_search_levels() {
        assert_eq!(SafeSearchLevel::from(0), SafeSearchLevel::Off);
        assert_eq!(SafeSearchLevel::from(1), SafeSearchLevel::Moderate);
        assert_eq!(SafeSearchLevel::from(2), SafeSearchLevel::Strict);
        assert_eq!(SafeSearchLevel::from(99), SafeSearchLevel::Strict);
    }

    #[test]
    fn test_adult_content_detection() {
        assert!(might_be_adult_content("This contains porn"));
        assert!(might_be_adult_content("NSFW content here"));
        assert!(!might_be_adult_content("Normal content"));
        assert!(!might_be_adult_content("Technology news"));
    }

    #[test]
    fn test_filter_results() {
        let results = vec!["normal", "porn site", "news", "xxx content"];
        let filtered = filter_results(results, SafeSearchLevel::Strict, |s| *s);
        assert_eq!(filtered, vec!["normal", "news"]);
    }
}
