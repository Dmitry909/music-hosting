use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Local};
use hex;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::Read, str, sync::Arc, sync::RwLock};

#[derive(Default)]
struct UserData {
    password_hash: String,
    active_token: String,
}

#[derive(Default)]
struct AppState {
    users: HashMap<String, UserData>,
}

type SharedState = Arc<RwLock<AppState>>;

pub fn create_app() -> Router {
    let shared_state = SharedState::default();
    Router::new()
        .route("/singup", post(singup))
        .route("/delete_account", delete(delete_account))
        .route("/login", post(login))
        .with_state(shared_state)
}

fn get_hash(password: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qIy074EXAsMI");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
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
        UserData {
            password_hash: get_hash(&input_payload.password),
            active_token: String::new(),
        },
    );

    let response = SingupResponse {
        username: input_payload.username,
    };

    (StatusCode::CREATED, Json(response)).into_response()
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

async fn delete_account(
    State(state): State<SharedState>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if !users.contains_key(&input_payload.username) {
        return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
    }

    let password_hash = get_hash(&input_payload.password);
    if users[&input_payload.username].password_hash != password_hash {
        return (StatusCode::FORBIDDEN, "Wrong password").into_response();
    }

    users.remove(&input_payload.username);
    let response = DeleteAccountResponse {
        username: input_payload.username,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenData {
    username: String,
    expires: usize,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {}

async fn login(
    State(state): State<SharedState>,
    Json(input_payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if !users.contains_key(&input_payload.username) {
        return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
    }

    let password_hash = get_hash(&input_payload.password);
    if users[&input_payload.username].password_hash != password_hash {
        return (StatusCode::FORBIDDEN, "Wrong password").into_response();
    }

    let secret = b"my_secret_key_d47fjs&w3)wj";
    let encoding_header = Header::new(Algorithm::HS256);

    let expires = Local::now() + Duration::hours(24);
    let token_data = TokenData {
        username: input_payload.username.clone(), // TODO do not clone here
        expires: expires.timestamp() as usize,
    };
    let token = encode(
        &encoding_header,
        &token_data,
        &EncodingKey::from_secret(secret),
    )
    .expect("Failed to encode token");

    users.get_mut(&input_payload.username).unwrap().active_token = token.clone();

    let response = LoginResponse {};

    (StatusCode::OK, [("Authorization", token)], Json(response)).into_response()
}
