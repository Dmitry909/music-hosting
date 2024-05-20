use main_service;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        println!("Usage: ./main_service [port] [auth_service:port] [tracks_service:port] [playlists_service:port] [queue_service:port]");
        std::process::exit(1);
    }
    let port = &args[1];
    let host = format!("0.0.0.0:{}", port);

    let app = main_service::create_app(&args[2], &args[3], &args[4], &args[5]).await;

    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
