use axum::{
    body::{self, Bytes},
    extract::{DefaultBodyLimit, Multipart, Query, Request, State},
    http::{header, request, response, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Form, Json, Router,
};
use chrono::Local;
use hex;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest;
use serde::{de, forward_to_deserialize_any, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    borrow::Borrow,
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    slice::RSplitN,
    str,
    sync::{Arc, RwLock},
};
use tokio::time::timeout;
use tokio::time::Duration;

pub async fn create_app() -> Router {
    Router::new()
        // .route("/", get(root_handler))
        .route("/signup", post(signup))
        // .route("/delete_account", delete(delete_account))
        .route("/login", post(login))
        // .route("/logout", post(logout))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
}

// async fn root_handler() -> Result<impl IntoResponse, impl IntoResponse> {
//     // TODO тут видимо надо выдавать флаттер приложение?
// }

#[derive(Serialize, Deserialize)]
struct SignupRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct DeleteAccountRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn send_requests_with_timeouts<InputJsonType: Serialize>(
    url: &str,
    input_payload: InputJsonType,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let durations = vec![
        Duration::from_millis(150),
        Duration::from_millis(300),
        Duration::from_millis(600),
        Duration::from_millis(1200),
    ];

    let client = reqwest::Client::new();

    for duration in durations.iter() {
        let timeout_result = timeout(*duration, client.post(url).json(&input_payload).send()).await;

        match timeout_result {
            Ok(auth_response_result) => match auth_response_result {
                Ok(response) => {
                    let resp_status = response.status();
                    let resp_headers = response.headers().clone();
                    let resp_body = response.bytes().await.unwrap_or_default();
                    return Ok((resp_status, resp_headers, resp_body).into_response());
                }
                Err(_) => {
                    return Err(
                        (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()
                    );
                }
            },
            Err(_) => {}
        };
    }

    Err((StatusCode::SERVICE_UNAVAILABLE, "Auth service unavailable").into_response())
}

async fn signup(
    Json(input_payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let url = "http://localhost:3000/signup";

    send_requests_with_timeouts(url, input_payload).await
}

// async fn delete_account(
//     Json(input_payload): Json<DeleteAccountRequest>,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     // TODO send to all other services:
//     // 0) respond to user immediately, all next requests do in separate threads
//     // 1) auth service (delete line with user from db there)
//     // 2) tracks service (delete all user's tracks)
//     // 3) playlists service (delete all user's playlists)
//     // 4) rates service (delete all user's rates from db)
//     // NOTE: don't change rates on tracks that this users rated.

//     Ok((StatusCode::NOT_IMPLEMENTED).into_response())
// }

async fn login(
    Json(input_payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let url = "http://localhost:3000/login";

    send_requests_with_timeouts(url, input_payload).await
}
