use std::env;
use tracks_service;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./tracks_service [port]");
        std::process::exit(1);
    }
    let port = &args[1];
    let host = format!("0.0.0.0:{}", port);

    let tracks_db_url = "postgresql://postgres:qwerty@localhost:5432/tracks_service";

    let app = tracks_service::create_app(tracks_db_url, false).await;

    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
