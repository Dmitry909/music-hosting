use auth_service;

#[tokio::main]
async fn main() {
    let users_db_url = "postgresql://postgres:qwerty@localhost:5432/auth_service";

    let app = auth_service::create_app(users_db_url, false).await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
