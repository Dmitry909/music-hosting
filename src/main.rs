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

type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    let shared_state = SharedState::default();

    let app = Router::new()
        .route("/", get(root))
        .route("/singup", post(singup))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn singup(
    State(state): State<SharedState>,
    Json(input_payload): Json<SingupRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    users.insert(input_payload.username.clone(), input_payload.password);

    let response = SingupResponse {
        username: input_payload.username,
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    fn send_get_request(uri: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .method("GET")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn get_by_name_success() {
        let shared_state = SharedState::default();

        let app = Router::new()
            .route("/", get(root))
            .route("/singup", post(singup))
            .with_state(shared_state);

        let response = app.oneshot(send_get_request("/")).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
