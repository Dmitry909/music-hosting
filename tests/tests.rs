use axum::Router;
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::str;

use music_hosting;

#[cfg(test)]
mod tests {
    use std::io::empty;

    use super::*;
    use axum::body::Body;
    use axum::http::{self, header, HeaderMap, Request};
    use axum::http::{request, StatusCode};
    use axum::routing::head;
    use http_body_util::BodyExt;
    use jsonwebtoken::Header;
    use mime;
    use tower::{Service, ServiceExt};

    fn create_get_request(uri: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn create_get_request_with_header(
        uri: &str,
        header_name: &str,
        header_value: &str,
    ) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .header(header_name, header_value)
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

    fn create_post_request_with_header(
        uri: &str,
        body: Body,
        header_name: &str,
        header_value: &str,
    ) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .header(header_name, header_value)
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

    async fn create_testing_app() -> Router {
        let users_db_url = "postgresql://postgres:qwerty@localhost:5432/music_hosting";
        music_hosting::create_app(users_db_url, true).await
    }

    async fn send_batch_requests(
        app: &mut Router,
        requests: Vec<Request<Body>>,
        expected_exit_codes: Vec<StatusCode>,
        expected_bodies: Vec<&str>,
    ) -> Vec<HeaderMap> {
        assert_eq!(requests.len(), expected_exit_codes.len());
        assert_eq!(requests.len(), expected_bodies.len());

        let mut all_headers = vec![];

        let mut expected_exit_codes_iter = expected_exit_codes.into_iter();
        let mut expected_bodies_iter = expected_bodies.into_iter();

        for request in requests.into_iter() {
            let response = ServiceExt::<Request<Body>>::ready(app)
                .await
                .unwrap()
                .call(request)
                .await
                .unwrap();

            all_headers.push(response.headers().clone());

            let status = response.status();
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let body_string = str::from_utf8(&body).unwrap();

            assert_eq!(
                (status, body_string),
                (
                    expected_exit_codes_iter.next().unwrap(),
                    expected_bodies_iter.next().unwrap()
                )
            );
        }

        all_headers
    }

    #[tokio::test]
    #[serial]
    async fn signup_bad_requests() {
        let mut app = create_testing_app().await;

        let requests = vec![
            create_post_request("/signup", Body::empty()),
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex_no_password\"}"),
            ),
            create_post_request("/signup", Body::from("{\"password\": \"no_username\"}")),
            create_post_request("/signup", Body::from("qwerty")),
            create_post_request("/signup", Body::from("{abc}")),
        ];

        let expected_exit_codes = vec![
            StatusCode::BAD_REQUEST,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::BAD_REQUEST,
            StatusCode::BAD_REQUEST,
        ];

        let expected_bodies = vec![
            "Failed to parse the request body as JSON: EOF while parsing a value at line 1 column 0",
            "Failed to deserialize the JSON body into the target type: missing field `password` at line 1 column 32",
            "Failed to deserialize the JSON body into the target type: missing field `username` at line 1 column 27",
            "Failed to parse the request body as JSON: expected value at line 1 column 1",
            "Failed to parse the request body as JSON: key must be a string at line 1 column 2",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
    }

    #[tokio::test]
    #[serial]
    async fn signup_the_same_user() {
        let mut app = create_testing_app().await;

        let requests = vec![
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
            ),
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
        ];

        let expected_exit_codes = vec![
            StatusCode::CREATED,
            StatusCode::CONFLICT,
            StatusCode::CONFLICT,
        ];

        let expected_bodies = vec![
            "{\"username\":\"alex\"}",
            "Username exists",
            "Username exists",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
    }

