//! This module provides engine categorization functionality similar to SearXNG.
//!
//! SearXNG uses a hierarchical category system with main tabs and sub-groups.
//! This module implements a similar system for websurfx.
//!
//! Categories as tabs (from SearXNG settings.yml):
//!   general, images, videos, news, map, music, it, science, files, social media
//!
//! Additional categories:
//!   onions (Tor hidden services), other (miscellaneous)

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Main category tabs (like SearXNG's `categories_as_tabs`)
/// Matches SearXNG's category system exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineCategory {
    /// General web search engines (default)
    General,
    /// Image search engines
    Images,
    /// Video search engines
    Videos,
    /// News search engines
    News,
    /// Map and location search engines
    Map,
    /// Music and audio search engines
    Music,
    /// IT, programming, and tech search engines
    It,
    /// Science and academic search engines
    Science,
    /// File and torrent search engines
    Files,
    /// Social media search engines
    SocialMedia,
    /// Shopping and e-commerce search engines
    Shopping,
    /// Reference and encyclopedias
    Reference,
    /// Onion/Tor hidden services (like SearXNG)
    Onions,
    /// Other/miscellaneous
    Other,
}

impl EngineCategory {
    /// Get all available categories
    pub fn all() -> &'static [EngineCategory] {
        &[
            EngineCategory::General,
            EngineCategory::Images,
            EngineCategory::Videos,
            EngineCategory::News,
            EngineCategory::Map,
            EngineCategory::Music,
            EngineCategory::It,
            EngineCategory::Science,
            EngineCategory::Files,
            EngineCategory::SocialMedia,
            EngineCategory::Shopping,
            EngineCategory::Reference,
            EngineCategory::Onions,
            EngineCategory::Other,
        ]
    }

    /// Get the display name of the category
    pub fn display_name(&self) -> &'static str {
        match self {
            EngineCategory::General => "General",
            EngineCategory::Images => "Images",
            EngineCategory::Videos => "Videos",
            EngineCategory::News => "News",
            EngineCategory::Map => "Map",
            EngineCategory::Music => "Music",
            EngineCategory::It => "IT",
            EngineCategory::Science => "Science",
            EngineCategory::Files => "Files",
            EngineCategory::SocialMedia => "Social Media",
            EngineCategory::Shopping => "Shopping",
            EngineCategory::Reference => "Reference",
            EngineCategory::Onions => "Onions",
            EngineCategory::Other => "Other",
        }
    }

    /// Get the bang syntax for the category (like SearXNG's !general, !images, etc.)
    pub fn bang(&self) -> &'static str {
        match self {
            EngineCategory::General => "!general",
            EngineCategory::Images => "!images",
            EngineCategory::Videos => "!videos",
            EngineCategory::News => "!news",
            EngineCategory::Map => "!map",
            EngineCategory::Music => "!music",
            EngineCategory::It => "!it",
            EngineCategory::Science => "!science",
            EngineCategory::Files => "!files",
            EngineCategory::SocialMedia => "!social",
            EngineCategory::Shopping => "!shop",
            EngineCategory::Reference => "!ref",
            EngineCategory::Onions => "!onions",
            EngineCategory::Other => "!other",
        }
    }

    /// Parse a category from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "general" | "web" => Some(EngineCategory::General),
            "images" | "image" | "img" => Some(EngineCategory::Images),
            "videos" | "video" | "vid" => Some(EngineCategory::Videos),
            "news" => Some(EngineCategory::News),
            "map" | "maps" | "location" => Some(EngineCategory::Map),
            "music" | "audio" => Some(EngineCategory::Music),
            "it" | "tech" | "programming" | "code" => Some(EngineCategory::It),
            "science" | "academic" | "scholar" => Some(EngineCategory::Science),
            "files" | "file" | "torrents" | "torrent" => Some(EngineCategory::Files),
            "social" | "social_media" | "socialmedia" => Some(EngineCategory::SocialMedia),
            "shopping" | "shop" | "ecommerce" => Some(EngineCategory::Shopping),
            "reference" | "ref" | "wiki" | "encyclopedia" => Some(EngineCategory::Reference),
            "onions" | "onion" | "tor" => Some(EngineCategory::Onions),
            "other" | "misc" => Some(EngineCategory::Other),
            _ => None,
        }
    }
}

/// Sub-categories within main categories (like SearXNG's groups)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineSubcategory {
    // General subcategories
    Web,
    Books,
    Currency,
    Translate,

    // IT subcategories
    Packages,
    QAndA,
    Repos,
    SoftwareWikis,

    // Science subcategories
    ScientificPublications,

    // Files subcategories
    Apps,
    Torrents,

    // Images subcategories
    Icons,
    Photos,

    // Music subcategories
    Lyrics,
    Radio,
    Streaming,

    // Reference subcategories
    Wikimedia,
    Encyclopedia,
    Dictionary,

    // Videos subcategories
    Movies,
    Anime,

    // Social subcategories
    Fediverse,
    Forums,

    // No specific subcategory
    None,
}

impl EngineSubcategory {
    /// Get the display name of the subcategory
    pub fn display_name(&self) -> &'static str {
        match self {
            EngineSubcategory::Web => "Web",
            EngineSubcategory::Books => "Books",
            EngineSubcategory::Currency => "Currency",
            EngineSubcategory::Translate => "Translate",
            EngineSubcategory::Packages => "Packages",
            EngineSubcategory::QAndA => "Q&A",
            EngineSubcategory::Repos => "Repositories",
            EngineSubcategory::SoftwareWikis => "Software Wikis",
            EngineSubcategory::ScientificPublications => "Scientific Publications",
            EngineSubcategory::Apps => "Apps",
            EngineSubcategory::Torrents => "Torrents",
            EngineSubcategory::Icons => "Icons",
            EngineSubcategory::Photos => "Photos",
            EngineSubcategory::Lyrics => "Lyrics",
            EngineSubcategory::Radio => "Radio",
            EngineSubcategory::Streaming => "Streaming",
            EngineSubcategory::Wikimedia => "Wikimedia",
            EngineSubcategory::Encyclopedia => "Encyclopedia",
            EngineSubcategory::Dictionary => "Dictionary",
            EngineSubcategory::Movies => "Movies",
            EngineSubcategory::Anime => "Anime",
            EngineSubcategory::Fediverse => "Fediverse",
            EngineSubcategory::Forums => "Forums",
            EngineSubcategory::None => "",
        }
    }
}

/// Engine metadata containing category information (only serializable, not deserializable due to static references)
#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo {
    /// Engine name
    pub name: &'static str,
    /// Display name for UI
    pub display_name: &'static str,
    /// Main categories this engine belongs to (can be multiple)
    #[serde(serialize_with = "serialize_categories")]
    pub categories: &'static [EngineCategory],
    /// Subcategory within the main category
    pub subcategory: EngineSubcategory,
    /// Short bang shortcut (like !g for Google, !ddg for DuckDuckGo)
    pub shortcut: &'static str,
    /// Whether the engine is enabled by default
    pub default_enabled: bool,
    /// Language/Region this engine is optimized for (empty for global)
    pub region: &'static str,
    /// Description of the engine
    pub description: &'static str,
}

/// Custom serializer for static category slice
fn serialize_categories<S>(categories: &&'static [EngineCategory], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(categories.len()))?;
    for cat in *categories {
        seq.serialize_element(cat)?;
    }
    seq.end()
}

/// Static engine registry
static ENGINE_REGISTRY: OnceLock<HashMap<&'static str, EngineInfo>> = OnceLock::new();

