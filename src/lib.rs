use axum::{
    body::Bytes,
    extract::State,
    http::{header, response, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Local};
use hex;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{forward_to_deserialize_any, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    borrow::Borrow,
    collections::HashMap,
    io::Read,
    str,
    sync::{Arc, RwLock},
};

#[derive(Default)]
struct UserData {
    password_hash: String,
    active_token: String,
}

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

struct AppState {
    pool: Pool<Postgres>,
}

pub fn create_app(pool: Pool<Postgres>) -> Router {
    let shared_state = Arc::new(AppState { pool });
    Router::new()
        .route("/signup", post(signup))
        // .route("/delete_account", delete(delete_account))
        // .route("/login", post(login))
        // .route("/logout", post(logout))
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
struct SignupRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct SignupResponse {
    username: String,
}

#[derive(Debug)]
pub struct UsersModel {
    pub username: String,
    pub password_hash: String,
    pub active_token: String,
}

async fn signup(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        UsersModel,
        "INSERT INTO users (username,password_hash,active_token) VALUES ($1, $2, $3) RETURNING *",
        input_payload.username.to_string(),
        "qwe",
        "rty",
    )
    .fetch_all(&state.pool)
    .await;

    // let users = &mut state.write().unwrap().users;
    // if users.contains_key(&input_payload.username) {
    //     return (StatusCode::CONFLICT, "Username exists").into_response();
    // }
    // users.insert(
    //     input_payload.username.clone(),
    //     UserData {
    //         password_hash: get_hash(&input_payload.password),
    //         active_token: String::new(),
    //     },
    // );

    let response = SignupResponse {
        username: input_payload.username,
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

// #[derive(Deserialize)]
// struct DeleteAccountRequest {
//     username: String,
//     password: String,
// }

// #[derive(Serialize, Deserialize)]
// struct DeleteAccountResponse {
//     username: String,
// }

// async fn delete_account(
//     State(state): State<SharedState>,
//     Json(input_payload): Json<DeleteAccountRequest>,
// ) -> impl IntoResponse {
//     let users = &mut state.write().unwrap().users;
//     if !users.contains_key(&input_payload.username) {
//         return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
//     }

//     let password_hash = get_hash(&input_payload.password);
//     if users[&input_payload.username].password_hash != password_hash {
//         return (StatusCode::FORBIDDEN, "Wrong password").into_response();
//     }

//     users.remove(&input_payload.username);
//     let response = DeleteAccountResponse {
//         username: input_payload.username,
//     };

//     (StatusCode::OK, Json(response)).into_response()
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct TokenData {
//     username: String,
//     exp: usize,
// }

// #[derive(Deserialize)]
// struct LoginRequest {
//     username: String,
//     password: String,
// }

// fn generate_token(username: &String) -> String {
//     let secret = b"my_secret_key_d47fjs&w3)wj";
//     let token_data = TokenData {
//         username: username.clone(),
//         exp: (Local::now() + Duration::hours(24)).timestamp() as usize,
//     };
//     let encoding_key = EncodingKey::from_secret(secret);
//     encode(&Header::default(), &token_data, &encoding_key).unwrap()
// }

// async fn login(
//     State(state): State<SharedState>,
//     Json(input_payload): Json<LoginRequest>,
// ) -> impl IntoResponse {
//     let users = &mut state.write().unwrap().users;
//     if !users.contains_key(&input_payload.username) {
//         return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
//     }

//     let password_hash = get_hash(&input_payload.password);
//     if users[&input_payload.username].password_hash != password_hash {
//         return (StatusCode::FORBIDDEN, "Wrong password").into_response();
//     }

//     let token = generate_token(&input_payload.username);

//     users.get_mut(&input_payload.username).unwrap().active_token = token.clone();

//     (StatusCode::OK, [("Authorization", token)]).into_response()
// }

// fn decode_token(
//     token: &str,
// ) -> Result<jsonwebtoken::TokenData<TokenData>, jsonwebtoken::errors::Error> {
//     let secret = b"my_secret_key_d47fjs&w3)wj";
//     return decode::<TokenData>(
//         token,
//         &DecodingKey::from_secret(secret),
//         &Validation::new(Algorithm::HS256),
//     );
// }

// async fn logout(State(state): State<SharedState>, headers: HeaderMap) -> impl IntoResponse {
//     if !headers.contains_key("Authorization") {
//         return (StatusCode::UNAUTHORIZED, "Token is missing").into_response();
//     }
//     let token = headers["Authorization"].to_str().unwrap();

//     let decoded_token = match decode_token(token) {
//         Ok(c) => c.claims,
//         Err(err) => {
//             println!("Error: {:?}", err);
//             return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
//         }
//     };

//     let users = &mut state.write().unwrap().users;
//     if !users.contains_key(&decoded_token.username) {
//         return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
//     }

//     users.get_mut(&decoded_token.username).unwrap().active_token = String::new();

//     (StatusCode::OK).into_response()
// }
