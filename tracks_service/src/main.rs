use tracks_service;

#[tokio::main]
async fn main() {
    let tracks_db_url = "postgresql://postgres:qwerty@localhost:5432/tracks_service";

    let app = tracks_service::create_app(tracks_db_url, false).await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
