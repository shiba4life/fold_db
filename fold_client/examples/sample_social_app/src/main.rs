use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fold_client::ipc::client::{IpcClient, IpcClientError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Get the app ID and token from environment variables
    let app_id = std::env::var("FOLD_CLIENT_APP_ID").unwrap_or_else(|_| {
        eprintln!("FOLD_CLIENT_APP_ID environment variable not set");
        std::process::exit(1);
    });
    let token = std::env::var("FOLD_CLIENT_APP_TOKEN").unwrap_or_else(|_| {
        eprintln!("FOLD_CLIENT_APP_TOKEN environment variable not set");
        std::process::exit(1);
    });
    let socket_dir = std::env::var("FOLD_CLIENT_SOCKET_DIR").unwrap_or_else(|_| {
        eprintln!("FOLD_CLIENT_SOCKET_DIR environment variable not set");
        std::process::exit(1);
    });

    // Connect to the FoldClient
    let client = match IpcClient::connect(&PathBuf::from(socket_dir), &app_id, &token).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to FoldClient: {}", e);
            std::process::exit(1);
        }
    };

    // Create the app state
    let app_state = AppState {
        client: Arc::new(Mutex::new(client)),
    };

    // Create the router
    let app = Router::new()
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/posts", get(list_posts).post(create_post))
        .route("/api/comments", get(list_comments).post(create_comment))
        .route("/api/discover-nodes", get(discover_nodes))
        .route("/api/posts/:id/comments", get(get_post_comments))
        .nest_service("/", ServeDir::new("static"))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(app_state);

    // Start the server
    let addr = "0.0.0.0:3000";
    println!("Listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// App state
struct AppState {
    client: Arc<Mutex<IpcClient>>,
}

// User model
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Option<String>,
    username: String,
    full_name: Option<String>,
    bio: Option<String>,
    created_at: Option<String>,
}

// Post model
#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: Option<String>,
    title: String,
    content: String,
    author_id: String,
    created_at: Option<String>,
}

// Comment model
#[derive(Debug, Serialize, Deserialize)]
struct Comment {
    id: Option<String>,
    content: String,
    author_id: String,
    post_id: String,
    created_at: Option<String>,
}

// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// List users handler
async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    match client.query("user", &["*"], None).await {
        Ok(users) => (StatusCode::OK, Json(users)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// Create user handler
async fn create_user(
    State(state): State<AppState>,
    Json(mut user): Json<User>,
) -> impl IntoResponse {
    // Generate an ID and timestamp if not provided
    user.id = Some(user.id.unwrap_or_else(|| Uuid::new_v4().to_string()));
    user.created_at = Some(
        user.created_at
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
    );

    // Create the user
    let mut client = state.client.lock().await;
    match client
        .create("user", serde_json::to_value(user).unwrap())
        .await
    {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// List posts handler
async fn list_posts(State(state): State<AppState>) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    match client.query("post", &["*"], None).await {
        Ok(posts) => (StatusCode::OK, Json(posts)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// Create post handler
async fn create_post(
    State(state): State<AppState>,
    Json(mut post): Json<Post>,
) -> impl IntoResponse {
    // Generate an ID and timestamp if not provided
    post.id = Some(post.id.unwrap_or_else(|| Uuid::new_v4().to_string()));
    post.created_at = Some(
        post.created_at
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
    );

    // Create the post
    let mut client = state.client.lock().await;
    match client
        .create("post", serde_json::to_value(post).unwrap())
        .await
    {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// List comments handler
async fn list_comments(State(state): State<AppState>) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    match client.query("comment", &["*"], None).await {
        Ok(comments) => (StatusCode::OK, Json(comments)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// Create comment handler
async fn create_comment(
    State(state): State<AppState>,
    Json(mut comment): Json<Comment>,
) -> impl IntoResponse {
    // Generate an ID and timestamp if not provided
    comment.id = Some(comment.id.unwrap_or_else(|| Uuid::new_v4().to_string()));
    comment.created_at = Some(
        comment
            .created_at
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
    );

    // Create the comment
    let mut client = state.client.lock().await;
    match client
        .create("comment", serde_json::to_value(comment).unwrap())
        .await
    {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// Get post comments handler
async fn get_post_comments(
    State(state): State<AppState>,
    Path(post_id): Path<String>,
) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    let filter = serde_json::json!({
        "field": "post_id",
        "operator": "eq",
        "value": post_id
    });
    match client.query("comment", &["*"], Some(filter)).await {
        Ok(comments) => (StatusCode::OK, Json(comments)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}

// Discover nodes handler
async fn discover_nodes(State(state): State<AppState>) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    match client.discover_nodes().await {
        Ok(nodes) => (StatusCode::OK, Json(nodes)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
    }
}
