//! This module provides the functionality to parse the lua config and convert the config options
//! into rust readable form.

use crate::handler::{FileType, file_path};
use crate::models::engine_health::{SuspendedTimesConfig, init_health_manager};
use crate::models::parser::{AggregatorConfig, CacheBackend, EngineHealthConfig, RateLimiter, Style};
use log::LevelFilter;
use mlua::Lua;
use reqwest::Proxy;
use std::{collections::HashMap, fs, thread::available_parallelism};

/// A named struct which stores the parsed config file options.
pub struct Config {
    /// It stores the parsed port number option on which the server should launch.
    pub port: u16,
    /// It stores the parsed ip address option on which the server should launch
    pub binding_ip: String,
    /// It stores the theming options for the website.
    pub style: Style,
    /// It stores the cache backend type (memory or redis).
    pub cache_backend: CacheBackend,
    /// It stores the redis connection url address on which the redis
    /// client should connect.
    pub redis_url: String,
    /// It stores the max TTL for search results in cache.
    pub cache_expiry_time: u16,
    /// It stores the option to whether enable or disable production use.
    pub aggregator: AggregatorConfig,
    /// It stores the option to whether enable or disable logs.
    pub logging: bool,
    /// It stores the option to whether enable or disable debug mode.
    pub debug: bool,
    /// It toggles whether to use adaptive HTTP windows
    pub adaptive_window: bool,
    /// It stores all the engine names that were enabled by the user.
    pub upstream_search_engines: HashMap<String, bool>,
    /// It stores the time (secs) which controls the server request timeout.
    pub request_timeout: u8,
    /// It stores the number of threads which controls the app will use to run.
    pub threads: u8,
    /// Set the keep-alive time for client connections to the HTTP server
    pub client_connection_keep_alive: u8,
    /// It stores configuration options for the ratelimiting middleware.
    pub rate_limiter: RateLimiter,
    /// It stores the level of safe search to be used for restricting content in the
    /// search results.
    pub safe_search: u8,
    /// It stores the TCP connection keepalive duration in seconds.
    pub tcp_connection_keep_alive: u8,
    /// It stores the pool idle connection timeout in seconds.
    pub pool_idle_connection_timeout: u8,
    /// Url of the proxy to use for outgoing requests.
    pub proxy: Option<Proxy>,
    /// It stores the number of https connections to keep in the pool.
    pub number_of_https_connections: u8,
    /// It stores the operating system's TLS certificates for https requests.
    pub operating_system_tls_certificates: bool,
    /// Configuration for engine health management.
    pub engine_health: EngineHealthConfig,
}

impl Config {
    /// A function which parses the config.lua file and puts all the parsed options in the newly
    /// constructed Config struct and returns it.
    ///
    /// # Arguments
    ///
    /// * `logging_initialized` - It takes a boolean which ensures that the logging doesn't get
    ///   initialized twice. Pass false if the logger has not yet been initialized.
    ///
    /// # Error
    ///
    /// Returns a lua parse error if parsing of the config.lua file fails or has a syntax error
    /// or io error if the config.lua file doesn't exists otherwise it returns a newly constructed
    /// Config struct with all the parsed config options from the parsed config file.
    pub async fn parse(logging_initialized: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let globals = lua.globals();

        lua.load(&fs::read_to_string(file_path(FileType::Config).await?)?)
            .exec_async()
            .await?;

        let parsed_threads: u8 = globals.get("threads")?;

        let debug: bool = globals.get("debug")?;
        let logging: bool = globals.get("logging")?;
        let adaptive_window: bool = globals.get("adaptive_window")?;

        if !logging_initialized {
            set_logging_level(debug, logging);
        }

        let threads: u8 = if parsed_threads == 0 {
            let total_num_of_threads: usize = available_parallelism()?.get() / 2;
            log::error!(
                "Config Error: The value of `threads` option should be a non zero positive integer"
            );
            log::error!("Falling back to using {} threads", total_num_of_threads);
            total_num_of_threads as u8
        } else {
            parsed_threads
        };

        let rate_limiter: HashMap<String, u8> = globals.get("rate_limiter")?;

        let parsed_safe_search: u8 = globals.get("safe_search")?;
        let safe_search: u8 = if parsed_safe_search <= 4 {
            parsed_safe_search
        } else {
            log::error!(
                "Config Error: The value of `safe_search` option should be a non zero positive integer from 0 to 4."
            );
            log::error!("Falling back to using the value `1` for the option");
            1
        };

        let parsed_cet: u16 = globals.get("cache_expiry_time")?;
        let cache_expiry_time = if parsed_cet < 60 {
            log::error!("Config Error: The value of `cache_expiry_time` must be greater than 60");
            log::error!("Falling back to using the value `60` for the option");
            60
        } else {
            parsed_cet
        };

        // Parse cache backend type
        let cache_backend_str: String = globals.get("cache_backend").unwrap_or_else(|_| "memory".to_string());
        let cache_backend = CacheBackend::from_str(&cache_backend_str);
        log::info!("Using cache backend: {:?}", cache_backend);

        let proxy_opt: Option<String> = globals.get("proxy")?;
        let proxy = proxy_opt.and_then(|proxy_str| {
            Proxy::all(proxy_str).ok().and_then(|_| {
                log::error!("Invalid proxy url, defaulting to no proxy.");
                None
            })
        });

        // Parse engine health configuration
        let engine_health_config = parse_engine_health_config(&globals);

        // Initialize the health manager with the parsed configuration
        init_health_manager(engine_health_config.suspended_times.clone());
        log::info!("Engine health management initialized (auto_suspend: {})", engine_health_config.enable_auto_suspend);

        Ok(Config {
            operating_system_tls_certificates: globals.get("operating_system_tls_certificates")?,
            port: globals.get("port")?,
            binding_ip: globals.get("binding_ip")?,
            style: Style::new(
                globals.get("theme")?,
                globals.get("colorscheme")?,
                globals.get("animation")?,
            ),
            cache_backend,
            redis_url: globals.get("redis_url").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            cache_expiry_time,
            aggregator: AggregatorConfig {
                random_delay: globals.get("production_use")?,
            },
            logging,
            debug,
            adaptive_window,
            upstream_search_engines: globals.get("upstream_search_engines")?,
            request_timeout: globals.get("request_timeout")?,
            tcp_connection_keep_alive: globals.get("tcp_connection_keep_alive")?,
            pool_idle_connection_timeout: globals.get("pool_idle_connection_timeout")?,
            number_of_https_connections: globals.get("number_of_https_connections")?,
            threads,
            client_connection_keep_alive: globals.get("client_connection_keep_alive")?,
            rate_limiter: RateLimiter {
                number_of_requests: rate_limiter["number_of_requests"],
                time_limit: rate_limiter["time_limit"],
            },
            safe_search,
            proxy,
            engine_health: engine_health_config,
        })
    }
}

