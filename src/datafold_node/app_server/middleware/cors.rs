use warp::cors::Cors;

/// Create a CORS configuration for the API server
pub fn create_cors() -> Cors {
    warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST", "GET", "OPTIONS"])
        .allow_headers(vec![
            "Content-Type",
            "x-public-key",
            "x-signature",
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
        ])
        .max_age(3600)
        .build()
}