/// Get the engine registry (initializes on first access)
pub fn get_engine_registry() -> &'static HashMap<&'static str, EngineInfo> {
    ENGINE_REGISTRY.get_or_init(|| {
        let mut map = HashMap::new();
        init_engine_registry(&mut map);
        map
    })
}

/// Initialize the engine registry with all engines
fn init_engine_registry(map: &mut HashMap<&'static str, EngineInfo>) {

    // ===== General Web Search Engines =====
    map.insert("google", EngineInfo {
        name: "google",
        display_name: "Google",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!g",
        default_enabled: true,
        region: "",
        description: "Google Search - the world's most popular search engine",
    });

    map.insert("bing", EngineInfo {
        name: "bing",
        display_name: "Bing",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!bi",
        default_enabled: true,
        region: "",
        description: "Microsoft Bing Search",
    });

    map.insert("duckduckgo", EngineInfo {
        name: "duckduckgo",
        display_name: "DuckDuckGo",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!ddg",
        default_enabled: true,
        region: "",
        description: "Privacy-focused search engine",
    });

    map.insert("brave", EngineInfo {
        name: "brave",
        display_name: "Brave Search",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!br",
        default_enabled: true,
        region: "",
        description: "Independent search engine by Brave",
    });

    map.insert("startpage", EngineInfo {
        name: "startpage",
        display_name: "Startpage",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!sp",
        default_enabled: true,
        region: "",
        description: "Privacy-focused search using Google results",
    });

    map.insert("qwant", EngineInfo {
        name: "qwant",
        display_name: "Qwant",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!qw",
        default_enabled: true,
        region: "EU",
        description: "European privacy-focused search engine",
    });

    map.insert("mojeek", EngineInfo {
        name: "mojeek",
        display_name: "Mojeek",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!mjk",
        default_enabled: true,
        region: "UK",
        description: "Independent search engine with its own index",
    });

    map.insert("yahoo", EngineInfo {
        name: "yahoo",
        display_name: "Yahoo",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!yh",
        default_enabled: false,
        region: "",
        description: "Yahoo Search",
    });

    map.insert("yandex", EngineInfo {
        name: "yandex",
        display_name: "Yandex",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!yd",
        default_enabled: false,
        region: "RU",
        description: "Russian search engine",
    });

    map.insert("ask", EngineInfo {
        name: "ask",
        display_name: "Ask.com",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!ask",
        default_enabled: false,
        region: "",
        description: "Ask.com search engine",
    });

    map.insert("searx", EngineInfo {
        name: "searx",
        display_name: "SearX",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!sx",
        default_enabled: false,
        region: "",
        description: "Privacy-respecting metasearch engine",
    });

    map.insert("librex", EngineInfo {
        name: "librex",
        display_name: "LibreX",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!lx",
        default_enabled: false,
        region: "",
        description: "Privacy-respecting metasearch engine",
    });

    // ===== Chinese Search Engines =====
    map.insert("baidu", EngineInfo {
        name: "baidu",
        display_name: "百度",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!bd",
        default_enabled: false,
        region: "CN",
        description: "China's largest search engine",
    });

    map.insert("sogou", EngineInfo {
        name: "sogou",
        display_name: "搜狗",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!sogou",
        default_enabled: false,
        region: "CN",
        description: "Sogou Search (Tencent)",
    });

    map.insert("360search", EngineInfo {
        name: "360search",
        display_name: "360搜索",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!360",
        default_enabled: false,
        region: "CN",
        description: "360 Search Engine (China)",
    });

    map.insert("chinaso", EngineInfo {
        name: "chinaso",
        display_name: "中国搜索",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!chinaso",
        default_enabled: false,
        region: "CN",
        description: "China Search - State-run search engine",
    });

    // ===== Korean Search Engines =====
    map.insert("naver", EngineInfo {
        name: "naver",
        display_name: "네이버",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!nvr",
        default_enabled: false,
        region: "KR",
        description: "South Korea's largest search portal",
    });

    // ===== Video Search Engines =====
    map.insert("youtube", EngineInfo {
        name: "youtube",
        display_name: "YouTube",
        categories: &[EngineCategory::Videos, EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!yt",
        default_enabled: true,
        region: "",
        description: "World's largest video platform",
    });

    map.insert("vimeo", EngineInfo {
        name: "vimeo",
        display_name: "Vimeo",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!vm",
        default_enabled: false,
        region: "",
        description: "High-quality video hosting platform",
    });

    map.insert("dailymotion", EngineInfo {
        name: "dailymotion",
        display_name: "Dailymotion",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!dm",
        default_enabled: false,
        region: "",
        description: "Video-sharing platform",
    });

    map.insert("bilibili", EngineInfo {
        name: "bilibili",
        display_name: "哔哩哔哩",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Anime,
        shortcut: "!bil",
        default_enabled: false,
        region: "CN",
        description: "Chinese video sharing website focused on animation, comics, and games",
    });

    // ===== Image Search Engines =====
    map.insert("unsplash", EngineInfo {
        name: "unsplash",
        display_name: "Unsplash",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!us",
        default_enabled: true,
        region: "",
        description: "Free high-resolution photos",
    });

    map.insert("flickr", EngineInfo {
        name: "flickr",
        display_name: "Flickr",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!fl",
        default_enabled: false,
        region: "",
        description: "Image and video hosting service",
    });

    map.insert("pixabay", EngineInfo {
        name: "pixabay",
        display_name: "Pixabay",
        categories: &[EngineCategory::Images, EngineCategory::Videos],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!pxb",
        default_enabled: false,
        region: "",
        description: "Free stock photos, illustrations, and videos",
    });

    map.insert("deviantart", EngineInfo {
        name: "deviantart",
        display_name: "DeviantArt",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!da",
        default_enabled: false,
        region: "",
        description: "Online art community",
    });

    // ===== Music Search Engines =====
    map.insert("soundcloud", EngineInfo {
        name: "soundcloud",
        display_name: "SoundCloud",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!sc",
        default_enabled: true,
        region: "",
        description: "Audio distribution platform",
    });

    map.insert("bandcamp", EngineInfo {
        name: "bandcamp",
        display_name: "Bandcamp",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!bc",
        default_enabled: false,
        region: "",
        description: "Music community platform for artists",
    });

    map.insert("genius", EngineInfo {
        name: "genius",
        display_name: "Genius",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Lyrics,
        shortcut: "!gen",
        default_enabled: false,
        region: "",
        description: "Song lyrics and music knowledge",
    });

    map.insert("spotify", EngineInfo {
        name: "spotify",
        display_name: "Spotify",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!spot",
        default_enabled: false,
        region: "",
        description: "Digital music streaming service",
    });

    map.insert("applemusic", EngineInfo {
        name: "applemusic",
        display_name: "Apple Music",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!am",
        default_enabled: false,
        region: "",
        description: "Apple's music streaming service",
    });

    // ===== IT/Programming Search Engines =====
    map.insert("github", EngineInfo {
        name: "github",
        display_name: "GitHub",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!gh",
        default_enabled: true,
        region: "",
        description: "Code hosting platform for version control and collaboration",
    });

    map.insert("gitlab", EngineInfo {
        name: "gitlab",
        display_name: "GitLab",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!gl",
        default_enabled: false,
        region: "",
        description: "DevOps platform with Git repository management",
    });

    map.insert("codeberg", EngineInfo {
        name: "codeberg",
        display_name: "Codeberg",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!cb",
        default_enabled: false,
        region: "",
        description: "Non-profit, community-run code hosting",
    });

    map.insert("bitbucket", EngineInfo {
        name: "bitbucket",
        display_name: "Bitbucket",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!bb",
        default_enabled: false,
        region: "",
        description: "Atlassian's Git-based code hosting",
    });

    map.insert("stackexchange", EngineInfo {
        name: "stackexchange",
        display_name: "Stack Overflow",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::QAndA,
        shortcut: "!so",
        default_enabled: true,
        region: "",
        description: "Programming Q&A community",
    });

    map.insert("npm", EngineInfo {
        name: "npm",
        display_name: "npm",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!npm",
        default_enabled: true,
        region: "",
        description: "Node.js package registry",
    });

    map.insert("pypi", EngineInfo {
        name: "pypi",
        display_name: "PyPI",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!pypi",
        default_enabled: true,
        region: "",
        description: "Python Package Index",
    });

    map.insert("crates", EngineInfo {
        name: "crates",
        display_name: "crates.io",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!crates",
        default_enabled: true,
        region: "",
        description: "Rust package registry",
    });

    map.insert("dockerhub", EngineInfo {
        name: "dockerhub",
        display_name: "Docker Hub",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!dh",
        default_enabled: true,
        region: "",
        description: "Container image registry",
    });

    map.insert("huggingface", EngineInfo {
        name: "huggingface",
        display_name: "Hugging Face",
        categories: &[EngineCategory::It, EngineCategory::Science],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!hf",
        default_enabled: true,
        region: "",
        description: "AI model and dataset hub",
    });

    map.insert("hackernews", EngineInfo {
        name: "hackernews",
        display_name: "Hacker News",
        categories: &[EngineCategory::It, EngineCategory::News],
        subcategory: EngineSubcategory::Forums,
        shortcut: "!hn",
        default_enabled: true,
        region: "",
        description: "Tech news and discussions",
    });

    map.insert("lobsters", EngineInfo {
        name: "lobsters",
        display_name: "Lobsters",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Forums,
        shortcut: "!lo",
        default_enabled: false,
        region: "",
        description: "Computing-focused link aggregation",
    });

    // ===== Science/Academic Search Engines =====
    map.insert("arxiv", EngineInfo {
        name: "arxiv",
        display_name: "arXiv",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!arx",
        default_enabled: true,
        region: "",
        description: "Open-access preprint repository",
    });

    map.insert("pubmed", EngineInfo {
        name: "pubmed",
        display_name: "PubMed",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!pub",
        default_enabled: true,
        region: "",
        description: "Biomedical literature database",
    });

    map.insert("semanticscholar", EngineInfo {
        name: "semanticscholar",
        display_name: "Semantic Scholar",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!ss",
        default_enabled: true,
        region: "",
        description: "AI-powered research tool",
    });

    map.insert("crossref", EngineInfo {
        name: "crossref",
        display_name: "Crossref",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!cr",
        default_enabled: false,
        region: "",
        description: "Scholarly metadata registry",
    });

    map.insert("core", EngineInfo {
        name: "core",
        display_name: "CORE",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!core",
        default_enabled: false,
        region: "",
        description: "Open access research papers",
    });

    map.insert("googlescholar", EngineInfo {
        name: "googlescholar",
        display_name: "Google Scholar",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!gos",
        default_enabled: true,
        region: "",
        description: "Google's academic search",
    });

    map.insert("wolframalpha", EngineInfo {
        name: "wolframalpha",
        display_name: "Wolfram Alpha",
        categories: &[EngineCategory::Science, EngineCategory::Reference],
        subcategory: EngineSubcategory::None,
        shortcut: "!wa",
        default_enabled: false,
        region: "",
        description: "Computational knowledge engine",
    });

    // ===== Reference/Encyclopedia Search Engines =====
    map.insert("wikipedia", EngineInfo {
        name: "wikipedia",
        display_name: "Wikipedia",
        categories: &[EngineCategory::Reference, EngineCategory::General],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wp",
        default_enabled: true,
        region: "",
        description: "Free online encyclopedia",
    });

    map.insert("wikidata", EngineInfo {
        name: "wikidata",
        display_name: "Wikidata",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wd",
        default_enabled: false,
        region: "",
        description: "Free knowledge base",
    });

    map.insert("wikibooks", EngineInfo {
        name: "wikibooks",
        display_name: "Wikibooks",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wb",
        default_enabled: false,
        region: "",
        description: "Free textbooks",
    });

    map.insert("wikiquote", EngineInfo {
        name: "wikiquote",
        display_name: "Wikiquote",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wq",
        default_enabled: false,
        region: "",
        description: "Free quote compendium",
    });

    map.insert("wikinews", EngineInfo {
        name: "wikinews",
        display_name: "Wikinews",
        categories: &[EngineCategory::Reference, EngineCategory::News],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wn",
        default_enabled: false,
        region: "",
        description: "Free-content news source",
    });

    map.insert("wikisource", EngineInfo {
        name: "wikisource",
        display_name: "Wikisource",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!ws",
        default_enabled: false,
        region: "",
        description: "Free library",
    });

    map.insert("wikiversity", EngineInfo {
        name: "wikiversity",
        display_name: "Wikiversity",
        categories: &[EngineCategory::Reference, EngineCategory::Science],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wv",
        default_enabled: false,
        region: "",
        description: "Free learning resources",
    });

    map.insert("wikivoyage", EngineInfo {
        name: "wikivoyage",
        display_name: "Wikivoyage",
        categories: &[EngineCategory::Reference, EngineCategory::Map],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wy",
        default_enabled: false,
        region: "",
        description: "Free travel guide",
    });

    map.insert("wiktionary", EngineInfo {
        name: "wiktionary",
        display_name: "Wiktionary",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Dictionary,
        shortcut: "!wt",
        default_enabled: false,
        region: "",
        description: "Free dictionary",
    });

    map.insert("wikispecies", EngineInfo {
        name: "wikispecies",
        display_name: "Wikispecies",
        categories: &[EngineCategory::Reference, EngineCategory::Science],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wsp",
        default_enabled: false,
        region: "",
        description: "Free species directory",
    });

    map.insert("wikicommons", EngineInfo {
        name: "wikicommons",
        display_name: "Wikimedia Commons",
        categories: &[EngineCategory::Images, EngineCategory::Reference],
        subcategory: EngineSubcategory::Wikimedia,
        shortcut: "!wc",
        default_enabled: false,
        region: "",
        description: "Free media repository",
    });

    map.insert("openlibrary", EngineInfo {
        name: "openlibrary",
        display_name: "Open Library",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Books,
        shortcut: "!ol",
        default_enabled: false,
        region: "",
        description: "Open, editable library catalog",
    });

    map.insert("internetarchive", EngineInfo {
        name: "internetarchive",
        display_name: "Internet Archive",
        categories: &[EngineCategory::Reference, EngineCategory::Files],
        subcategory: EngineSubcategory::None,
        shortcut: "!ia",
        default_enabled: false,
        region: "",
        description: "Digital library of Internet sites",
    });

    // ===== Map/Location Search Engines =====
    map.insert("openstreetmap", EngineInfo {
        name: "openstreetmap",
        display_name: "OpenStreetMap",
        categories: &[EngineCategory::Map],
        subcategory: EngineSubcategory::None,
        shortcut: "!osm",
        default_enabled: true,
        region: "",
        description: "Free, editable map of the world",
    });

    // ===== Social Media Search Engines =====
    map.insert("reddit", EngineInfo {
        name: "reddit",
        display_name: "Reddit",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::Forums,
        shortcut: "!re",
        default_enabled: true,
        region: "",
        description: "The front page of the internet",
    });

    map.insert("twitter", EngineInfo {
        name: "twitter",
        display_name: "X (Twitter)",
        categories: &[EngineCategory::SocialMedia, EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!tw",
        default_enabled: false,
        region: "",
        description: "Social networking and microblogging",
    });

    map.insert("mastodon", EngineInfo {
        name: "mastodon",
        display_name: "Mastodon",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::Fediverse,
        shortcut: "!mast",
        default_enabled: false,
        region: "",
        description: "Decentralized social network",
    });

    map.insert("lemmy", EngineInfo {
        name: "lemmy",
        display_name: "Lemmy",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::Fediverse,
        shortcut: "!lem",
        default_enabled: false,
        region: "",
        description: "Federated link aggregator",
    });

    // Chinese social
    map.insert("zhihu", EngineInfo {
        name: "zhihu",
        display_name: "知乎",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::QAndA,
        shortcut: "!zh",
        default_enabled: false,
        region: "CN",
        description: "Chinese Q&A platform",
    });

    map.insert("weibo", EngineInfo {
        name: "weibo",
        display_name: "微博",
        categories: &[EngineCategory::SocialMedia, EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!weibo",
        default_enabled: false,
        region: "CN",
        description: "Chinese microblogging platform",
    });

    map.insert("douban", EngineInfo {
        name: "douban",
        display_name: "豆瓣",
        categories: &[EngineCategory::SocialMedia, EngineCategory::Reference],
        subcategory: EngineSubcategory::Movies,
        shortcut: "!db",
        default_enabled: false,
        region: "CN",
        description: "Chinese social network for movies, books, and music",
    });

    // ===== Shopping Search Engines =====
    map.insert("ebay", EngineInfo {
        name: "ebay",
        display_name: "eBay",
        categories: &[EngineCategory::Shopping],
        subcategory: EngineSubcategory::None,
        shortcut: "!ebay",
        default_enabled: false,
        region: "",
        description: "Online marketplace",
    });

    map.insert("amazon", EngineInfo {
        name: "amazon",
        display_name: "Amazon",
        categories: &[EngineCategory::Shopping],
        subcategory: EngineSubcategory::None,
        shortcut: "!amz",
        default_enabled: false,
        region: "",
        description: "Online retailer",
    });

    map.insert("taobao", EngineInfo {
        name: "taobao",
        display_name: "淘宝",
        categories: &[EngineCategory::Shopping],
        subcategory: EngineSubcategory::None,
        shortcut: "!tb",
        default_enabled: false,
        region: "CN",
        description: "Chinese online shopping",
    });

    map.insert("jd", EngineInfo {
        name: "jd",
        display_name: "京东",
        categories: &[EngineCategory::Shopping],
        subcategory: EngineSubcategory::None,
        shortcut: "!jd",
        default_enabled: false,
        region: "CN",
        description: "Chinese e-commerce company",
    });

    // ===== Movies/Entertainment =====
    map.insert("imdb", EngineInfo {
        name: "imdb",
        display_name: "IMDb",
        categories: &[EngineCategory::Videos, EngineCategory::Reference],
        subcategory: EngineSubcategory::Movies,
        shortcut: "!imdb",
        default_enabled: false,
        region: "",
        description: "Internet Movie Database",
    });

    map.insert("rottentomatoes", EngineInfo {
        name: "rottentomatoes",
        display_name: "Rotten Tomatoes",
        categories: &[EngineCategory::Videos, EngineCategory::Reference],
        subcategory: EngineSubcategory::Movies,
        shortcut: "!rt",
        default_enabled: false,
        region: "",
        description: "Movie and TV review aggregator",
    });

    // ===== Batch 1: Image/Video Variants =====
    map.insert("bing_images", EngineInfo {
        name: "bing_images",
        display_name: "Bing Images",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!bii",
        default_enabled: true,
        region: "",
        description: "Bing image search",
    });

    map.insert("bing_videos", EngineInfo {
        name: "bing_videos",
        display_name: "Bing Videos",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!biv",
        default_enabled: false,
        region: "",
        description: "Bing video search",
    });

    map.insert("bing_news", EngineInfo {
        name: "bing_news",
        display_name: "Bing News",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!bin",
        default_enabled: false,
        region: "",
        description: "Bing news search",
    });

    map.insert("google_images", EngineInfo {
        name: "google_images",
        display_name: "Google Images",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!goi",
        default_enabled: true,
        region: "",
        description: "Google image search",
    });

    map.insert("google_videos", EngineInfo {
        name: "google_videos",
        display_name: "Google Videos",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!gov",
        default_enabled: false,
        region: "",
        description: "Google video search",
    });

    map.insert("google_news", EngineInfo {
        name: "google_news",
        display_name: "Google News",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!gon",
        default_enabled: true,
        region: "",
        description: "Google news search",
    });

    map.insert("google_play", EngineInfo {
        name: "google_play",
        display_name: "Google Play",
        categories: &[EngineCategory::Files, EngineCategory::It],
        subcategory: EngineSubcategory::Apps,
        shortcut: "!gp",
        default_enabled: false,
        region: "",
        description: "Google Play Store apps",
    });

    map.insert("duckduckgo_images", EngineInfo {
        name: "duckduckgo_images",
        display_name: "DuckDuckGo Images",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!ddgi",
        default_enabled: true,
        region: "",
        description: "DuckDuckGo image search",
    });

    map.insert("duckduckgo_videos", EngineInfo {
        name: "duckduckgo_videos",
        display_name: "DuckDuckGo Videos",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!ddgv",
        default_enabled: false,
        region: "",
        description: "DuckDuckGo video search",
    });

    map.insert("duckduckgo_news", EngineInfo {
        name: "duckduckgo_news",
        display_name: "DuckDuckGo News",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!ddgn",
        default_enabled: false,
        region: "",
        description: "DuckDuckGo news search",
    });

    map.insert("brave_images", EngineInfo {
        name: "brave_images",
        display_name: "Brave Images",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!bri",
        default_enabled: false,
        region: "",
        description: "Brave image search",
    });

    map.insert("brave_videos", EngineInfo {
        name: "brave_videos",
        display_name: "Brave Videos",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!brv",
        default_enabled: false,
        region: "",
        description: "Brave video search",
    });

    map.insert("brave_news", EngineInfo {
        name: "brave_news",
        display_name: "Brave News",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!brn",
        default_enabled: false,
        region: "",
        description: "Brave news search",
    });

    map.insert("qwant_images", EngineInfo {
        name: "qwant_images",
        display_name: "Qwant Images",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!qwi",
        default_enabled: false,
        region: "EU",
        description: "Qwant image search",
    });

    map.insert("qwant_videos", EngineInfo {
        name: "qwant_videos",
        display_name: "Qwant Videos",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!qwv",
        default_enabled: false,
        region: "EU",
        description: "Qwant video search",
    });

    map.insert("qwant_news", EngineInfo {
        name: "qwant_news",
        display_name: "Qwant News",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!qwn",
        default_enabled: false,
        region: "EU",
        description: "Qwant news search",
    });

    map.insert("yandex_images", EngineInfo {
        name: "yandex_images",
        display_name: "Yandex Images",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!ydi",
        default_enabled: false,
        region: "RU",
        description: "Yandex image search",
    });

    map.insert("yandex_videos", EngineInfo {
        name: "yandex_videos",
        display_name: "Yandex Videos",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!ydv",
        default_enabled: false,
        region: "RU",
        description: "Yandex video search",
    });

    map.insert("yandex_music", EngineInfo {
        name: "yandex_music",
        display_name: "Yandex Music",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!ydm",
        default_enabled: false,
        region: "RU",
        description: "Yandex music streaming",
    });

    map.insert("yahoo_news", EngineInfo {
        name: "yahoo_news",
        display_name: "Yahoo News",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!yhn",
        default_enabled: false,
        region: "",
        description: "Yahoo news search",
    });

    // ===== Batch 2: Onions/Tor Services =====
    map.insert("ahmia", EngineInfo {
        name: "ahmia",
        display_name: "Ahmia",
        categories: &[EngineCategory::Onions, EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!ahm",
        default_enabled: false,
        region: "",
        description: "Search engine for Tor hidden services",
    });

    map.insert("torch", EngineInfo {
        name: "torch",
        display_name: "Torch",
        categories: &[EngineCategory::Onions],
        subcategory: EngineSubcategory::Web,
        shortcut: "!torch",
        default_enabled: false,
        region: "",
        description: "Tor search engine",
    });

    map.insert("not_evil", EngineInfo {
        name: "not_evil",
        display_name: "Not Evil",
        categories: &[EngineCategory::Onions],
        subcategory: EngineSubcategory::Web,
        shortcut: "!ne",
        default_enabled: false,
        region: "",
        description: "Tor hidden service search",
    });

    // ===== Batch 2: Files/Torrents =====
    map.insert("piratebay", EngineInfo {
        name: "piratebay",
        display_name: "The Pirate Bay",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!tpb",
        default_enabled: false,
        region: "",
        description: "BitTorrent index",
    });

    map.insert("nyaa", EngineInfo {
        name: "nyaa",
        display_name: "Nyaa",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!nyaa",
        default_enabled: false,
        region: "",
        description: "Anime torrent tracker",
    });

    map.insert("kickass", EngineInfo {
        name: "kickass",
        display_name: "KickassTorrents",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!kat",
        default_enabled: false,
        region: "",
        description: "Torrent search engine",
    });

    map.insert("btdigg", EngineInfo {
        name: "btdigg",
        display_name: "BTDigg",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!btd",
        default_enabled: false,
        region: "",
        description: "BitTorrent DHT search",
    });

    map.insert("bt4g", EngineInfo {
        name: "bt4g",
        display_name: "BT4G",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!bt4g",
        default_enabled: false,
        region: "",
        description: "BitTorrent search engine",
    });

    map.insert("torznab", EngineInfo {
        name: "torznab",
        display_name: "Torznab",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!tz",
        default_enabled: false,
        region: "",
        description: "Torznab API indexer",
    });

    map.insert("tokyotoshokan", EngineInfo {
        name: "tokyotoshokan",
        display_name: "Tokyo Toshokan",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Torrents,
        shortcut: "!tt",
        default_enabled: false,
        region: "JP",
        description: "Japanese BitTorrent tracker",
    });

    map.insert("fdroid", EngineInfo {
        name: "fdroid",
        display_name: "F-Droid",
        categories: &[EngineCategory::Files, EngineCategory::It],
        subcategory: EngineSubcategory::Apps,
        shortcut: "!fd",
        default_enabled: false,
        region: "",
        description: "Free and open source Android apps",
    });

    map.insert("apkmirror", EngineInfo {
        name: "apkmirror",
        display_name: "APKMirror",
        categories: &[EngineCategory::Files],
        subcategory: EngineSubcategory::Apps,
        shortcut: "!apk",
        default_enabled: false,
        region: "",
        description: "Android APK downloads",
    });

    map.insert("apple_app_store", EngineInfo {
        name: "apple_app_store",
        display_name: "App Store",
        categories: &[EngineCategory::Files, EngineCategory::It],
        subcategory: EngineSubcategory::Apps,
        shortcut: "!as",
        default_enabled: false,
        region: "",
        description: "Apple App Store",
    });

    // ===== Batch 2: Alternative Video Platforms =====
    map.insert("peertube", EngineInfo {
        name: "peertube",
        display_name: "PeerTube",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!pt",
        default_enabled: false,
        region: "",
        description: "Decentralized video platform",
    });

    map.insert("sepiasearch", EngineInfo {
        name: "sepiasearch",
        display_name: "Sepia Search",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!sep",
        default_enabled: false,
        region: "",
        description: "PeerTube search engine",
    });

    map.insert("invidious", EngineInfo {
        name: "invidious",
        display_name: "Invidious",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!inv",
        default_enabled: false,
        region: "",
        description: "YouTube alternative frontend",
    });

    map.insert("piped", EngineInfo {
        name: "piped",
        display_name: "Piped",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!pip",
        default_enabled: false,
        region: "",
        description: "Privacy-friendly YouTube frontend",
    });

    map.insert("odysee", EngineInfo {
        name: "odysee",
        display_name: "Odysee",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!ody",
        default_enabled: false,
        region: "",
        description: "Blockchain-based video platform",
    });

    map.insert("rumble", EngineInfo {
        name: "rumble",
        display_name: "Rumble",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!rum",
        default_enabled: false,
        region: "",
        description: "Video sharing platform",
    });

    map.insert("bitchute", EngineInfo {
        name: "bitchute",
        display_name: "BitChute",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!btc",
        default_enabled: false,
        region: "",
        description: "Video hosting service",
    });

    // ===== Batch 3: More IT/Developer Tools =====
    map.insert("sourcehut", EngineInfo {
        name: "sourcehut",
        display_name: "SourceHut",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!srht",
        default_enabled: false,
        region: "",
        description: "Minimalist code hosting",
    });

    map.insert("gitea", EngineInfo {
        name: "gitea",
        display_name: "Gitea",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!gitea",
        default_enabled: false,
        region: "",
        description: "Self-hosted Git service",
    });

    map.insert("lib_rs", EngineInfo {
        name: "lib_rs",
        display_name: "lib.rs",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!librs",
        default_enabled: false,
        region: "",
        description: "Rust crate search",
    });

    map.insert("hex", EngineInfo {
        name: "hex",
        display_name: "Hex.pm",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!hex",
        default_enabled: false,
        region: "",
        description: "Elixir/Erlang package manager",
    });

    map.insert("metacpan", EngineInfo {
        name: "metacpan",
        display_name: "MetaCPAN",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!cpan",
        default_enabled: false,
        region: "",
        description: "Perl module search",
    });

    map.insert("pkg_go_dev", EngineInfo {
        name: "pkg_go_dev",
        display_name: "pkg.go.dev",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!go",
        default_enabled: false,
        region: "",
        description: "Go package documentation",
    });

    map.insert("packagist", EngineInfo {
        name: "packagist",
        display_name: "Packagist",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!php",
        default_enabled: false,
        region: "",
        description: "PHP package repository",
    });

    map.insert("rubygems", EngineInfo {
        name: "rubygems",
        display_name: "RubyGems",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!gem",
        default_enabled: false,
        region: "",
        description: "Ruby package manager",
    });

    map.insert("maven", EngineInfo {
        name: "maven",
        display_name: "Maven Central",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!mvn",
        default_enabled: false,
        region: "",
        description: "Java package repository",
    });

    map.insert("nuget", EngineInfo {
        name: "nuget",
        display_name: "NuGet",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!nuget",
        default_enabled: false,
        region: "",
        description: ".NET package manager",
    });

    map.insert("searchcode", EngineInfo {
        name: "searchcode",
        display_name: "Searchcode",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Repos,
        shortcut: "!sc",
        default_enabled: false,
        region: "",
        description: "Source code search engine",
    });

    map.insert("mdn", EngineInfo {
        name: "mdn",
        display_name: "MDN Web Docs",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::SoftwareWikis,
        shortcut: "!mdn",
        default_enabled: true,
        region: "",
        description: "Mozilla developer documentation",
    });

    map.insert("microsoft_learn", EngineInfo {
        name: "microsoft_learn",
        display_name: "Microsoft Learn",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::SoftwareWikis,
        shortcut: "!msft",
        default_enabled: false,
        region: "",
        description: "Microsoft documentation",
    });

    map.insert("archlinux", EngineInfo {
        name: "archlinux",
        display_name: "Arch Wiki",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::SoftwareWikis,
        shortcut: "!arch",
        default_enabled: false,
        region: "",
        description: "Arch Linux documentation",
    });

    map.insert("gentoo", EngineInfo {
        name: "gentoo",
        display_name: "Gentoo Wiki",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::SoftwareWikis,
        shortcut: "!gentoo",
        default_enabled: false,
        region: "",
        description: "Gentoo Linux documentation",
    });

    map.insert("alpinelinux", EngineInfo {
        name: "alpinelinux",
        display_name: "Alpine Linux",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!alpine",
        default_enabled: false,
        region: "",
        description: "Alpine Linux packages",
    });

    map.insert("voidlinux", EngineInfo {
        name: "voidlinux",
        display_name: "Void Linux",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!void",
        default_enabled: false,
        region: "",
        description: "Void Linux packages",
    });

    map.insert("repology", EngineInfo {
        name: "repology",
        display_name: "Repology",
        categories: &[EngineCategory::It],
        subcategory: EngineSubcategory::Packages,
        shortcut: "!rep",
        default_enabled: false,
        region: "",
        description: "Cross-distro package search",
    });

    // ===== Batch 3: More Music Platforms =====
    map.insert("deezer", EngineInfo {
        name: "deezer",
        display_name: "Deezer",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!dz",
        default_enabled: false,
        region: "",
        description: "Music streaming service",
    });

    map.insert("mixcloud", EngineInfo {
        name: "mixcloud",
        display_name: "Mixcloud",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!mx",
        default_enabled: false,
        region: "",
        description: "DJ mixes and radio shows",
    });

    map.insert("freesound", EngineInfo {
        name: "freesound",
        display_name: "Freesound",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::None,
        shortcut: "!fs",
        default_enabled: false,
        region: "",
        description: "Collaborative sound database",
    });

    map.insert("radio_browser", EngineInfo {
        name: "radio_browser",
        display_name: "Radio Browser",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Radio,
        shortcut: "!rb",
        default_enabled: false,
        region: "",
        description: "Internet radio directory",
    });

    map.insert("fyyd", EngineInfo {
        name: "fyyd",
        display_name: "fyyd",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Radio,
        shortcut: "!fyyd",
        default_enabled: false,
        region: "",
        description: "Podcast search engine",
    });

    map.insert("podcastindex", EngineInfo {
        name: "podcastindex",
        display_name: "Podcast Index",
        categories: &[EngineCategory::Music],
        subcategory: EngineSubcategory::Radio,
        shortcut: "!pod",
        default_enabled: false,
        region: "",
        description: "Open podcast database",
    });

    // ===== Batch 4: Science/Academic =====
    map.insert("base", EngineInfo {
        name: "base",
        display_name: "BASE",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!base",
        default_enabled: false,
        region: "",
        description: "Bielefeld Academic Search Engine",
    });

    map.insert("openalex", EngineInfo {
        name: "openalex",
        display_name: "OpenAlex",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!oal",
        default_enabled: false,
        region: "",
        description: "Open catalog of scholarly works",
    });

    map.insert("pdbe", EngineInfo {
        name: "pdbe",
        display_name: "PDBe",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!pdb",
        default_enabled: false,
        region: "",
        description: "Protein Data Bank in Europe",
    });

    map.insert("springer", EngineInfo {
        name: "springer",
        display_name: "Springer",
        categories: &[EngineCategory::Science],
        subcategory: EngineSubcategory::ScientificPublications,
        shortcut: "!spr",
        default_enabled: false,
        region: "",
        description: "Scientific publisher",
    });

    map.insert("annas_archive", EngineInfo {
        name: "annas_archive",
        display_name: "Anna's Archive",
        categories: &[EngineCategory::Science, EngineCategory::Files],
        subcategory: EngineSubcategory::Books,
        shortcut: "!anna",
        default_enabled: false,
        region: "",
        description: "Library search engine",
    });

    map.insert("zlibrary", EngineInfo {
        name: "zlibrary",
        display_name: "Z-Library",
        categories: &[EngineCategory::Science, EngineCategory::Files],
        subcategory: EngineSubcategory::Books,
        shortcut: "!zlib",
        default_enabled: false,
        region: "",
        description: "Electronic library",
    });

    map.insert("loc", EngineInfo {
        name: "loc",
        display_name: "Library of Congress",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Books,
        shortcut: "!loc",
        default_enabled: false,
        region: "US",
        description: "US national library",
    });

    map.insert("goodreads", EngineInfo {
        name: "goodreads",
        display_name: "Goodreads",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Books,
        shortcut: "!gr",
        default_enabled: false,
        region: "",
        description: "Book discovery platform",
    });

    // ===== Batch 4: Translation =====
    map.insert("deepl", EngineInfo {
        name: "deepl",
        display_name: "DeepL",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Translate,
        shortcut: "!dl",
        default_enabled: false,
        region: "",
        description: "AI-powered translation",
    });

    map.insert("libretranslate", EngineInfo {
        name: "libretranslate",
        display_name: "LibreTranslate",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Translate,
        shortcut: "!lt",
        default_enabled: false,
        region: "",
        description: "Open source translation",
    });

    map.insert("lingva", EngineInfo {
        name: "lingva",
        display_name: "Lingva",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Translate,
        shortcut: "!lgv",
        default_enabled: false,
        region: "",
        description: "Privacy-friendly translation",
    });

    map.insert("mozhi", EngineInfo {
        name: "mozhi",
        display_name: "Mozhi",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Translate,
        shortcut: "!mz",
        default_enabled: false,
        region: "",
        description: "Translation aggregator",
    });

    // ===== Batch 4: Currency =====
    map.insert("currency", EngineInfo {
        name: "currency",
        display_name: "Currency Converter",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Currency,
        shortcut: "!cur",
        default_enabled: false,
        region: "",
        description: "Currency conversion",
    });

    // ===== Batch 4: Weather =====
    map.insert("wttr", EngineInfo {
        name: "wttr",
        display_name: "wttr.in",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::None,
        shortcut: "!wttr",
        default_enabled: false,
        region: "",
        description: "Weather forecast service",
    });

    map.insert("open_meteo", EngineInfo {
        name: "open_meteo",
        display_name: "Open-Meteo",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::None,
        shortcut: "!met",
        default_enabled: false,
        region: "",
        description: "Open source weather API",
    });

    // ===== Batch 4: Reference/Dictionary =====
    map.insert("wordnik", EngineInfo {
        name: "wordnik",
        display_name: "Wordnik",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Dictionary,
        shortcut: "!wnik",
        default_enabled: false,
        region: "",
        description: "Online dictionary",
    });

    map.insert("etymonline", EngineInfo {
        name: "etymonline",
        display_name: "Etymonline",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Dictionary,
        shortcut: "!etym",
        default_enabled: false,
        region: "",
        description: "Etymology dictionary",
    });

    map.insert("dictzone", EngineInfo {
        name: "dictzone",
        display_name: "DictZone",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Dictionary,
        shortcut: "!dz",
        default_enabled: false,
        region: "",
        description: "Multilingual dictionary",
    });

    map.insert("duden", EngineInfo {
        name: "duden",
        display_name: "Duden",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Dictionary,
        shortcut: "!dud",
        default_enabled: false,
        region: "DE",
        description: "German dictionary",
    });

    map.insert("jisho", EngineInfo {
        name: "jisho",
        display_name: "Jisho",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Dictionary,
        shortcut: "!jp",
        default_enabled: false,
        region: "JP",
        description: "Japanese-English dictionary",
    });

    map.insert("emojipedia", EngineInfo {
        name: "emojipedia",
        display_name: "Emojipedia",
        categories: &[EngineCategory::Reference],
        subcategory: EngineSubcategory::Encyclopedia,
        shortcut: "!emoji",
        default_enabled: false,
        region: "",
        description: "Emoji reference",
    });

    // ===== Batch 5: More Image Engines =====
    map.insert("pinterest", EngineInfo {
        name: "pinterest",
        display_name: "Pinterest",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!pin",
        default_enabled: false,
        region: "",
        description: "Image sharing platform",
    });

    map.insert("imgur", EngineInfo {
        name: "imgur",
        display_name: "Imgur",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!img",
        default_enabled: false,
        region: "",
        description: "Image hosting service",
    });

    map.insert("openverse", EngineInfo {
        name: "openverse",
        display_name: "Openverse",
        categories: &[EngineCategory::Images, EngineCategory::Music],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!ov",
        default_enabled: false,
        region: "",
        description: "Open source media search",
    });

    map.insert("openclipart", EngineInfo {
        name: "openclipart",
        display_name: "Openclipart",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Icons,
        shortcut: "!ocl",
        default_enabled: false,
        region: "",
        description: "Free clipart library",
    });

    map.insert("svgrepo", EngineInfo {
        name: "svgrepo",
        display_name: "SVG Repo",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Icons,
        shortcut: "!svg",
        default_enabled: false,
        region: "",
        description: "Free SVG icons",
    });

    map.insert("material_icons", EngineInfo {
        name: "material_icons",
        display_name: "Material Icons",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Icons,
        shortcut: "!mat",
        default_enabled: false,
        region: "",
        description: "Google Material Design icons",
    });

    map.insert("uxwing", EngineInfo {
        name: "uxwing",
        display_name: "UXWing",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Icons,
        shortcut: "!ux",
        default_enabled: false,
        region: "",
        description: "Free icons collection",
    });

    map.insert("wallhaven", EngineInfo {
        name: "wallhaven",
        display_name: "Wallhaven",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!wh",
        default_enabled: false,
        region: "",
        description: "Wallpaper search",
    });

    map.insert("artic", EngineInfo {
        name: "artic",
        display_name: "Art Institute of Chicago",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!art",
        default_enabled: false,
        region: "US",
        description: "Art museum collection",
    });

    map.insert("adobe_stock", EngineInfo {
        name: "adobe_stock",
        display_name: "Adobe Stock",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!adobe",
        default_enabled: false,
        region: "",
        description: "Stock image library",
    });

    map.insert("tineye", EngineInfo {
        name: "tineye",
        display_name: "TinEye",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::None,
        shortcut: "!tin",
        default_enabled: false,
        region: "",
        description: "Reverse image search",
    });

    map.insert("ipernity", EngineInfo {
        name: "ipernity",
        display_name: "Ipernity",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!ip",
        default_enabled: false,
        region: "",
        description: "Photo sharing community",
    });

    map.insert("pixiv", EngineInfo {
        name: "pixiv",
        display_name: "Pixiv",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!pxv",
        default_enabled: false,
        region: "JP",
        description: "Japanese art community",
    });

    map.insert("findthatmeme", EngineInfo {
        name: "findthatmeme",
        display_name: "Find That Meme",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!meme",
        default_enabled: false,
        region: "",
        description: "Meme search engine",
    });

    map.insert("frinkiac", EngineInfo {
        name: "frinkiac",
        display_name: "Frinkiac",
        categories: &[EngineCategory::Images],
        subcategory: EngineSubcategory::Photos,
        shortcut: "!frink",
        default_enabled: false,
        region: "",
        description: "Simpsons screenshot search",
    });

    // ===== Batch 5: More Social Media =====
    map.insert("9gag", EngineInfo {
        name: "9gag",
        display_name: "9GAG",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::None,
        shortcut: "!9g",
        default_enabled: false,
        region: "",
        description: "Social media platform for memes",
    });

    map.insert("tootfinder", EngineInfo {
        name: "tootfinder",
        display_name: "Tootfinder",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::Fediverse,
        shortcut: "!toot",
        default_enabled: false,
        region: "",
        description: "Mastodon post search",
    });

    map.insert("discourse", EngineInfo {
        name: "discourse",
        display_name: "Discourse",
        categories: &[EngineCategory::SocialMedia],
        subcategory: EngineSubcategory::Forums,
        shortcut: "!disc",
        default_enabled: false,
        region: "",
        description: "Discussion platform search",
    });

    // ===== Batch 5: Chinese Video Platforms =====
    map.insert("iqiyi", EngineInfo {
        name: "iqiyi",
        display_name: "爱奇艺",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!iqy",
        default_enabled: false,
        region: "CN",
        description: "Chinese video streaming platform",
    });

    map.insert("acfun", EngineInfo {
        name: "acfun",
        display_name: "AcFun",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Anime,
        shortcut: "!acfun",
        default_enabled: false,
        region: "CN",
        description: "Chinese video sharing website",
    });

    map.insert("niconico", EngineInfo {
        name: "niconico",
        display_name: "Niconico",
        categories: &[EngineCategory::Videos],
        subcategory: EngineSubcategory::Streaming,
        shortcut: "!nico",
        default_enabled: false,
        region: "JP",
        description: "Japanese video sharing platform",
    });

    // ===== Batch 6: Map Engines =====
    map.insert("apple_maps", EngineInfo {
        name: "apple_maps",
        display_name: "Apple Maps",
        categories: &[EngineCategory::Map],
        subcategory: EngineSubcategory::None,
        shortcut: "!amap",
        default_enabled: false,
        region: "",
        description: "Apple's mapping service",
    });

    map.insert("photon", EngineInfo {
        name: "photon",
        display_name: "Photon",
        categories: &[EngineCategory::Map],
        subcategory: EngineSubcategory::None,
        shortcut: "!pho",
        default_enabled: false,
        region: "",
        description: "OpenStreetMap geocoder",
    });

    // ===== Batch 6: News Engines =====
    map.insert("reuters", EngineInfo {
        name: "reuters",
        display_name: "Reuters",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!reu",
        default_enabled: false,
        region: "",
        description: "International news agency",
    });

    map.insert("ansa", EngineInfo {
        name: "ansa",
        display_name: "ANSA",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!ansa",
        default_enabled: false,
        region: "IT",
        description: "Italian news agency",
    });

    map.insert("tagesschau", EngineInfo {
        name: "tagesschau",
        display_name: "Tagesschau",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!ts",
        default_enabled: false,
        region: "DE",
        description: "German public news",
    });

    map.insert("il_post", EngineInfo {
        name: "il_post",
        display_name: "Il Post",
        categories: &[EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!ilp",
        default_enabled: false,
        region: "IT",
        description: "Italian news",
    });

    map.insert("bpb", EngineInfo {
        name: "bpb",
        display_name: "BPB",
        categories: &[EngineCategory::News, EngineCategory::Reference],
        subcategory: EngineSubcategory::None,
        shortcut: "!bpb",
        default_enabled: false,
        region: "DE",
        description: "German federal civic education",
    });

    map.insert("ina", EngineInfo {
        name: "ina",
        display_name: "INA",
        categories: &[EngineCategory::News, EngineCategory::Videos],
        subcategory: EngineSubcategory::None,
        shortcut: "!ina",
        default_enabled: false,
        region: "FR",
        description: "French audiovisual archive",
    });

    map.insert("ccc_media", EngineInfo {
        name: "ccc_media",
        display_name: "CCC Media",
        categories: &[EngineCategory::Videos, EngineCategory::It],
        subcategory: EngineSubcategory::None,
        shortcut: "!ccc",
        default_enabled: false,
        region: "DE",
        description: "Chaos Computer Club media",
    });

    map.insert("mediathekviewweb", EngineInfo {
        name: "mediathekviewweb",
        display_name: "MediathekViewWeb",
        categories: &[EngineCategory::Videos, EngineCategory::News],
        subcategory: EngineSubcategory::None,
        shortcut: "!mvw",
        default_enabled: false,
        region: "DE",
        description: "German public TV archive",
    });

    // ===== Batch 6: Alternative General Search =====
    map.insert("marginalia", EngineInfo {
        name: "marginalia",
        display_name: "Marginalia",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!marg",
        default_enabled: false,
        region: "",
        description: "Independent search engine",
    });

    map.insert("wiby", EngineInfo {
        name: "wiby",
        display_name: "Wiby",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!wiby",
        default_enabled: false,
        region: "",
        description: "Search engine for classic web",
    });

    map.insert("mwmbl", EngineInfo {
        name: "mwmbl",
        display_name: "Mwmbl",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!mw",
        default_enabled: false,
        region: "",
        description: "Non-profit search engine",
    });

    map.insert("stract", EngineInfo {
        name: "stract",
        display_name: "Stract",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!str",
        default_enabled: false,
        region: "",
        description: "Open source web search",
    });

    map.insert("yep", EngineInfo {
        name: "yep",
        display_name: "Yep",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!yep",
        default_enabled: false,
        region: "",
        description: "Privacy-focused search",
    });

    map.insert("presearch", EngineInfo {
        name: "presearch",
        display_name: "Presearch",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!pre",
        default_enabled: false,
        region: "",
        description: "Decentralized search engine",
    });

    map.insert("seekr", EngineInfo {
        name: "seekr",
        display_name: "Seekr",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!seekr",
        default_enabled: false,
        region: "",
        description: "AI-powered search engine",
    });

    map.insert("seznam", EngineInfo {
        name: "seznam",
        display_name: "Seznam",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!szn",
        default_enabled: false,
        region: "CZ",
        description: "Czech search engine",
    });

    map.insert("quark", EngineInfo {
        name: "quark",
        display_name: "夸克",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!quark",
        default_enabled: false,
        region: "CN",
        description: "Alibaba's search engine",
    });

    // ===== Batch 6: AI/LLM Search =====
    map.insert("yacy", EngineInfo {
        name: "yacy",
        display_name: "YaCy",
        categories: &[EngineCategory::General],
        subcategory: EngineSubcategory::Web,
        shortcut: "!yacy",
        default_enabled: false,
        region: "",
        description: "Decentralized web search",
    });
}

/// Get all engines in a specific category
pub fn get_engines_by_category(category: EngineCategory) -> Vec<&'static EngineInfo> {
    get_engine_registry()
        .values()
        .filter(|info| info.categories.contains(&category))
        .collect()
}

/// Get all engines in a specific subcategory
pub fn get_engines_by_subcategory(subcategory: EngineSubcategory) -> Vec<&'static EngineInfo> {
    get_engine_registry()
        .values()
        .filter(|info| info.subcategory == subcategory)
        .collect()
}

/// Get engine info by name
pub fn get_engine_info(engine_name: &str) -> Option<&'static EngineInfo> {
    get_engine_registry().get(engine_name.to_lowercase().as_str())
}

/// Get all default enabled engines
pub fn get_default_engines() -> Vec<&'static EngineInfo> {
    get_engine_registry()
        .values()
        .filter(|info| info.default_enabled)
        .collect()
}

/// Get all engine names
pub fn get_all_engine_names() -> Vec<&'static str> {
    get_engine_registry().keys().copied().collect()
}

