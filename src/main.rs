use music_hosting;

#[tokio::main]
async fn main() {
    let app = music_hosting::create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
