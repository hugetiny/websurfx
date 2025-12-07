//! Real network test for engine auto-configuration
//!
//! Run with: cargo test --test autoconfig_live -- --nocapture

use std::fs;
use std::time::{Duration, Instant};

/// Test result for an engine
#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    success: bool,
    response_time_ms: u64,
    error: Option<String>,
}

/// Simple HTTP client test
async fn test_url(name: &str, url: &str) -> TestResult {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let start = Instant::now();

    match client.get(url).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let status = response.status();

            if status.is_success() || status.as_u16() == 302 || status.as_u16() == 301 {
                let body = response.text().await.unwrap_or_default();
                let has_content = body.len() > 500;

                TestResult {
                    name: name.to_string(),
                    success: has_content,
                    response_time_ms: elapsed,
                    error: if has_content { None } else { Some("Minimal response".to_string()) },
                }
            } else {
                TestResult {
                    name: name.to_string(),
                    success: false,
                    response_time_ms: elapsed,
                    error: Some(format!("HTTP {}", status.as_u16())),
                }
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            TestResult {
                name: name.to_string(),
                success: false,
                response_time_ms: elapsed,
                error: Some(if e.is_timeout() { "Timeout".to_string() }
                           else if e.is_connect() { "Connection failed".to_string() }
                           else { format!("{}", e) }),
            }
        }
    }
}

/// Generate Lua config section
fn generate_lua_config(results: &[TestResult]) -> String {
    let mut lines = vec![];
    lines.push("upstream_search_engines = {".to_string());
    lines.push("\t-- ===== Auto-generated based on test results =====".to_string());

    // Separate by success
    let mut passed: Vec<_> = results.iter().filter(|r| r.success).collect();
    let mut failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    passed.sort_by_key(|r| r.response_time_ms);
    failed.sort_by_key(|r| &r.name);

    lines.push("\t-- ✅ Enabled engines (passed tests)".to_string());
    for r in &passed {
        lines.push(format!("\t{} = true, -- {}ms", r.name, r.response_time_ms));
    }

    lines.push("".to_string());
    lines.push("\t-- ❌ Disabled engines (failed tests)".to_string());
    for r in &failed {
        let err = r.error.as_deref().unwrap_or("Unknown");
        lines.push(format!("\t{} = false, -- {}", r.name, err));
    }

    lines.push("}".to_string());
    lines.join("\n")
}

#[tokio::test]
async fn test_engines_and_update_config() {
    println!("\n============================================================");
    println!("=== Engine Auto-Configuration Live Test ===");
    println!("============================================================\n");

    // Define test endpoints for each engine
    let engines = vec![
        // General Search
        ("DuckDuckGo", "https://duckduckgo.com/html/?q=test"),
        ("Bing", "https://www.bing.com/search?q=test"),
        ("Brave", "https://search.brave.com/search?q=test"),
        ("Google", "https://www.google.com/search?q=test"),
        ("Startpage", "https://www.startpage.com/do/search?q=test"),
        ("Mojeek", "https://www.mojeek.com/search?q=test"),
        ("Qwant", "https://www.qwant.com/?q=test"),

        // Chinese Engines
        ("Baidu", "https://www.baidu.com/s?wd=test"),
        ("Sogou", "https://www.sogou.com/web?query=test"),
        ("Search360", "https://www.so.com/s?q=test"),

        // Reference
        ("Wikipedia", "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch=test&format=json"),

        // Developer
        ("GitHub", "https://api.github.com/search/repositories?q=test&per_page=1"),
        ("StackExchange", "https://api.stackexchange.com/2.3/search?order=desc&sort=activity&intitle=test&site=stackoverflow"),
        ("npm", "https://registry.npmjs.org/-/v1/search?text=test&size=1"),
        ("PyPI", "https://pypi.org/pypi/requests/json"),
        ("Crates", "https://crates.io/api/v1/crates?q=test&per_page=1"),

        // Academic
        ("arXiv", "http://export.arxiv.org/api/query?search_query=all:test&max_results=1"),

        // Social
        ("HackerNews", "https://hn.algolia.com/api/v1/search?query=test&hitsPerPage=1"),

        // Media
        ("IMDb", "https://www.imdb.com/find/?q=test"),
        ("Unsplash", "https://unsplash.com/s/photos/test"),
    ];

    println!("Testing {} engines...\n", engines.len());
    println!("{:<15} {:>10} {:>8} {}", "Engine", "Time", "Status", "Details");
    println!("{}", "-".repeat(60));

    let mut results = Vec::new();

    for (name, url) in &engines {
        let result = test_url(name, url).await;

        let status = if result.success { "✅ OK" } else { "❌ FAIL" };
        let details = result.error.as_deref().unwrap_or("");
        println!("{:<15} {:>7}ms {:>8} {}", name, result.response_time_ms, status, details);

        results.push(result);

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Summary
    let passed = results.iter().filter(|r| r.success).count();
    let failed = results.len() - passed;

    println!("\n{}", "=".repeat(60));
    println!("=== Summary ===");
    println!("{}", "=".repeat(60));
    println!("Total tested: {}", results.len());
    println!("✅ Passed: {} ({:.1}%)", passed, (passed as f64 / results.len() as f64) * 100.0);
    println!("❌ Failed: {} ({:.1}%)", failed, (failed as f64 / results.len() as f64) * 100.0);

    // Generate config
    println!("\n{}", "=".repeat(60));
    println!("=== Generated config.lua Section ===");
    println!("{}", "=".repeat(60));
    let config = generate_lua_config(&results);
    println!("{}", config);

    // Write to file
    let config_path = "websurfx/config_autotest.lua";
    if let Err(e) = fs::write(config_path, &config) {
        println!("\n⚠️  Could not write to {}: {}", config_path, e);
    } else {
        println!("\n📝 Saved to {}", config_path);
    }

    println!("\n{}", "=".repeat(60));
    println!("=== Recommendations ===");
    println!("{}", "=".repeat(60));

    // Find fastest engines
    let mut fastest: Vec<_> = results.iter().filter(|r| r.success).collect();
    fastest.sort_by_key(|r| r.response_time_ms);

    println!("\n🚀 Top 5 Fastest Engines:");
    for (i, r) in fastest.iter().take(5).enumerate() {
        println!("  {}. {} ({}ms)", i + 1, r.name, r.response_time_ms);
    }

    if failed > 0 {
        println!("\n⚠️  {} engines failed and should be disabled:", failed);
        for r in results.iter().filter(|r| !r.success) {
            println!("  - {} ({})", r.name, r.error.as_deref().unwrap_or("Unknown"));
        }
    }
}