    #[tokio::test]
    #[serial]
    async fn signup_and_delete() {
        let mut app = create_testing_app().await;

        let requests = vec![
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_delete_request(
                "/delete_account",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
            ),
            create_delete_request(
                "/delete_account",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_delete_request(
                "/delete_account",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
            ),
            create_delete_request(
                "/delete_account",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
        ];

        let expected_exit_codes = vec![
            StatusCode::CREATED,
            StatusCode::NOT_FOUND,
            StatusCode::OK,
            StatusCode::NOT_FOUND,
            StatusCode::NOT_FOUND,
            StatusCode::CREATED,
        ];

        let expected_bodies = vec![
            "{\"username\":\"alex\"}",
            "Username doesn't exist or password is wrong",
            "{\"username\":\"alex\"}",
            "Username doesn't exist or password is wrong",
            "Username doesn't exist or password is wrong",
            "{\"username\":\"alex\"}",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
    }

    #[tokio::test]
    #[serial]
    async fn login_and_logout() {
        let mut app = create_testing_app().await;

        let requests = vec![
            create_post_request(
                "/login",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request(
                "/login",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1991\"}"),
            ),
            create_post_request(
                "/login",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
        ];

        let expected_exit_codes = vec![
            StatusCode::NOT_FOUND,
            StatusCode::CREATED,
            StatusCode::NOT_FOUND,
            StatusCode::OK,
        ];

        let expected_bodies = vec![
            "Username doesn't exist or password is wrong",
            "{\"username\":\"alex\"}",
            "Username doesn't exist or password is wrong",
            "",
        ];

        let headers =
            send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
        let alex_token = headers[3]["authorization"].to_str().unwrap();

        let requests = vec![
            create_post_request("/logout", Body::empty()),
            create_post_request_with_header("/logout", Body::empty(), "Authorization", "token :)"),
            create_post_request_with_header("/logout", Body::empty(), "Authorization", alex_token),
            create_post_request_with_header("/logout", Body::empty(), "Authorization", alex_token),
            create_delete_request(
                "/delete_account",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request_with_header("/logout", Body::empty(), "Authorization", alex_token),
        ];

        let expected_exit_codes = vec![
            StatusCode::UNAUTHORIZED,
            StatusCode::UNAUTHORIZED,
            StatusCode::OK,
            StatusCode::OK,
            StatusCode::OK,
            StatusCode::NOT_FOUND,
        ];

        let expected_bodies = vec![
            "Token is missing",
            "Invalid token",
            "",
            "",
            "{\"username\":\"alex\"}",
            "Username doesn't exist",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TokenData {
        username: String,
        exp: usize,
    }

    fn generate_token(username: &str, exp: usize) -> String {
        let secret = b"my_secret_key_d47fjs&w3)wj";
        let token_data = TokenData {
            username: username.to_string(),
            exp,
        };
        let encoding_key = EncodingKey::from_secret(secret);
        encode(&Header::default(), &token_data, &encoding_key).unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn login_and_check_token() {
        let mut app = create_testing_app().await;

        let requests = vec![
            create_post_request(
                "/signup",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
            create_post_request(
                "/login",
                Body::from("{\"username\": \"alex\",\"password\": \"alex1990\"}"),
            ),
        ];

        let expected_exit_codes = vec![StatusCode::CREATED, StatusCode::OK];

        let expected_bodies = vec!["{\"username\":\"alex\"}", ""];

        let headers =
            send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
        let alex_token = headers[1]["authorization"].to_str().unwrap();
        let alex_old_token = generate_token(
            "alex",
            (Local::now() - Duration::minutes(1)).timestamp() as usize,
        );
        let alex_ok_token = generate_token(
            "alex",
            (Local::now() + Duration::hours(23)).timestamp() as usize,
        );

        let requests = vec![
            create_get_request_with_header("/check_token", "Authorization", alex_token),
            create_get_request_with_header("/check_token", "Authorization", &alex_old_token),
            create_get_request("/check_token"),
            create_get_request_with_header("/check_token", "Authorization", "invalid token"),
            create_get_request_with_header("/check_token", "Authorization", &alex_ok_token),
        ];

        let expected_exit_codes = vec![
            StatusCode::OK,
            StatusCode::UNAUTHORIZED,
            StatusCode::UNAUTHORIZED,
            StatusCode::UNAUTHORIZED,
            StatusCode::OK,
        ];

        let expected_bodies = vec![
            "{\"username\":\"alex\"}",
            "Token expired",
            "Token is missing",
            "Invalid token",
            "{\"username\":\"alex\"}",
        ];

        send_batch_requests(&mut app, requests, expected_exit_codes, expected_bodies).await;
    }
}