/// Get engines by region
pub fn get_engines_by_region(region: &str) -> Vec<&'static EngineInfo> {
    get_engine_registry()
        .values()
        .filter(|info| info.region.eq_ignore_ascii_case(region))
        .collect()
}

/// Parse bang syntax to find engine or category
pub fn parse_bang(input: &str) -> BangResult {
    let input = input.trim();
    if !input.starts_with('!') {
        return BangResult::NoBang;
    }

    let bang = &input[1..];

    // Check if it's a category bang
    if let Some(category) = EngineCategory::from_str(bang) {
        return BangResult::Category(category);
    }

    // Check if it's an engine shortcut
    for info in get_engine_registry().values() {
        let shortcut = info.shortcut.trim_start_matches('!');
        if shortcut.eq_ignore_ascii_case(bang) {
            return BangResult::Engine(info.name);
        }
    }

    // Check if it matches engine name directly
    if let Some(info) = get_engine_registry().get(bang.to_lowercase().as_str()) {
        return BangResult::Engine(info.name);
    }

    BangResult::Unknown(bang.to_string())
}

/// Result of parsing a bang syntax
#[derive(Debug, Clone)]
pub enum BangResult {
    /// No bang found in input
    NoBang,
    /// Matched a category (e.g., !images)
    Category(EngineCategory),
    /// Matched a specific engine (e.g., !g for Google)
    Engine(&'static str),
    /// Unknown bang
    Unknown(String),
}

/// Category information for API response
#[derive(Debug, Clone, Serialize)]
pub struct CategoryInfo {
    pub id: String,
    pub name: String,
    pub bang: String,
    pub engine_count: usize,
    pub engines: Vec<String>,
}

/// Get all categories with their engine counts
pub fn get_categories_info() -> Vec<CategoryInfo> {
    EngineCategory::all()
        .iter()
        .map(|cat| {
            let engines = get_engines_by_category(*cat);
            CategoryInfo {
                id: format!("{:?}", cat).to_lowercase(),
                name: cat.display_name().to_string(),
                bang: cat.bang().to_string(),
                engine_count: engines.len(),
                engines: engines.iter().map(|e| e.name.to_string()).collect(),
            }
        })
        .collect()
}

/// Filter engines by multiple categories
pub fn filter_engines_by_categories(categories: &[EngineCategory]) -> HashSet<&'static str> {
    let mut result = HashSet::new();
    for category in categories {
        for info in get_engines_by_category(*category) {
            result.insert(info.name);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_str() {
        assert_eq!(EngineCategory::from_str("general"), Some(EngineCategory::General));
        assert_eq!(EngineCategory::from_str("IMAGES"), Some(EngineCategory::Images));
        assert_eq!(EngineCategory::from_str("it"), Some(EngineCategory::It));
        assert_eq!(EngineCategory::from_str("unknown"), None);
    }

    #[test]
    fn test_parse_bang() {
        match parse_bang("!g") {
            BangResult::Engine(name) => assert_eq!(name, "google"),
            _ => panic!("Expected Engine result"),
        }

        match parse_bang("!images") {
            BangResult::Category(cat) => assert_eq!(cat, EngineCategory::Images),
            _ => panic!("Expected Category result"),
        }

        match parse_bang("no bang") {
            BangResult::NoBang => {}
            _ => panic!("Expected NoBang result"),
        }
    }

    #[test]
    fn test_get_engines_by_category() {
        let general_engines = get_engines_by_category(EngineCategory::General);
        assert!(general_engines.iter().any(|e| e.name == "google"));
        assert!(general_engines.iter().any(|e| e.name == "bing"));
    }

    #[test]
    fn test_get_default_engines() {
        let defaults = get_default_engines();
        assert!(defaults.iter().any(|e| e.name == "google"));
        assert!(defaults.iter().all(|e| e.default_enabled));
    }
}
