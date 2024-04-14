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
    tracks_pool: Pool<Postgres>,
}

#[derive(Debug)]
pub struct TracksModel {
    pub username: String,
    pub password_hash: String,
    pub active_token: String,
}

pub async fn create_app(tracks_db_url: &str, need_to_clear: bool) -> Router {
    let tracks_pool = create_pool(tracks_db_url).await;

    if need_to_clear {
        let _ = sqlx::query_as!(TracksModel, "TRUNCATE TABLE tracks",)
            .execute(&tracks_pool)
            .await;
    }

    // sqlx::migrate!("./migrations").run(&pool);

    let shared_state = Arc::new(AppState { tracks_pool });
    Router::new()
        // .route("/delete_account", delete(delete_account))
        // .route("/upload_track", post(upload_track))
        // .route("/delete_track", delete(delete_track))
        // .route("/download_track", get(download_track))
        // .route("/search", get(search))
        // .route("/comment_track", post(comment_track))
        // .route("/delete_comment", delete(delete_comment))
        // .route("/get_comments", get(get_comments))
        .with_state(shared_state)
}

#[derive(Serialize, Deserialize)]
struct DeleteAccountRequest {
    username: String,
}

// async fn delete_account(
//     State(state): State<Arc<AppState>>,
//     Json(input_payload): Json<DeleteAccountRequest>,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     let query_result = sqlx::query_as!(
//         TracksModel,
//         "DELETE FROM tracks WHERE username=$1 AND password_hash=$2 RETURNING *",
//         input_payload.username,
//         get_hash(&input_payload.password),
//     )
//     .fetch_optional(&state.pool)
//     .await;

//     match query_result {
//         Ok(user_optional) => match user_optional {
//             Some(_) => {
//                 let response = DeleteAccountResponse {
//                     username: input_payload.username,
//                 };

//                 return Ok((StatusCode::OK, Json(response)).into_response());
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
