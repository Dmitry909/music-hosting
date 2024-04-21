use axum::{
    body::{self, Bytes},
    extract::{DefaultBodyLimit, Multipart, Query, Request, State},
    http::{header, request, response, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
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

use sqlx::{database, postgres::PgPoolOptions, query, Pool, Postgres};

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
    tracks_pool: Pool<Postgres>,
}

#[derive(Debug)]
pub struct TracksModel {
    pub id: i64,
    pub author_username: String,
    pub name: String,
    pub cnt_rates: i64,
    pub sum_rates: i64,
}

#[derive(Debug)]
pub struct TracksOnlyIdModel {
    pub id: i64,
}

pub async fn create_app(tracks_db_url: &str, need_to_clear: bool) -> Router {
    let tracks_pool = create_pool(tracks_db_url).await;

    if need_to_clear {
        let _ = sqlx::query_as!(TracksModel, "TRUNCATE TABLE tracks",)
            .execute(&tracks_pool)
            .await;
        match fs::remove_dir_all("./tracks") {
            Ok(_) => {}
            Err(_) => {}
        };
    }

    // sqlx::migrate!("./migrations").run(&pool);

    let shared_state = Arc::new(AppState { tracks_pool });
    Router::new()
        .route("/delete_account", delete(delete_account))
        .route("/upload_track", post(upload_track))
        .route("/delete_track", delete(delete_track))
        .route("/download_track", get(download_track))
        .route("/search", get(search))
        // .route("/comment_track", post(comment_track))
        // .route("/delete_comment", delete(delete_comment))
        // .route("/get_comments", get(get_comments))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(shared_state)
}

#[derive(Serialize, Deserialize)]
struct DeleteAccountRequest {
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadTrackRequest {
    username: String,
    track_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadTrackResponse {
    id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeleteTrackRequest {
    username: String,
    track_id: i64,
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
struct SearchResponseItem {
    id: i64,
    author_username: String,
    track_name: String,
    rating: f64,
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let query_result = sqlx::query_as!(
        TracksModel,
        "DELETE FROM tracks WHERE author_username=$1 RETURNING *",
        input_payload.username,
    )
    .fetch_all(&state.tracks_pool)
    .await;

    match query_result {
        Ok(tracks_vec) => {
            for track in tracks_vec.iter() {
                let filename: String = format!("tracks/{}.mp3", track.id);
                match fs::remove_file(filename) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Unknown filesystem error",
                        )
                            .into_response());
                    }
                };
            }
            Ok((StatusCode::OK).into_response())
        }
        Err(_) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown database error").into_response())
        }
    }
}

async fn upload_track(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut file: Option<Bytes> = None;
    let mut request: Option<UploadTrackRequest> = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
        if field_name == "file" {
            file = Some(field.bytes().await.unwrap());
        } else if field_name == "json" {
            let data = field.bytes().await.unwrap();
            let result: Result<UploadTrackRequest, serde_json::Error> =
                serde_json::from_slice(&data);
            request = match result {
                Ok(upload_track_request) => Some(upload_track_request),
                Err(_) => {
                    return Ok((StatusCode::BAD_REQUEST, "Incorrect input json").into_response());
                }
            };
        }
    }

    let file_content = match file {
        Some(value) => value,
        None => {
            return Err((StatusCode::BAD_REQUEST, "File wasn't provided").into_response());
        }
    };
    let request = match request {
        Some(value) => value,
        None => {
            return Ok((StatusCode::BAD_REQUEST, "Json wasn't provided").into_response());
        }
    };

    let query_result = sqlx::query_as!(
        TracksModel,
        "INSERT INTO tracks (author_username, name, cnt_rates, sum_rates) VALUES ($1, $2, $3, $4) RETURNING *",
        request.username,
        request.track_name,
        0,
        0,
    )
    .fetch_one(&state.tracks_pool)
    .await;

    match query_result {
        Ok(new_line) => {
            let mut file_path = "./tracks/".to_string();
            file_path += &new_line.id.to_string();
            file_path += ".mp3";
            let mut file = match File::create(file_path) {
                Ok(file) => file,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unknown filesystem error",
                    )
                        .into_response());
                }
            };
            match file.write_all(&file_content) {
                Ok(_) => {}
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unknown filesystem error",
                    )
                        .into_response());
                }
            };

            let resp = UploadTrackResponse { id: new_line.id };
            Ok((StatusCode::CREATED, Json(resp)).into_response())
        }
        Err(_) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown database error").into_response())
        }
    }
}

async fn delete_track(
    State(state): State<Arc<AppState>>,
    Json(input_payload): Json<DeleteTrackRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let query_result = sqlx::query_as!(
        TracksModel,
        "DELETE FROM tracks WHERE id=$1 AND author_username=$2 RETURNING *",
        input_payload.track_id,
        input_payload.username,
    )
    .fetch_all(&state.tracks_pool)
    .await;

    match query_result {
        Ok(tracks_vec) => match tracks_vec.len() {
            0 => Ok((
                StatusCode::NOT_FOUND,
                format!(
                    "Track with id {} doesn't exist or it is not yours",
                    input_payload.track_id
                ),
            )
                .into_response()),
            1 => match fs::remove_file(format!("tracks/{}.mp3", input_payload.track_id)) {
                Ok(_) => Ok((StatusCode::OK).into_response()),
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unknown filesystem error",
                )
                    .into_response()),
            },
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown database error").into_response()),
        },
        Err(_) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown database error").into_response())
        }
    }
}