/// Parse engine health configuration from Lua globals
fn parse_engine_health_config(globals: &mlua::Table) -> EngineHealthConfig {
    let default_config = EngineHealthConfig::default();

    // Try to get engine_health table from config
    let engine_health_table: Option<HashMap<String, mlua::Value>> = globals.get("engine_health").ok();

    if engine_health_table.is_none() {
        log::info!("No engine_health configuration found, using defaults");
        return default_config;
    }

    let engine_health = engine_health_table.unwrap();

    // Parse enable_auto_suspend
    let enable_auto_suspend = engine_health
        .get("enable_auto_suspend")
        .and_then(|v| match v {
            mlua::Value::Boolean(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(true);

    // Parse ban times
    let ban_time_on_fail = engine_health
        .get("ban_time_on_fail")
        .and_then(|v| match v {
            mlua::Value::Integer(i) => Some(*i as u64),
            mlua::Value::Number(n) => Some(*n as u64),
            _ => None,
        })
        .unwrap_or(default_config.suspended_times.ban_time_on_fail);

    let max_ban_time_on_fail = engine_health
        .get("max_ban_time_on_fail")
        .and_then(|v| match v {
            mlua::Value::Integer(i) => Some(*i as u64),
            mlua::Value::Number(n) => Some(*n as u64),
            _ => None,
        })
        .unwrap_or(default_config.suspended_times.max_ban_time_on_fail);

    // Parse suspended_times table
    let suspended_times_table: Option<HashMap<String, mlua::Value>> = engine_health
        .get("suspended_times")
        .and_then(|v| match v {
            mlua::Value::Table(t) => {
                let mut map = HashMap::new();
                for pair in t.clone().pairs::<String, mlua::Value>() {
                    if let Ok((k, v)) = pair {
                        map.insert(k, v);
                    }
                }
                Some(map)
            }
            _ => None,
        });

    let mut suspended_times = SuspendedTimesConfig::default();
    suspended_times.ban_time_on_fail = ban_time_on_fail;
    suspended_times.max_ban_time_on_fail = max_ban_time_on_fail;

    if let Some(times) = suspended_times_table {
        suspended_times.access_denied = times
            .get("access_denied")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.access_denied);

        suspended_times.captcha = times
            .get("captcha")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.captcha);

        suspended_times.too_many_requests = times
            .get("too_many_requests")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.too_many_requests);

        suspended_times.timeout = times
            .get("timeout")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.timeout);

        suspended_times.ssl_error = times
            .get("ssl_error")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.ssl_error);

        suspended_times.http_error = times
            .get("http_error")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.http_error);

        suspended_times.parse_error = times
            .get("parse_error")
            .and_then(|v| match v {
                mlua::Value::Integer(i) => Some(*i as u64),
                mlua::Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(suspended_times.parse_error);
    }

    EngineHealthConfig {
        suspended_times,
        enable_auto_suspend,
    }
}

/// a helper function that sets the proper logging level
///
/// # Arguments
///
/// * `debug` - It takes the option to whether enable or disable debug mode.
/// * `logging` - It takes the option to whether enable or disable logs.
fn set_logging_level(debug: bool, logging: bool) {
    if let Ok(pkg_env_var) = std::env::var("PKG_ENV") {
        if pkg_env_var.to_lowercase() == "dev" {
            env_logger::Builder::new()
                .filter(None, LevelFilter::Info)
                .init();
            return;
        }
    }

    // Initializing logging middleware with level set to default or info.
    let log_level = match (debug, logging) {
        (true, true) => LevelFilter::Debug,
        (true, false) => LevelFilter::Debug,
        (false, true) => LevelFilter::Info,
        (false, false) => LevelFilter::Error,
    };

    env_logger::Builder::new().filter(None, log_level).init();
}
