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

#[macro_use]
extern crate lazy_static;

const AUTH_HOST: &str = "http://localhost:3000";
const TRACKS_HOST: &str = "http://localhost:3001";
const PLAYLISTS_HOST: &str = "http://localhost:3003";

// EP = ENDPOINT
lazy_static! {
    pub static ref SIGNUP_EP: String = format!("{}/signup", AUTH_HOST);
    pub static ref LOGIN_EP: String = format!("{}/login", AUTH_HOST);
    pub static ref LOGOUT_EP: String = format!("{}/logout", AUTH_HOST);
    pub static ref DELETE_ACCOUNT_EP_AUTH: String = format!("{}/delete_account", AUTH_HOST);
    pub static ref CHECK_TOKEN_EP: String = format!("{}/check_token", AUTH_HOST);
    pub static ref SEARCH_EP_AUTH: String = format!("{}/search", AUTH_HOST);
    //
    pub static ref UPLOAD_TRACK_EP: String = format!("{}/upload_track", TRACKS_HOST);
    pub static ref DELETE_ACCOUNT_EP_TRACKS: String = format!("{}/delete_account", TRACKS_HOST);
    pub static ref DELETE_TRACK_EP: String = format!("{}/delete_track", TRACKS_HOST);
    pub static ref DOWNLOAD_TRACK_EP: String = format!("{}/download_track", TRACKS_HOST);
    pub static ref SEARCH_EP_TRACKS: String = format!("{}/search", TRACKS_HOST);
    //
    pub static ref CREATE_PLAYLIST_EP: String = format!("{}/create_playlist", PLAYLISTS_HOST);
    pub static ref DELETE_PLAYLIST_EP: String = format!("{}/delete_playlist", PLAYLISTS_HOST);
    pub static ref ADD_TO_PLAYLIST_EP: String = format!("{}/add_to_playlist", PLAYLISTS_HOST);
    pub static ref DELETE_FROM_PLAYLIST_EP: String = format!("{}/delete_from_playlist", PLAYLISTS_HOST);
    pub static ref GET_PLAYLIST_EP: String = format!("{}/get_playlist", PLAYLISTS_HOST);
    pub static ref SEARCH_EP_PLAYLISTS: String = format!("{}/search", PLAYLISTS_HOST);
    pub static ref DELETE_ACCOUNT_EP_PLAYLISTS: String = format!("{}/delete_playlist", PLAYLISTS_HOST);
}

pub async fn create_app() -> Router {
    Router::new()
        // .route("/", get(root_handler))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/check_token", get(check_token))
        //
        .route("/upload_track", post(upload_track))
        .route("/delete_track", delete(delete_track))
        .route("/download_track", get(download_track))
        //
        .route("/create_playlist", post(create_playlist))
        .route("/delete_playlist", delete(delete_playlist))
        .route("/add_to_playlist", put(add_to_playlist))
        .route("/delete_from_playlist", delete(delete_from_playlist))
        .route("/get_playlist", get(get_playlist))
        //
        .route("/search", get(search))
        //
        .route("/delete_account", delete(delete_account))
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

#[derive(Serialize, Deserialize, Debug)]
struct CreatePlaylistRequest {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreatePlaylistMiddleRequest {
    username: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeletePlaylistRequest {
    playlist_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeletePlaylistMiddleRequest {
    username: String,
    playlist_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddToPlaylistRequest {
    playlist_id: i64,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddToPlaylistMiddleRequest {
    username: String,
    playlist_id: i64,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteFromPlaylistRequest {
    playlist_id: i64,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteFromPlaylistMiddleRequest {
    username: String,
    playlist_id: i64,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetPlaylistParams {
    playlist_id: i64,
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

async fn check_token(headers: HeaderMap) -> Response {
    send_requests_with_timeouts(
        &CHECK_TOKEN_EP,
        &reqwest::Method::GET,
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

async fn resolve_username_from_token(headers: HeaderMap) -> Result<String, Response> {
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
            return Err(staus_code.into_response());
        }
    };
    let body: Bytes = match auth_resp.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "auth_resp.bytes() failed",
            )
                .into_response());
        }
    };
    let check_token_response: Result<CheckTokenResponse, _> = serde_json::from_slice(&body);
    let check_token_response = match check_token_response {
        Ok(resp) => resp,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "check_token response is in incorrect format",
            )
                .into_response());
        }
    };

    let username = check_token_response.username;
    Ok(username)
}

async fn upload_track(headers: HeaderMap, mut multipart: Multipart) -> Response {
    let resolve_result = resolve_username_from_token(headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

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
    let resolve_result = resolve_username_from_token(headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

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

async fn create_playlist(
    headers: HeaderMap,
    Json(input_payload): Json<CreatePlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

    let middle_request = CreatePlaylistMiddleRequest {
        username,
        name: input_payload.name,
    };

    send_requests_with_timeouts(
        &CREATE_PLAYLIST_EP,
        &reqwest::Method::POST,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn delete_playlist(
    headers: HeaderMap,
    Json(input_payload): Json<DeletePlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

    let middle_request = DeletePlaylistMiddleRequest {
        username,
        playlist_id: input_payload.playlist_id,
    };

    send_requests_with_timeouts(
        &DELETE_PLAYLIST_EP,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn add_to_playlist(
    headers: HeaderMap,
    Json(input_payload): Json<AddToPlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

    // TODO: нужно еще проверять, правда ли такой трек вообще существует.

    let middle_request = AddToPlaylistMiddleRequest {
        username,
        playlist_id: input_payload.playlist_id,
        track_id: input_payload.track_id,
    };

    send_requests_with_timeouts(
        &ADD_TO_PLAYLIST_EP,
        &reqwest::Method::PUT,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn delete_from_playlist(
    headers: HeaderMap,
    Json(input_payload): Json<DeleteFromPlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

    let middle_request = DeleteFromPlaylistMiddleRequest {
        username,
        playlist_id: input_payload.playlist_id,
        track_id: input_payload.track_id,
    };

    send_requests_with_timeouts(
        &DELETE_FROM_PLAYLIST_EP,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn get_playlist(Query(params): Query<GetPlaylistParams>) -> Response {
    send_requests_with_timeouts(
        &GET_PLAYLIST_EP,
        &reqwest::Method::GET,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Playlists",
    )
    .await
}

async fn search(Query(params): Query<SearchParams>) -> Response {
    send_requests_with_timeouts(
        &SEARCH_EP_TRACKS,
        &reqwest::Method::POST,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}
