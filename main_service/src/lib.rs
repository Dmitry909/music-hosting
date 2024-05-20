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
use std::sync::Arc;
use std::{any::Any, str};
use tokio::time::timeout;
use tokio::time::Duration;

struct AppState {
    SIGNUP_EP: String,
    LOGIN_EP: String,
    LOGOUT_EP: String,
    DELETE_ACCOUNT_EP_AUTH: String,
    CHECK_TOKEN_EP: String,
    SEARCH_EP_AUTH: String,
    UPLOAD_TRACK_EP: String,
    DELETE_ACCOUNT_EP_TRACKS: String,
    DELETE_TRACK_EP: String,
    DOWNLOAD_TRACK_EP: String,
    SEARCH_EP_TRACKS: String,
    CREATE_PLAYLIST_EP: String,
    DELETE_PLAYLIST_EP: String,
    ADD_TO_PLAYLIST_EP: String,
    DELETE_FROM_PLAYLIST_EP: String,
    GET_PLAYLIST_EP: String,
    SEARCH_EP_PLAYLISTS: String,
    DELETE_ACCOUNT_EP_PLAYLISTS: String,
    PLAY_TRACK_EP: String,
    GET_NEXT_TRACK_EP: String,
}

#[macro_use]
extern crate lazy_static;

pub async fn create_app(AUTH_HOST: &str, TRACKS_HOST: &str, PLAYLISTS_HOST: &str, QUEUE_HOST: &str) -> Router {
    let shared_state = Arc::new(AppState {
        SIGNUP_EP: format!("{}/signup", AUTH_HOST),
        LOGIN_EP: format!("{}/login", AUTH_HOST),
        LOGOUT_EP: format!("{}/logout", AUTH_HOST),
        DELETE_ACCOUNT_EP_AUTH: format!("{}/delete_account", AUTH_HOST),
        CHECK_TOKEN_EP: format!("{}/check_token", AUTH_HOST),
        SEARCH_EP_AUTH: format!("{}/search", AUTH_HOST),
        UPLOAD_TRACK_EP: format!("{}/upload_track", TRACKS_HOST),
        DELETE_ACCOUNT_EP_TRACKS: format!("{}/delete_account", TRACKS_HOST),
        DELETE_TRACK_EP: format!("{}/delete_track", TRACKS_HOST),
        DOWNLOAD_TRACK_EP: format!("{}/download_track", TRACKS_HOST),
        SEARCH_EP_TRACKS: format!("{}/search", TRACKS_HOST),
        CREATE_PLAYLIST_EP: format!("{}/create_playlist", PLAYLISTS_HOST),
        DELETE_PLAYLIST_EP: format!("{}/delete_playlist", PLAYLISTS_HOST),
        ADD_TO_PLAYLIST_EP: format!("{}/add_to_playlist", PLAYLISTS_HOST),
        DELETE_FROM_PLAYLIST_EP: format!("{}/delete_from_playlist", PLAYLISTS_HOST),
        GET_PLAYLIST_EP: format!("{}/get_playlist", PLAYLISTS_HOST),
        SEARCH_EP_PLAYLISTS: format!("{}/search", PLAYLISTS_HOST),
        DELETE_ACCOUNT_EP_PLAYLISTS: format!("{}/delete_playlist", PLAYLISTS_HOST),
        PLAY_TRACK_EP: format!("{}/play_track", QUEUE_HOST),
        GET_NEXT_TRACK_EP: format!("{}/get_next_track", QUEUE_HOST),
    });

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
        // .route("/play_track", get(play_track))
        .route("/get_next_track", get(get_next_track))
        //
        .route("/search", get(search))
        //
        .route("/delete_account", delete(delete_account))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(shared_state)
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

async fn signup(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<SignupRequest>,
) -> Response {
    send_requests_with_timeouts(
        &state.SIGNUP_EP,
        &reqwest::Method::POST,
        {},
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<LoginRequest>,
) -> Response {
    send_requests_with_timeouts(
        &state.LOGIN_EP,
        &reqwest::Method::POST,
        {},
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await
}

async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    send_requests_with_timeouts(
        &state.LOGOUT_EP,
        &reqwest::Method::POST,
        {},
        headers,
        &EmptyRequest {},
        "Auth",
    )
    .await
}

async fn check_token(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    send_requests_with_timeouts(
        &state.CHECK_TOKEN_EP,
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

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> Response {
    // TODO send to all other services:
    // 0) respond to user immediately, all next requests do in separate threads
    // 1) auth service (delete line with user from db there)
    // 2) tracks service (delete all user's tracks)
    // 3) playlists service (delete all user's playlists)
    // 4) rates service (delete all user's rates from db)
    // NOTE: don't change rates on tracks that this users rated.

    // TODO сделать асинхронным

    let auth_resp = send_one_request(
        &state.DELETE_ACCOUNT_EP_AUTH,
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
        &state.DELETE_ACCOUNT_EP_TRACKS,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &input_payload,
        "Auth",
    )
    .await;

    return (StatusCode::OK).into_response();
}

async fn resolve_username_from_token(
    state: &Arc<AppState>,
    headers: HeaderMap,
) -> Result<String, Response> {
    let auth_resp = send_requests_with_timeouts_reqwest(
        &state.CHECK_TOKEN_EP,
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

async fn upload_track(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
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
        .post(state.UPLOAD_TRACK_EP.as_str())
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
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input_payload): Json<DeleteTrackRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
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
        &state.DELETE_TRACK_EP,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &middle_request,
        "Tracks",
    )
    .await
}

async fn download_track(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadTrackParams>,
) -> Response {
    send_requests_with_timeouts(
        &state.DOWNLOAD_TRACK_EP,
        &reqwest::Method::GET,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}

async fn create_playlist(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input_payload): Json<CreatePlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
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
        &state.CREATE_PLAYLIST_EP,
        &reqwest::Method::POST,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input_payload): Json<DeletePlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
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
        &state.DELETE_PLAYLIST_EP,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn add_to_playlist(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input_payload): Json<AddToPlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
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
        &state.ADD_TO_PLAYLIST_EP,
        &reqwest::Method::PUT,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn delete_from_playlist(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input_payload): Json<DeleteFromPlaylistRequest>,
) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
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
        &state.DELETE_FROM_PLAYLIST_EP,
        &reqwest::Method::DELETE,
        {},
        HeaderMap::new(),
        &middle_request,
        "Playlists",
    )
    .await
}

async fn get_playlist(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetPlaylistParams>,
) -> Response {
    send_requests_with_timeouts(
        &state.GET_PLAYLIST_EP,
        &reqwest::Method::GET,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Playlists",
    )
    .await
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Response {
    send_requests_with_timeouts(
        &state.SEARCH_EP_TRACKS,
        &reqwest::Method::GET,
        params,
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}

async fn get_next_track(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let resolve_result = resolve_username_from_token(&state, headers).await;
    let username = match resolve_result {
        Ok(username) => username,
        Err(response) => {
            return response;
        }
    };

    // TODO this request must be sent with username
    send_requests_with_timeouts(
        &state.GET_NEXT_TRACK_EP,
        &reqwest::Method::GET,
        {},
        HeaderMap::new(),
        &EmptyRequest {},
        "Tracks",
    )
    .await
}
