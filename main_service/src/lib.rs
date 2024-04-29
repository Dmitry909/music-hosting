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
use std::str;
use tokio::time::timeout;
use tokio::time::Duration;

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
    pub static ref DOWNLOAD_TRACK_EP: String = format!("{}/download_track", TRACKS_HOST);
    pub static ref SEARCH_EP: String = format!("{}/search", TRACKS_HOST);
}

pub async fn create_app() -> Router {
    Router::new()
        // .route("/", get(root_handler))
        .route("/signup", post(signup))
        .route("/delete_account", delete(delete_account))
        .route("/login", post(login))
        .route("/logout", post(logout))
        //
        .route("/upload_track", post(upload_track))
        .route("/delete_track", delete(delete_track))
        .route("/download_track", get(download_track))
        .route("/search", get(search))
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
struct CheckTokenResponse {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct EmptyRequest {}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteTrackRequest {
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteTrackMiddleRequest {
    username: String,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadTrackMiddleRequest {
    username: String,
    track_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadTrackParams {
    id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchParams {
    query: String,
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

async fn send_requests_with_timeouts_reqwest<ParamsType: Serialize, InputJsonType: Serialize>(
    url: &str,
    method: &reqwest::Method,
    params: ParamsType,
    headers: HeaderMap,
    input_payload: &InputJsonType,
) -> Result<reqwest::Response, StatusCode> {
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
                    return Ok(response);
                }
                Err(_) => {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
            Err(_) => {}
        };
    }

    Err(StatusCode::SERVICE_UNAVAILABLE)
}

async fn signup(Json(input_payload): Json<SignupRequest>) -> Response {
    send_requests_with_timeouts(
        &SIGNUP_EP,
        &reqwest::Method::POST,
        {},
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await
}

async fn login(Json(input_payload): Json<LoginRequest>) -> Response {
    send_requests_with_timeouts(
        &LOGIN_EP,
        &reqwest::Method::POST,
        {},
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await
}

async fn logout(headers: HeaderMap) -> Response {
    send_requests_with_timeouts(
        &LOGOUT_EP,
        &reqwest::Method::POST,
        {},
        headers,
        &EmptyRequest {},
        "Auth",
    )
    .await
}

async fn send_one_request<InputJsonType: Serialize>(
    url: &str,
    method: &reqwest::Method,
    headers: HeaderMap,
    input_payload: &InputJsonType,
    service_name: &str,
) -> Result<reqwest::Response, axum::response::Response> {
    let client = reqwest::Client::new();

    let timeout_result = timeout(
        Duration::from_secs(10),
        client
            .request(method.clone(), url)
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

async fn delete_account(Json(input_payload): Json<DeleteAccountRequest>) -> Response {
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
        &reqwest::Method::DELETE,
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
                return (resp_status, resp_headers, resp_body).into_response();
            }
        }
        Err(err) => {
            return err;
        }
    };

    // TODO этот запрос слать уже в другом треде, пользователю ответить сразу
    send_requests_with_timeouts(
        &DELETE_ACCOUNT_EP_TRACKS,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await;

    return (StatusCode::OK).into_response();
}

async fn upload_track(headers: HeaderMap, mut multipart: Multipart) -> Response {
    let auth_resp = send_requests_with_timeouts_reqwest(
        &CHECK_TOKEN_EP,
        &reqwest::Method::GET,
        {},
        headers,
        &EmptyRequest {},
    )
    .await;
    let auth_resp = match auth_resp {
        Ok(resp) => resp,
        Err(staus_code) => {
            return staus_code.into_response();
        }
    };
    let body = match auth_resp.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };
    let check_token_response: Result<CheckTokenResponse, _> = serde_json::from_slice(&body);
    let check_token_response = match check_token_response {
        Ok(resp) => resp,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "check_token response is in incorrect format",
            )
                .into_response();
        }
    };

    let username = check_token_response.username;

    ////

    let mut form = reqwest::multipart::Form::new();

    let mut track_name: String = String::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        if name == "track_name" {
            track_name = String::from_utf8(data.to_vec()).unwrap();
            continue;
        }
        let part = reqwest::multipart::Part::stream(data);
        form = form.part(name, part);
    }
    let upload_track_middle_request = UploadTrackMiddleRequest {
        username,
        track_name,
    };
    let part = reqwest::multipart::Part::stream(
        serde_json::to_string(&upload_track_middle_request).unwrap(),
    );
    form = form.part("json", part);

    ////

    let client = reqwest::Client::new();
    let response = client
        .post(UPLOAD_TRACK_EP.as_str())
        .multipart(form)
        .send()
        .await;

    match response {
        Ok(response) => {
            let status = response.status();
            let body = response.bytes().await.unwrap();
            (status, body).into_response()
        }
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE).into_response(),
    }
}

async fn delete_track(
    headers: HeaderMap,
    Json(input_payload): Json<DeleteTrackRequest>,
) -> Response {
    let auth_resp = send_requests_with_timeouts_reqwest(
        &CHECK_TOKEN_EP,
        &reqwest::Method::GET,
        {},
        headers,
        &EmptyRequest {},
    )
    .await;
    let auth_resp = match auth_resp {
        Ok(resp) => resp,
        Err(staus_code) => {
            return staus_code.into_response();
        }
    };
    let body = match auth_resp.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };
    let check_token_response: CheckTokenResponse = serde_json::from_slice(&body).unwrap();
    let username = check_token_response.username;

    ////
    let middle_request = DeleteTrackMiddleRequest {
        username,
        track_id: input_payload.track_id,
    };

    send_requests_with_timeouts(
        &DELETE_TRACK_EP,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &middle_request,
        "Tracks",
    )
    .await
}

async fn download_track(Query(params): Query<DownloadTrackParams>) -> Response {
    send_requests_with_timeouts(
        &DOWNLOAD_TRACK_EP,
        &reqwest::Method::GET,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}

async fn search(Query(params): Query<SearchParams>) -> Response {
    send_requests_with_timeouts(
        &SEARCH_EP,
        &reqwest::Method::POST,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}
