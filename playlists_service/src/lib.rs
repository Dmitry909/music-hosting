use axum::{
    body::{self, Bytes},
    extract::{DefaultBodyLimit, Multipart, Query, Request, State},
    http::{header, request, response, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Form, Json, Router,
};
use chrono::{Duration, Local};
use hex;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
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

use sqlx::{database, postgres::PgPoolOptions, query, query_as, Pool, Postgres};

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
    playlists_pool: Pool<Postgres>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaylistsModel {
    pub id: i64,
    pub owner_username: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaylistsModelOnlyUsername {
    pub owner_username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaylistsTracksModel {
    pub id: i64,
    pub playlist_id: i64,
    pub track_id: i64,
}

pub async fn create_app(playlists_db_url: &str, need_to_clear: bool) -> Router {
    let playlists_pool = create_pool(playlists_db_url).await;

    if need_to_clear {
        let _ = sqlx::query_as!(PlaylistsModel, "TRUNCATE TABLE playlists",)
            .execute(&playlists_pool)
            .await;
        let _ = sqlx::query_as!(PlaylistsTracksModel, "TRUNCATE TABLE playlists_tracks",)
            .execute(&playlists_pool)
            .await;
    }

    // sqlx::migrate!("./migrations").run(&pool);

    let shared_state = Arc::new(AppState { playlists_pool });
    Router::new()
        .route("/create_playlist", post(create_playlist))
        .route("/add_to_playlist", put(add_to_playlist))
        .route("/get_playlist", get(get_playlist))
        .route("/delete_from_playlist", delete(delete_from_playlist))
        .route("/delete_playlist", delete(delete_playlist))
        .route("/delete_account", delete(delete_account))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(shared_state)
}

#[derive(Serialize, Deserialize, Debug)]
struct CreatePlaylistRequest {
    username: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreatePlaylistResponse {
    id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddToPlaylistRequest {
    username: String,
    playlist_id: i64,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetPlaylistParams {
    playlist_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteFromPlaylistRequest {
    username: String,
    playlist_id: i64,
    track_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeletePlaylistRequest {
    username: String,
    playlist_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteAccountRequest {
    username: String,
}

async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<CreatePlaylistRequest>,
) -> Response {
    let query_result = sqlx::query_as!(
        PlaylistsModel,
        "INSERT INTO playlists (owner_username, name) VALUES ($1, $2) RETURNING *",
        input_payload.username,
        input_payload.name,
    )
    .fetch_one(&state.playlists_pool)
    .await;

    match query_result {
        Ok(query_result) => {
            let response = CreatePlaylistResponse {
                id: query_result.id,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn check_playlist_belongs_user(
    pool: &Pool<Postgres>,
    username: String,
    playlist_id: i64,
) -> Option<Response> {
    let query_result = sqlx::query_as!(
        PlaylistsModelOnlyUsername,
        "SELECT owner_username FROM playlists WHERE id=$1",
        playlist_id
    )
    .fetch_optional(pool)
    .await;

    match query_result {
        Ok(query_result) => match query_result {
            None => Some(
                (
                    StatusCode::NOT_FOUND,
                    format!("Playlist {} doesn't exist", playlist_id),
                )
                    .into_response(),
            ),
            Some(result) => {
                if result.owner_username == username {
                    return None;
                }
                return Some(
                    (
                        StatusCode::FORBIDDEN,
                        format!("You are not allowed to modify playlist {}", playlist_id),
                    )
                        .into_response(),
                );
            }
        },
        Err(_) => Some((StatusCode::INTERNAL_SERVER_ERROR).into_response()),
    }
}

async fn add_to_playlist(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<AddToPlaylistRequest>,
) -> Response {
    let belongs = check_playlist_belongs_user(
        &state.playlists_pool,
        input_payload.username,
        input_payload.playlist_id,
    )
    .await;

    match belongs {
        Some(err_response) => return err_response,
        None => {}
    };

    let query_result = sqlx::query_as!(
        PlaylistsTracksModel,
        "INSERT INTO playlists_tracks (playlist_id, track_id) VALUES ($1, $2)",
        input_payload.playlist_id,
        input_payload.track_id,
    )
    .execute(&state.playlists_pool)
    .await;

    match query_result {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn ensure_playlist_exists(pool: &Pool<Postgres>, playlist_id: i64) -> Option<Response> {
    let query_result = sqlx::query_as!(
        PlaylistsModel,
        "SELECT * FROM playlists WHERE id=$1",
        playlist_id,
    )
    .fetch_optional(pool)
    .await;

    match query_result {
        Ok(query_result) => match query_result {
            Some(_) => None,
            None => Some(
                (
                    StatusCode::NOT_FOUND,
                    format!("Playlist {} doesn't exist", playlist_id),
                )
                    .into_response(),
            ),
        },
        Err(_) => Some((StatusCode::INTERNAL_SERVER_ERROR).into_response()),
    }
}

async fn get_playlist(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetPlaylistParams>,
) -> Response {
    let ensure_result = ensure_playlist_exists(&state.playlists_pool, params.playlist_id).await;
    match ensure_result {
        Some(response) => return response,
        None => {}
    };

    let query_result = sqlx::query!(
        "SELECT track_id FROM playlists_tracks WHERE playlist_id=$1",
        params.playlist_id,
    )
    .fetch_all(&state.playlists_pool)
    .await;

    match query_result {
        Ok(vec_tracks_ids) => {
            let tracks: Vec<i64> = vec_tracks_ids
                .into_iter()
                .map(|record| match record.track_id {
                    Some(track_id) => track_id,
                    None => -1,
                })
                .collect();
            (StatusCode::OK, Json(tracks)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown database error").into_response(),
    }
}

async fn delete_from_playlist(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeleteFromPlaylistRequest>,
) -> Response {
    let belongs = check_playlist_belongs_user(
        &state.playlists_pool,
        input_payload.username,
        input_payload.playlist_id,
    )
    .await;

    match belongs {
        Some(err_response) => return err_response,
        None => {}
    };

    let query_result = sqlx::query!(
        "DELETE FROM playlists_tracks WHERE playlist_id=$1 AND track_id=$2 RETURNING *",
        input_payload.playlist_id,
        input_payload.track_id,
    )
    .fetch_optional(&state.playlists_pool)
    .await;

    match query_result {
        Ok(result) => match result {
            Some(_) => (StatusCode::OK).into_response(),
            None => (
                StatusCode::NOT_FOUND,
                "No such playlist or track in playlist",
            )
                .into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeletePlaylistRequest>,
) -> Response {
    let belongs = check_playlist_belongs_user(
        &state.playlists_pool,
        input_payload.username,
        input_payload.playlist_id,
    )
    .await;

    match belongs {
        Some(err_response) => return err_response,
        None => {}
    };

    let query_result = sqlx::query_as!(
        PlaylistsModel,
        "DELETE FROM playlists WHERE id=$1 RETURNING *",
        input_payload.playlist_id,
    )
    .fetch_optional(&state.playlists_pool)
    .await;

    match query_result {
        Ok(result) => match result {
            Some(_) => (StatusCode::OK).into_response(),
            None => (StatusCode::NOT_FOUND, "No such playlist").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> Response {
    let query_result = sqlx::query_as!(
        PlaylistsModel,
        "DELETE FROM playlists WHERE owner_username=$1",
        input_payload.username,
    )
    .fetch_optional(&state.playlists_pool)
    .await;

    match query_result {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}
