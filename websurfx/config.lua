-- ### General ###
logging = true -- an option to enable or disable logs.
debug = false -- an option to enable or disable debug mode.
threads = 10 -- the amount of threads that the app will use to run (the value should be greater than 0).

-- ### Server ###
port = "8080" -- port on which server should be launched
binding_ip = "127.0.0.1" --ip address on the which server should be launched.
production_use = false -- whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users (more than one))
-- if production_use is set to true
-- There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.
request_timeout = 30 -- timeout for the search requests sent to the upstream search engines to be fetched (value in seconds).
tcp_connection_keep_alive = 30 -- the amount of time the tcp connection should remain alive to the upstream search engines (or connected to the server). (value in seconds).
pool_idle_connection_timeout = 30 -- timeout for the idle connections in the reqwest HTTP connection pool (value in seconds).
rate_limiter = {
	number_of_requests = 20, -- The number of request that are allowed within a provided time limit.
	time_limit = 3, -- The time limit in which the quantity of requests that should be accepted.
}
-- Set whether the server will use an adaptive/dynamic HTTPS window size, see https://httpwg.org/specs/rfc9113.html#fc-principles
https_adaptive_window_size = true

operating_system_tls_certificates = true -- Set whether the server will use operating system's tls certificates alongside rustls certificates while fetching search results from the upstream engines.

number_of_https_connections = 10 -- the number of https connections that should be available in the connection pool.
-- Set keep-alive timer in seconds; keeps clients connected to the HTTP server, different from the connection to upstream search engines
client_connection_keep_alive = 120

-- ### Search ###
-- Filter results based on different levels. The levels provided are:
-- {{
-- 0 - None
-- 1 - Low
-- 2 - Moderate
-- 3 - High
-- 4 - Aggressive
-- }}
safe_search = 2

-- ### Website ###
-- The different colorschemes provided are:
-- {{
-- catppuccin-mocha
-- dark-chocolate
-- dracula
-- gruvbox-dark
-- monokai
-- nord
-- oceanic-next
-- one-dark
-- solarized-dark
-- solarized-light
-- tokyo-night
-- tomorrow-night
-- }}
colorscheme = "catppuccin-mocha" -- the colorscheme name which should be used for the website theme
-- The different themes provided are:
-- {{
-- simple
-- }}
theme = "simple" -- the theme name which should be used for the website
-- The different animations provided are:
-- {{
-- simple-frosted-glow
-- }}
animation = "simple-frosted-glow" -- the animation name which should be used with the theme or `nil` if you don't want any animations.

-- ### Caching ###
-- Cache backend type: "memory" for in-memory cache, "redis" for Redis cache
-- "memory" is recommended for client/desktop applications (no external dependencies)
-- "redis" requires a running Redis server
cache_backend = "memory"
redis_url = "redis://127.0.0.1:8082" -- redis connection url address on which the client should connect on (only used when cache_backend = "redis").
cache_expiry_time = 600 -- This option takes the expiry time of the search results (value in seconds and the value should be greater than or equal to 60 seconds).

-- ### Engine Health Management ###
-- Configuration for automatic engine suspension when errors occur
-- Similar to SearXNG's engine management functionality
engine_health = {
	-- Suspension times in seconds for different error types
	suspended_times = {
		access_denied = 86400,      -- 1 day for access denied/blocked (HTTP 403)
		captcha = 86400,            -- 1 day for CAPTCHA required
		too_many_requests = 3600,   -- 1 hour for rate limiting (HTTP 429)
		timeout = 60,               -- 1 minute for timeout errors
		ssl_error = 3600,           -- 1 hour for SSL/TLS errors
		http_error = 300,           -- 5 minutes for other HTTP errors
		parse_error = 600,          -- 10 minutes for parsing errors
	},
	-- Dynamic ban time settings
	ban_time_on_fail = 5,           -- Base ban time in seconds after an error
	max_ban_time_on_fail = 120,     -- Maximum ban time in seconds (exponential backoff cap)
	-- Whether to enable automatic suspension
	enable_auto_suspend = true,
}

-- ### Search Engines ###
-- ===== Auto-configured based on live test results =====
-- Test date: 2025-12-03
-- Passed: 12 engines, Failed: 8 engines
upstream_search_engines = {
	-- ✅ Enabled engines (passed connectivity test, sorted by response time)
	Search360 = true,    -- 84ms - 360 Search (fastest!)
	Baidu = true,        -- 142ms - Largest Chinese search engine
	Bing = true,         -- 413ms - Microsoft search
	Sogou = true,        -- 502ms - Major Chinese search engine
	Unsplash = true,     -- 736ms - Free photos
	HackerNews = true,   -- 749ms - Tech news and discussions
	GitHub = true,       -- 808ms - Code repositories
	Mojeek = true,       -- 1665ms - Privacy-focused UK search engine
	npm = true,          -- 1754ms - Node.js packages
	StackExchange = true, -- 1998ms - Q&A for developers
	arXiv = true,        -- 3894ms - Scientific papers
	Crates = true,       -- 8287ms - Rust packages (crates.io)

	-- ❌ Disabled engines (failed connectivity test)
	DuckDuckGo = false,  -- Timeout (15s)
	Brave = false,       -- Connection failed
	Startpage = false,   -- Timeout (15s)
	Google = false,      -- Timeout (15s) - May require anti-bot measures
	Qwant = false,       -- Connection failed
	Wikipedia = false,   -- Timeout (15s)
	PyPI = false,        -- Timeout (15s)
	IMDb = false,        -- Minimal response

	-- ⏸️ Not tested (kept previous state)
	Searx = false,
	LibreX = false,
	Yahoo = false,
	Yandex = false,      -- Russian search engine
	Ask = false,         -- Ask.com
	Chinaso = false,     -- Chinese national search engine
	Naver = false,       -- South Korean search engine
	Wikidata = false,    -- Structured knowledge base
	OpenLibrary = false, -- Free book library
	DockerHub = false,   -- Docker images
	HuggingFace = false, -- AI/ML models
	PubMed = false,      -- Medical literature
	Reddit = false,      -- Community discussions
	YouTube = false,     -- Video platform (via Invidious)
	Bilibili = false,    -- Chinese video platform
	SoundCloud = false,  -- Music streaming
} -- select the upstream search engines from which the results should be fetched.

proxy = nil -- Proxy to send outgoing requests through. Set to nil to disable.
