use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, sync::RwLock};

#[derive(Default)]
struct AppState {
    users: HashMap<String, String>,
}

type SharedStateType = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    let shared_state = SharedStateType::default();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(singup))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn singup(
    State(state): State<SharedStateType>,
    Json(payload): Json<SingupRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    users.insert(payload.username.clone(), payload.password);

    let response = SingupResponse {
        username: payload.username,
    };

    (StatusCode::CREATED, Json(response))
}

#[derive(Deserialize)]
struct SingupRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct SingupResponse {
    username: String,
}
