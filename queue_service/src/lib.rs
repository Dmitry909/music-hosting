use axum::{
    body::{self, Bytes},
    extract::{DefaultBodyLimit, Multipart, Query, Request, State},
    http::{header, request, response, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest;
use serde::{Deserialize, Serialize};
use std::{any::Any, str};
use tokio::time::timeout;
use tokio::time::Duration;
use std::sync::Arc;
use lazy_static::lazy_static;

struct AppState {
    // TODO store map<user, queue>
}

pub async fn create_app() -> Router {
    let shared_state = Arc::new(AppState {});
    Router::new()
        .route("/get_next_track", get(get_next_track))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(shared_state)
}

const TRACKS_HOST: &str = "http://localhost:3002";

lazy_static! {
    pub static ref GET_RANDOM_TRACK_ID_EP: String = format!("{}/get_random_track_id", TRACKS_HOST);
}

async fn send_requests_with_timeouts<ParamsType: Serialize, InputJsonType: Serialize>(
    url: &str,
    method: &reqwest::Method,
    params: ParamsType,
    headers: HeaderMap,
    input_payload: &InputJsonType,
    service_name: &str,
) -> Response {
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
                .request(method.clone(), url)
                .query(&params)
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
                    return (resp_status, resp_headers, resp_body).into_response();
                }
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response();
                }
            },
            Err(_) => {}
        };
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("{} service unavailable", service_name),
    )
        .into_response()
}

#[derive(Serialize, Deserialize)]
struct EmptyRequest {}

async fn get_next_track(
    State(state): State<Arc<AppState>>,
) -> Response {
    send_requests_with_timeouts(
        &GET_RANDOM_TRACK_ID_EP,
        &reqwest::Method::GET,
        {},
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}
