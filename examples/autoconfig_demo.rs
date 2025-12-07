//! Demo script to show automatic engine configuration based on test results
//!
//! This script:
//! 1. Tests all configured search engines
//! 2. Records success/failure for each engine
//! 3. Updates config.lua to enable/disable engines based on results

use std::collections::HashMap;
use std::fs;
use std::time::Duration;

/// Test result for an engine
#[derive(Debug, Clone)]
struct EngineTestResult {
    name: String,
    success: bool,
    response_time_ms: u64,
    result_count: usize,
    error: Option<String>,
}

/// Run a simple connectivity test for an engine
async fn test_engine(name: &str, search_url: &str, timeout_secs: u64) -> EngineTestResult {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .unwrap();

    let start = std::time::Instant::now();

    match client.get(search_url).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let status = response.status();

            if status.is_success() {
                let body = response.text().await.unwrap_or_default();
                let has_results = body.len() > 1000; // Simple check for content

                EngineTestResult {
                    name: name.to_string(),
                    success: has_results,
                    response_time_ms: elapsed,
                    result_count: if has_results { 1 } else { 0 },
                    error: if has_results { None } else { Some("Empty or minimal response".to_string()) },
                }
            } else {
                EngineTestResult {
                    name: name.to_string(),
                    success: false,
                    response_time_ms: elapsed,
                    result_count: 0,
                    error: Some(format!("HTTP {}", status.as_u16())),
                }
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let error_msg = if e.is_timeout() {
                "Timeout"
            } else if e.is_connect() {
                "Connection failed"
            } else {
                "Request failed"
            };

            EngineTestResult {
                name: name.to_string(),
                success: false,
                response_time_ms: elapsed,
                result_count: 0,
                error: Some(error_msg.to_string()),
            }
        }
    }
}

/// Generate updated config.lua content
fn generate_config_update(results: &[EngineTestResult]) -> String {
    let mut lines = vec!["upstream_search_engines = {".to_string()];

    // Sort by success then by name
    let mut sorted: Vec<_> = results.iter().collect();
    sorted.sort_by(|a, b| {
        b.success.cmp(&a.success)
            .then_with(|| a.name.cmp(&b.name))
    });

    for result in sorted {
        let enabled = result.success;
        let comment = if result.success {
            format!("-- OK: {}ms, {} results", result.response_time_ms, result.result_count)
        } else {
            format!("-- DISABLED: {}", result.error.as_deref().unwrap_or("Unknown error"))
        };

        lines.push(format!("\t{} = {}, {}", result.name, enabled, comment));
    }

    lines.push("}".to_string());
    lines.join("\n")
}

#[tokio::main]
async fn main() {
    println!("=== Engine Auto-Configuration Demo ===\n");

    // Define engines to test with their search URLs
    let engines = vec![
        ("DuckDuckGo", "https://duckduckgo.com/html/?q=test"),
        ("Bing", "https://www.bing.com/search?q=test"),
        ("Brave", "https://search.brave.com/search?q=test"),
        ("Wikipedia", "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch=test&format=json"),
        ("GitHub", "https://api.github.com/search/repositories?q=test"),
        ("Baidu", "https://www.baidu.com/s?wd=test"),
        ("Sogou", "https://www.sogou.com/web?query=test"),
        ("HackerNews", "https://hn.algolia.com/api/v1/search?query=test"),
        ("arXiv", "http://export.arxiv.org/api/query?search_query=all:test&max_results=1"),
    ];

    println!("Testing {} engines...\n", engines.len());

    let mut results = Vec::new();

    for (name, url) in engines {
        print!("Testing {}... ", name);
        let result = test_engine(name, url, 10).await;

        if result.success {
            println!("✅ OK ({}ms)", result.response_time_ms);
        } else {
            println!("❌ FAILED: {}", result.error.as_deref().unwrap_or("Unknown"));
        }

        results.push(result);
    }

    println!("\n=== Results Summary ===\n");

    let passed: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    println!("Passed: {} engines", passed.len());
    for r in &passed {
        println!("  ✅ {} ({}ms)", r.name, r.response_time_ms);
    }

    println!("\nFailed: {} engines", failed.len());
    for r in &failed {
        println!("  ❌ {} - {}", r.name, r.error.as_deref().unwrap_or("Unknown"));
    }

    println!("\n=== Generated Config ===\n");
    let config = generate_config_update(&results);
    println!("{}", config);

    println!("\n=== Recommendation ===");
    println!("Based on test results:");
    println!("- Enable {} engines that passed", passed.len());
    println!("- Disable {} engines that failed", failed.len());
}
