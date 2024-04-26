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
use serde::{de, forward_to_deserialize_any, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::thread;
use std::{
    borrow::Borrow,
    collections::HashMap,
    io::Read,
    str,
    sync::{Arc, RwLock},
};

use sqlx::{database, postgres::PgPoolOptions, Pool, Postgres};

pub async fn create_pool(database_url: &str) -> Pool<Postgres> {
    match PgPoolOptions::new().connect(&database_url).await {
        Ok(pool) => {
            return pool;
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };
}

struct AppState {
    pool: Pool<Postgres>,
}

pub async fn create_app(users_db_url: &str, need_to_clear: bool) -> Router {
    let pool = create_pool(users_db_url).await;

    if need_to_clear {
        let _ = sqlx::query_as!(UsersModel, "TRUNCATE TABLE users",)
            .execute(&pool)
            .await;
    }

    // sqlx::migrate!("./migrations").run(&pool);

    let shared_state = Arc::new(AppState { pool });
    Router::new()
        .route("/signup", post(signup))
        .route("/delete_account", delete(delete_account))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/check_token", get(check_token))
        .with_state(shared_state)
}

fn get_hash(password: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qIy074EXAsMI");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[derive(Debug)]
pub struct UsersModel {
    pub username: String,
    pub password_hash: String,
    pub active_token: String,
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

#[derive(Deserialize)]
struct DeleteAccountRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct DeleteAccountResponse {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct CheckTokenResponse {
    username: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenData {
    username: String,
    exp: usize,
}

fn generate_token(username: &String) -> String {
    let secret = b"my_secret_key_d47fjs&w3)wj";
    let token_data = TokenData {
        username: username.clone(),
        exp: (Local::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let encoding_key = EncodingKey::from_secret(secret);
    encode(&Header::default(), &token_data, &encoding_key).unwrap()
}

fn decode_token(
    token: &str,
) -> Result<jsonwebtoken::TokenData<TokenData>, jsonwebtoken::errors::Error> {
    let secret = b"my_secret_key_d47fjs&w3)wj";
    return decode::<TokenData>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    );
}

fn exp_expired(exp: usize) -> bool {
    exp < (Local::now().timestamp() as usize)
}

async fn signup(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let query_result = sqlx::query_as!(
        UsersModel,
        "INSERT INTO users VALUES ($1, $2, $3)",
        input_payload.username,
        get_hash(&input_payload.password),
        String::new(),
    )
    .execute(&state.pool)
    .await;

    match query_result {
        Ok(_) => {
            let response = SignupResponse {
                username: input_payload.username,
            };

            return Ok((StatusCode::CREATED, Json(response)).into_response());
        }
        Err(_) => {
            return Err((StatusCode::CONFLICT, "Username exists").into_response());
        }
    };
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let query_result = sqlx::query_as!(
        UsersModel,
        "DELETE FROM users WHERE username=$1 AND password_hash=$2 RETURNING *",
        input_payload.username,
        get_hash(&input_payload.password),
    )
    .fetch_optional(&state.pool)
    .await;

    match query_result {
        Ok(user_optional) => match user_optional {
            Some(_) => {
                let response = DeleteAccountResponse {
                    username: input_payload.username,
                };

                return Ok((StatusCode::OK, Json(response)).into_response());
            }
            None => {
                return Ok((
                    StatusCode::NOT_FOUND,
                    "Username doesn't exist or password is wrong",
                )
                    .into_response())
            }
        },
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response());
        }
    };
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let token = generate_token(&input_payload.username);

    let query_result = sqlx::query_as!(
        UsersModel,
        "UPDATE users SET active_token=$3 WHERE username=$1 AND password_hash=$2 RETURNING *",
        input_payload.username,
        get_hash(&input_payload.password),
        &token,
    )
    .fetch_optional(&state.pool)
    .await;

    match query_result {
        Ok(user_optional) => match user_optional {
            Some(_) => {
                return Ok((StatusCode::OK, [("Authorization", token)]).into_response());
            }
            None => {
                return Ok((
                    StatusCode::NOT_FOUND,
                    "Username doesn't exist or password is wrong",
                )
                    .into_response())
            }
        },
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response());
        }
    };
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if !headers.contains_key("Authorization") {
        return Ok((StatusCode::UNAUTHORIZED, "Token is missing").into_response());
    }
    let token = headers["Authorization"].to_str().unwrap();

    let decoded_token = match decode_token(token) {
        Ok(c) => c.claims,
        Err(_) => {
            return Ok((StatusCode::UNAUTHORIZED, "Invalid token").into_response());
        }
    };

    let query_result = sqlx::query_as!(
        UsersModel,
        "UPDATE users SET active_token='' WHERE username=$1 RETURNING *",
        &decoded_token.username,
    )
    .fetch_optional(&state.pool)
    .await;

    match query_result {
        Ok(user_optional) => match user_optional {
            Some(_) => Ok((StatusCode::OK).into_response()),
            None => Ok((StatusCode::NOT_FOUND, "Username doesn't exist").into_response()),
        },
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()),
    }
}

async fn check_token(headers: HeaderMap) -> impl IntoResponse {
    if !headers.contains_key("Authorization") {
        return (StatusCode::UNAUTHORIZED, "Token is missing").into_response();
    }
    let token = headers["Authorization"].to_str().unwrap();

    let decoded_token = match decode_token(token) {
        Ok(c) => c.claims,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
        }
    };

    if exp_expired(decoded_token.exp) {
        return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
    }

    let response = CheckTokenResponse {
        username: decoded_token.username,
    };
    (StatusCode::OK, Json(response)).into_response()
}