async fn download_track(
    Query(params): Query<DownloadTrackParams>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let file_path = format!("tracks/{}.mp3", params.id);

    match File::open(&file_path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            if let Err(_) = file.read_to_end(&mut buffer) {
                let resp: Bytes = "Failed to read file".as_bytes().into();
                return Err((StatusCode::INTERNAL_SERVER_ERROR, resp).into_response());
            }

            let bytes_file: Bytes = buffer.into();
            let mut resp = (StatusCode::OK, bytes_file).into_response();
            resp.headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("audio/mpeg"));
            resp.headers_mut().insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"file.mp3\""),
            );
            Ok(resp)
        }
        Err(_) => Ok((StatusCode::NOT_FOUND, "File not found".as_bytes()).into_response()),
    }
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let template = format!("%{}%", params.query);
    let query_result = sqlx::query_as!(
        TracksModel,
        "SELECT * FROM tracks WHERE name LIKE $1",
        template
    )
    .fetch_all(&state.tracks_pool)
    .await;

    match query_result {
        Ok(vec_tracks) => {
            let result: Vec<SearchResponseItem> = vec_tracks
                .into_iter()
                .map(|track| SearchResponseItem {
                    id: track.id,
                    author_username: track.author_username,
                    track_name: track.name,
                    rating: match track.cnt_rates {
                        0 => 0.0,
                        _ => (track.sum_rates as f64) / (track.cnt_rates as f64),
                    },
                })
                .collect();
            Ok((StatusCode::OK, Json(result)).into_response())
        }
        Err(_) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown database error").into_response())
        }
    }
}

// async fn signup(
//     State(state): State<Arc<AppState>>,
//     Json(input_payload): Json<SignupRequest>,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     let query_result = sqlx::query_as!(
//         TracksModel,
//         "INSERT INTO tracks VALUES ($1, $2, $3)",
//         input_payload.username,
//         get_hash(&input_payload.password),
//         String::new(),
//     )
//     .execute(&state.pool)
//     .await;

//     match query_result {
//         Ok(_) => {
//             let response = SignupResponse {
//                 username: input_payload.username,
//             };

//             return Ok((StatusCode::CREATED, Json(response)).into_response());
//         }
//         Err(_) => {
//             return Err((StatusCode::CONFLICT, "Username exists").into_response());
//         }
//     };
// }

// async fn login(
//     State(state): State<Arc<AppState>>,
//     Json(input_payload): Json<LoginRequest>,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     let token = generate_token(&input_payload.username);

//     let query_result = sqlx::query_as!(
//         TracksModel,
//         "UPDATE tracks SET active_token=$3 WHERE username=$1 AND password_hash=$2 RETURNING *",
//         input_payload.username,
//         get_hash(&input_payload.password),
//         &token,
//     )
//     .fetch_optional(&state.pool)
//     .await;

//     match query_result {
//         Ok(user_optional) => match user_optional {
//             Some(_) => {
//                 return Ok((StatusCode::OK, [("Authorization", token)]).into_response());
//             }
//             None => {
//                 return Ok((
//                     StatusCode::NOT_FOUND,
//                     "Username doesn't exist or password is wrong",
//                 )
//                     .into_response())
//             }
//         },
//         Err(_) => {
//             return Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response());
//         }
//     };
// }

// async fn logout(
//     State(state): State<Arc<AppState>>,
//     headers: HeaderMap,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     if !headers.contains_key("Authorization") {
//         return Ok((StatusCode::UNAUTHORIZED, "Token is missing").into_response());
//     }
//     let token = headers["Authorization"].to_str().unwrap();

//     let decoded_token = match decode_token(token) {
//         Ok(c) => c.claims,
//         Err(_) => {
//             return Ok((StatusCode::UNAUTHORIZED, "Invalid token").into_response());
//         }
//     };

//     let query_result = sqlx::query_as!(
//         TracksModel,
//         "UPDATE tracks SET active_token='' WHERE username=$1 RETURNING *",
//         &decoded_token.username,
//     )
//     .fetch_optional(&state.pool)
//     .await;

//     match query_result {
//         Ok(user_optional) => match user_optional {
//             Some(_) => Ok((StatusCode::OK).into_response()),
//             None => Ok((StatusCode::NOT_FOUND, "Username doesn't exist").into_response()),
//         },
//         Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()),
//     }
// }

// async fn check_token(headers: HeaderMap) -> impl IntoResponse {
//     if !headers.contains_key("Authorization") {
//         return (StatusCode::UNAUTHORIZED, "Token is missing").into_response();
//     }
//     let token = headers["Authorization"].to_str().unwrap();

//     let decoded_token = match decode_token(token) {
//         Ok(c) => c.claims,
//         Err(_) => {
//             return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
//         }
//     };

//     if exp_expired(decoded_token.exp) {
//         return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
//     }

//     let response = CheckTokenResponse {
//         username: decoded_token.username,
//     };
//     (StatusCode::OK, Json(response)).into_response()
// }
