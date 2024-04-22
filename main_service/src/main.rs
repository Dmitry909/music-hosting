use main_service;

#[tokio::main]
async fn main() {
    let app = main_service::create_app().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
