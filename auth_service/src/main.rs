use music_hosting;

#[tokio::main]
async fn main() {
    let users_db_url = "postgresql://postgres:qwerty@localhost:5432/music_hosting";

    let app = music_hosting::create_app(users_db_url, false).await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
