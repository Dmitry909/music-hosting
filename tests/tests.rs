use axum::Router;
use std::str;

use music_hosting;

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
        let mut app = music_hosting::create_app();

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
        let mut app = music_hosting::create_app();

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
    //     let mut app = music_hosting::create_app();

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
