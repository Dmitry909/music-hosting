use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, sync::RwLock};

#[derive(Default)]
struct AppState {
    users: HashMap<String, String>,
}

type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    let shared_state = SharedState::default();

    let app = Router::new()
        .route("/singup", post(singup))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn singup(
    State(state): State<SharedState>,
    Json(input_payload): Json<SingupRequest>,
) -> impl IntoResponse {
    let users = &mut state.write().unwrap().users;
    if users.contains_key(&input_payload.username) {
        return StatusCode::CONFLICT.into_response();
    }
    users.insert(input_payload.username.clone(), input_payload.password);

    let response = SingupResponse {
        username: input_payload.username,
    };

    (StatusCode::CREATED, Json(response)).into_response()
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::http::{self, Request};
    use http_body_util::BodyExt;
    use mime;
    use tower::ServiceExt;

    fn make_get_request(uri: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn make_post_request(uri: &str, body: Body) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(body)
            .unwrap()
    }

    #[tokio::test]
    async fn singup_user_different_bodies() {
        // TODO: make app and shared_state common, have to use not one_shot
        {
            let shared_state = SharedState::default();
            let app = Router::new()
                .route("/singup", post(singup))
                .with_state(shared_state);
            let response = app
                .oneshot(make_post_request("/singup", Body::empty()))
                .await
                .unwrap();
            // TODO: why BAD_REQUEST here? Must be UNPROCESSABLE_ENTITY.
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            // let json_body: Json<SingupResponse> = Json::from_bytes(&body).unwrap();
            // assert_eq!(json_body.0.username, "abc");
            assert_eq!(&body[..], b"Failed to parse the request body as JSON: EOF while parsing a value at line 1 column 0");
        }

        {
            let shared_state = SharedState::default();
            let app = Router::new()
                .route("/singup", post(singup))
                .with_state(shared_state);
            let response = app
                .oneshot(make_post_request(
                    "/singup",
                    Body::from("{\"username\": \"alex_no_password\"}"),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(&body[..], b"Failed to deserialize the JSON body into the target type: missing field `password` at line 1 column 32");
        }

        {
            let shared_state = SharedState::default();
            let app = Router::new()
                .route("/singup", post(singup))
                .with_state(shared_state);
            let response = app
                .oneshot(make_post_request(
                    "/singup",
                    Body::from("{\"password\": \"no_username\"}"),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(&body[..], b"Failed to deserialize the JSON body into the target type: missing field `username` at line 1 column 27");
        }

        {
            let shared_state = SharedState::default();
            let app = Router::new()
                .route("/singup", post(singup))
                .with_state(shared_state);
            let response = app
                .oneshot(make_post_request(
                    "/singup",
                    Body::from("{\"password\": \"no_username\"}"),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(&body[..], b"Failed to deserialize the JSON body into the target type: missing field `username` at line 1 column 27");
        }

        {
            let shared_state = SharedState::default();
            let app = Router::new()
                .route("/singup", post(singup))
                .with_state(shared_state);
            let response = app
                .oneshot(make_post_request(
                    "/singup",
                    Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(&body[..], b"{\"username\":\"alex\"}");
        }

        // {
        //     let response = app.oneshot(make_post_request("/singup", Body::from("abc"))).await.unwrap();
        //     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        // }
    }
}
