//! This main library module provides the functionality to provide and handle the Tcp server
//! and register all the routes for the `websurfx` meta search engine website.

mod aggregator;
mod cache;
pub mod checker_scheduler;
pub mod config_manager;
pub mod engine_checker;
mod engines;
mod handler;
pub mod metrics;
pub mod models;
pub mod parser;
mod routes;
pub mod templates;
mod user_agent;

use actix_cors::Cors;
use actix_files as fs;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    App, HttpServer,
    dev::Server,
    http::header,
    middleware::{Compress, Logger},
    web,
};
use handler::{FileType, file_path};
use parser::Config;
use tokio::{net::TcpListener, time::Duration};

/// Runs the web server on the provided TCP listener and returns a `Server` instance.
///
/// # Arguments
///
/// * `listener` - A `TcpListener` instance representing the address and port to listen on.
///
/// # Returns
///
/// Returns a `Result` containing a `Server` instance on success, or an `std::io::Error` on failure.
///
/// # Example
///
/// ```rust
/// use tokio::{sync::OnceCell, net::TcpListener};
/// use websurfx::{parser::Config, run};
///
/// /// A static constant for holding the parsed config.
/// static CONFIG: OnceCell<Config> = OnceCell::const_new();
///
/// #[tokio::main]
/// async fn main(){
///     // Initialize the parsed config globally.
///     let config = CONFIG
///        .get_or_try_init(|| async move {
///            Config::parse(false)
///                .await
///                .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e.to_string()))
///        })
///        .await
///        .unwrap();
///     let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind address");
///     let server = run(listener,&config).await.expect("Failed to start server");
/// }
/// ```
pub async fn run(listener: TcpListener, config: &'static Config) -> tokio::io::Result<Server> {
    let public_folder_path = file_path(FileType::Theme).await?;

    let server = HttpServer::new(move || {
        let cors: Cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::ORIGIN,
                header::CONTENT_TYPE,
                header::REFERER,
                header::COOKIE,
            ]);

        App::new()
            // Compress the responses provided by the server for the client requests.
            .wrap(Compress::default())
            .wrap(Logger::default()) // added logging middleware for logging.
            .app_data(web::Data::new(config))
            .wrap(cors)
            .wrap(Governor::new(
                &GovernorConfigBuilder::default()
                    .seconds_per_request(config.rate_limiter.time_limit as u64)
                    .burst_size(config.rate_limiter.number_of_requests as u32)
                    .finish()
                    .unwrap(),
            ))
            // Serve images and static files (css and js files).
            .service(
                fs::Files::new("/static", format!("{}/static", public_folder_path))
                    .show_files_listing(),
            )
            .service(
                fs::Files::new("/images", format!("{}/images", public_folder_path))
                    .show_files_listing(),
            )
            .service(routes::robots_data) // robots.txt
            .service(routes::index) // index page
            .service(routes::search::search) // search page
            .service(routes::about) // about page
            .service(routes::settings) // settings page
            .service(routes::export_import::download) // download page
            // Stats API routes (legacy + SearXNG-compatible)
            .service(routes::stats::get_all_stats)
            .service(routes::stats::get_engine_stats)
            .service(routes::stats::get_suspended_engines)
            .service(routes::stats::resume_engine)
            .service(routes::stats::reset_stats)
            .service(routes::stats::resume_all_engines)
            // SearXNG-compatible metrics routes
            .service(routes::stats::get_reliabilities)
            .service(routes::stats::get_openmetrics)
            .service(routes::stats::get_checker_status)
            .service(routes::stats::run_checker)
            .service(routes::stats::update_checker_config)
            .service(routes::stats::get_checker_config)
            // Checker API routes
            .service(routes::checker::get_checker_status)
            .service(routes::checker::get_engine_check_status)
            .service(routes::checker::check_single_engine)
            .service(routes::checker::check_multiple_engines)
            .service(routes::checker::get_healthy_engines)
            .service(routes::checker::get_unhealthy_engines)
            .service(routes::checker::reset_engine_check_status)
            .service(routes::checker::reset_all_check_statuses)
            // Categories API routes
            .service(routes::categories::list_categories)
            .service(routes::categories::list_engines)
            .service(routes::categories::list_default_engines)
            .service(routes::categories::parse_bang_query)
            .service(routes::categories::list_bangs)
            .service(routes::categories::get_category)
            .service(routes::categories::get_engine)
            .service(routes::categories::list_engines_by_region)
            // Health API routes
            .service(routes::health::health)
            .service(routes::health::reset_engine)
            .service(routes::health::reset_all)
            // SearXNG-compatible API routes (v1)
            .service(routes::searxng_api::search_get)
            .service(routes::searxng_api::search_post)
            .service(routes::searxng_api::get_config)
            .service(routes::searxng_api::list_categories_v1)
            .service(routes::searxng_api::list_engines_v1)
            .service(routes::searxng_api::autocomplete)
            .service(routes::searxng_api::get_preferences)
            .service(routes::searxng_api::get_languages)
            .service(routes::searxng_api::get_regions)
            .service(routes::searxng_api::get_time_ranges)
            .service(routes::searxng_api::get_safesearch_options)
            .service(routes::searxng_api::get_formats)
            .service(routes::searxng_api::get_plugins)
            .service(routes::searxng_api::get_engine_stats)
            .service(routes::searxng_api::toggle_engine)
            .service(routes::searxng_api::get_shortcuts)
            .service(routes::searxng_api::get_info)
            // OpenSearch routes
            .service(routes::opensearch::opensearch_xml)
            .service(routes::opensearch::autocomplete)
            .service(routes::opensearch::autocomplete_google)
            .service(routes::opensearch::autocomplete_wikipedia)
            .service(routes::opensearch::autocomplete_startpage)
            // Proxy routes
            .service(routes::proxy::image_proxy)
            .service(routes::proxy::morty_proxy)
            .service(routes::proxy::redirect_url)
            .service(routes::proxy::favicon_proxy)
            // Auto-config API routes (automatic engine enable/disable based on test results)
            .service(routes::autoconfig::get_status)
            .service(routes::autoconfig::get_settings)
            .service(routes::autoconfig::update_settings)
            .service(routes::autoconfig::run_config)
            .service(routes::autoconfig::get_enabled)
            .service(routes::autoconfig::get_disabled)
            .service(routes::autoconfig::enable_engine)
            .service(routes::autoconfig::disable_engine)
            .service(routes::autoconfig::get_lua_config)
            .service(routes::autoconfig::save_config)
            .service(routes::autoconfig::load_config)
            .default_service(web::route().to(routes::not_found)) // error page
    })
    .workers(config.threads as usize)
    // Set the keep-alive timer for client connections
    .keep_alive(Duration::from_secs(
        config.client_connection_keep_alive as u64,
    ))
    // Start server on 127.0.0.1 with the user provided port number. for example 127.0.0.1:8080.
    .listen(listener.into_std()?)?
    .run();
    Ok(server)
}
