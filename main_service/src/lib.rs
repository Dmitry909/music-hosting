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
struct SignupResponse {
    username: String,
}

async fn signup(
    Json(input_payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let client = reqwest::Client::new();
    let url = "http://localhost:3000/signup";

    let durations = vec![
        Duration::from_millis(150),
        Duration::from_millis(300),
        Duration::from_millis(600),
        Duration::from_millis(1200),
    ];

    for duration in durations.iter() {
        let timeout_result = timeout(*duration, client.post(url).json(&input_payload).send()).await;

        match timeout_result {
            Ok(auth_response_result) => match auth_response_result {
                Ok(response) => {
                    let resp_status = response.status();
                    let resp_body = response.bytes().await.unwrap();
                    return Ok((resp_status, resp_body).into_response());
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

    return Err((StatusCode::SERVICE_UNAVAILABLE, "Auth service unavailable").into_response());
}
