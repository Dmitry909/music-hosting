use music_hosting;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[tokio::main]
async fn main() {
    let database_url = "postgresql://postgres:qwerty@localhost:5432/music_hosting";
    let pool = match PgPoolOptions::new()
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let app = music_hosting::create_app(pool); // TODO maybe pool.clone() here?

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
