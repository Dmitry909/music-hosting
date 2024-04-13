use music_hosting;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[tokio::main]
async fn main() {
    let database_url = "postgresql://postgres:qwerty@localhost:5432/music_hosting";

    let app = music_hosting::create_app(database_url).await; // TODO maybe pool.clone() here?

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
