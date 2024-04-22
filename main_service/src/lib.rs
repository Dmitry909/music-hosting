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

pub async fn create_app() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
}

async fn root_handler() -> Result<impl IntoResponse, impl IntoResponse> {
    let file_path = "resources/index.html";
    match File::open(&file_path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            if let Err(_) = file.read_to_end(&mut buffer) {
                return Err(
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
                );
            }
            let text_response = String::from_utf8(buffer).unwrap();
            Ok((StatusCode::OK, text_response).into_response())
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response()),
    }
}
