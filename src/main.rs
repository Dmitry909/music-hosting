use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc, sync::RwLock};
use std::{io::Read, str};

#[derive(Default)]
struct AppState {
    users: HashMap<String, String>,
}

type SharedState = Arc<RwLock<AppState>>;

fn create_app() -> Router {
    let shared_state = SharedState::default();
    Router::new()
        .route("/singup", post(singup))
        .route("/delete_account", delete(delete_account))
        .with_state(shared_state)
}

#[tokio::main]
async fn main() {
    let app = create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_hash(password: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qIy074EXAsMI");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

async fn singup(
    State(state): State<SharedState>,
    Json(input_payload): Json<SingupRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if users.contains_key(&input_payload.username) {
        return (StatusCode::CONFLICT, "Username exists").into_response();
    }
    users.insert(
        input_payload.username.clone(),
        get_hash(&input_payload.password),
    );

    let response = SingupResponse {
        username: input_payload.username,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn delete_account(
    State(state): State<SharedState>,
    Json(input_payload): Json<DeleteAccountRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if !users.contains_key(&input_payload.username) {
        return (StatusCode::NOT_FOUND, "Username doesn't exist").into_response();
    }

    let password_hash = get_hash(&input_payload.password);
    if users[&input_payload.username] != password_hash {
        return (StatusCode::FORBIDDEN, "Wrong password").into_response();
    }

    users.remove(&input_payload.username);
    let response = DeleteAccountResponse {
        username: input_payload.username,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Deserialize)]
struct SingupRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct SingupResponse {
    username: String,
}

#[derive(Deserialize)]
struct DeleteAccountRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct DeleteAccountResponse {
    username: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{self, Request};
    use axum::http::{request, StatusCode};
    use http_body_util::BodyExt;
    use mime;
    use tower::{Service, ServiceExt};

    fn create_get_request(uri: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn create_post_request(uri: &str, body: Body) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(body)
            .unwrap()
    }

    fn create_delete_request(uri: &str, body: Body) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(uri)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(body)
            .unwrap()
    }

    async fn send_batch_requests(
        app: &mut Router,
        requests: Vec<Request<Body>>,
        expected_exit_codes: Vec<StatusCode>,
        expected_responses: Vec<&str>,
    ) {
        assert_eq!(requests.len(), expected_exit_codes.len());
        assert_eq!(requests.len(), expected_responses.len());

        let mut expected_exit_codes_iter = expected_exit_codes.into_iter();
        let mut expected_responses_iter = expected_responses.into_iter();

        for request in requests.into_iter() {
            let response = ServiceExt::<Request<Body>>::ready(app)
                .await
                .unwrap()
                .call(request)
                .await
                .unwrap();
            assert_eq!(response.status(), expected_exit_codes_iter.next().unwrap());
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let body_string = str::from_utf8(&body).unwrap();
            assert_eq!(body_string, expected_responses_iter.next().unwrap());
        }
    }

    #[tokio::test]
    async fn singup_bad_requests() {
        let mut app = create_app();

        let requests = vec![
            create_post_request("/singup", Body::empty()),
            create_post_request(
                "/singup",
                Body::from("{\"username\": \"alex_no_password\"}"),
            ),
            create_post_request("/singup", Body::from("{\"password\": \"no_username\"}")),
            create_post_request("/singup", Body::from("qwerty")),
            create_post_request("/singup", Body::from("{abc}")),
        ];

        let expected_exit_codes = vec![
            StatusCode::BAD_REQUEST,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::BAD_REQUEST,
            StatusCode::BAD_REQUEST,
        ];

        let expected_responses = vec![
            "Failed to parse the request body as JSON: EOF while parsing a value at line 1 column 0",
            "Failed to deserialize the JSON body into the target type: missing field `password` at line 1 column 32",
            "Failed to deserialize the JSON body into the target type: missing field `username` at line 1 column 27",
            "Failed to parse the request body as JSON: expected value at line 1 column 1",
            "Failed to parse the request body as JSON: key must be a string at line 1 column 2",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_responses).await;
    }

    #[tokio::test]
    async fn singup_the_same_user() {
        let mut app = create_app();

        let requests = vec![
            create_post_request(
                "/singup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request(
                "/singup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
            ),
            create_post_request(
                "/singup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
        ];

        let expected_exit_codes = vec![
            StatusCode::CREATED,
            StatusCode::CONFLICT,
            StatusCode::CONFLICT,
        ];

        let expected_responses = vec![
            "{\"username\":\"alex\"}",
            "Username exists",
            "Username exists",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_responses).await;
    }

    // #[tokio::test]
    // async fn singup_and_delete() {
    //     let mut app = create_app();

    //     let requests = vec![
    //         create_post_request(
    //             "/singup",
    //             Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
    //         ),
    //         create_delete_request(
    //             "/delete_account",
    //             Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
    //         ),
    //         create_delete_request(
    //             "/delete_account",
    //             Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
    //         ),
    //         create_delete_request(
    //             "/delete_account",
    //             Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
    //         ),
    //         create_delete_request(
    //             "/delete_account",
    //             Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
    //         ),
    //         create_post_request(
    //             "/singup",
    //             Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
    //         ),
    //     ];

    //     let expected_exit_codes = vec![
    //         StatusCode::CREATED,
    //         StatusCode::FORBIDDEN,
    //         StatusCode::OK,
    //         StatusCode::NOT_FOUND,
    //         StatusCode::NOT_FOUND,
    //         StatusCode::CREATED,
    //     ];

    //     let expected_responses = vec![
    //         "{\"username\":\"alex\"}",
    //         "Username exists",
    //         "Username exists",
    //     ];
    // }
}

//     let json_body: Json<SingupResponse> = Json::from_bytes(&body).unwrap();
//     assert_eq!(json_body.0.username, "abc");
