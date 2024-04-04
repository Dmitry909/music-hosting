use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc, sync::RwLock};
use std::{io::Read, str};

pub fn add_two(a: i32) -> i32 {
    internal_adder(a, 2)
}

fn internal_adder(a: i32, b: i32) -> i32 {
    a + b
}

#[derive(Default)]
struct AppState {
    users: HashMap<String, String>,
}

type SharedState = Arc<RwLock<AppState>>;

pub fn create_app() -> Router {
    let shared_state = SharedState::default();
    Router::new()
        .route("/singup", post(singup))
        .route("/delete_account", delete(delete_account))
        .with_state(shared_state)
}

fn get_hash(password: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qIy074EXAsMI");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

async fn singup(
    State(state): State<SharedState>,
    Json(input_payload): Json<SingupRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if users.contains_key(&input_payload.username) {
        return (StatusCode::CONFLICT, "Username exists").into_response();
    }
    users.insert(
        input_payload.username.clone(),
        get_hash(&input_payload.password),
    );

    let response = SingupResponse {
        username: input_payload.username,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn delete_account(
    State(state): State<SharedState>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if !users.contains_key(&input_payload.username) {
        return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
    }

    let password_hash = get_hash(&input_payload.password);
    if users[&input_payload.username] != password_hash {
        return (StatusCode::FORBIDDEN, "Wrong password").into_response();
    }

    users.remove(&input_payload.username);
    let response = DeleteAccountResponse {
        username: input_payload.username,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Deserialize)]
struct SingupRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct SingupResponse {
    username: String,
}

#[derive(Deserialize)]
struct DeleteAccountRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct DeleteAccountResponse {
    username: String,
}
