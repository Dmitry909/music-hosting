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
use serde::{
    de::{self, value},
    forward_to_deserialize_any, Deserialize, Serialize,
};
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
use tracing::instrument::WithSubscriber;

#[macro_use]
extern crate lazy_static;

const AUTH_HOST: &str = "http://localhost:3000";
const TRACKS_HOST: &str = "http://localhost:3001";

// EP = ENDPOINT
lazy_static! {
    pub static ref SIGNUP_EP: String = format!("{}/signup", AUTH_HOST);
    pub static ref LOGIN_EP: String = format!("{}/login", AUTH_HOST);
    pub static ref LOGOUT_EP: String = format!("{}/logout", AUTH_HOST);
    pub static ref DELETE_ACCOUNT_EP_AUTH: String = format!("{}/delete_account", AUTH_HOST);
    pub static ref CHECK_TOKEN_EP: String = format!("{}/check_token", AUTH_HOST);
    //
    pub static ref UPLOAD_TRACK_EP: String = format!("{}/upload_track", TRACKS_HOST);
    pub static ref DELETE_ACCOUNT_EP_TRACKS: String = format!("{}/delete_account", TRACKS_HOST);
    pub static ref DELETE_TRACK_EP: String = format!("{}/delete_track", TRACKS_HOST);
    pub static ref SEARCH_EP: String = format!("{}/search", TRACKS_HOST);
}

pub async fn create_app() -> Router {
    Router::new()
        // .route("/", get(root_handler))
        .route("/signup", post(signup))
        .route("/delete_account", delete(delete_account))
        .route("/login", post(login))
        .route("/logout", post(logout))
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

#[derive(Serialize, Deserialize)]
struct EmptyRequest {}

async fn send_requests_with_timeouts<InputJsonType: Serialize>(
    url: &str,
    headers: HeaderMap,
    input_payload: &InputJsonType,
    service_name: &str,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let durations = vec![
        Duration::from_millis(150),
        Duration::from_millis(300),
        Duration::from_millis(600),
        Duration::from_millis(1200),
    ];

    let client = reqwest::Client::new();

    for duration in durations.iter() {
        let timeout_result = timeout(
            *duration,
            client
                .post(url)
                .json(input_payload)
                .headers(headers.clone())
                .send(),
        )
        .await;

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

    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        format!("{} service unavailable", service_name),
    )
        .into_response())
}

async fn signup(
    Json(input_payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // TODO signup -- не идемпотентная операция, нельзя 2 раза делать запрос.
    // Вернее, ничего не разломается, но пользователь может получить ответ что такой пользователь уже есть.
    // В принципе это не очень плохо, можно пока забить.
    send_requests_with_timeouts(&SIGNUP_EP, HeaderMap::new(), &input_payload, "Auth").await
}

async fn login(
    Json(input_payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    send_requests_with_timeouts(&LOGIN_EP, HeaderMap::new(), &input_payload, "Auth").await
}

async fn logout(headers: HeaderMap) -> Result<impl IntoResponse, impl IntoResponse> {
    send_requests_with_timeouts(&LOGOUT_EP, headers, &EmptyRequest {}, "Auth").await
}

async fn send_one_request<InputJsonType: Serialize>(
    url: &str,
    headers: HeaderMap,
    input_payload: &InputJsonType,
    service_name: &str,
) -> Result<reqwest::Response, impl IntoResponse> {
    let client = reqwest::Client::new();

    let timeout_result = timeout(
        Duration::from_secs(10),
        client
            .post(url)
            .json(input_payload)
            .headers(headers.clone())
            .send(),
    )
    .await;

    match timeout_result {
        Ok(auth_response_result) => match auth_response_result {
            Ok(response) => Ok(response),
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()),
        },
        Err(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{} service unavailable", service_name),
        )
            .into_response()),
    }
}

async fn delete_account(
    Json(input_payload): Json<DeleteAccountRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // TODO send to all other services:
    // 0) respond to user immediately, all next requests do in separate threads
    // 1) auth service (delete line with user from db there)
    // 2) tracks service (delete all user's tracks)
    // 3) playlists service (delete all user's playlists)
    // 4) rates service (delete all user's rates from db)
    // NOTE: don't change rates on tracks that this users rated.

    // TODO сделать асинхронным

    let auth_resp = send_one_request(
        &DELETE_ACCOUNT_EP_AUTH,
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await;

    match auth_resp {
        Ok(response) => {
            if !response.status().is_success() {
                let resp_status = response.status();
                let resp_headers = response.headers().clone();
                let resp_body = response.bytes().await.unwrap_or_default();
                return Ok((resp_status, resp_headers, resp_body).into_response());
            }
        }
        Err(err) => {
            return Err(err);
        }
    };

    let tracks_resp = send_one_request(
        &DELETE_ACCOUNT_EP_TRACKS,
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await;

    match tracks_resp {
        Ok(response) => {
            let resp_status = response.status();
            let resp_headers = response.headers().clone();
            let resp_body = response.bytes().await.unwrap_or_default();
            Ok((resp_status, resp_headers, resp_body).into_response())
        }
        Err(err) => Err(err),
    }
}
