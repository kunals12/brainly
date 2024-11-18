use actix_web::{web::{delete, get, post, Data}, App, HttpServer};
mod routes;
use routes::{Content, User};
mod database;
use database::database_connetion;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    const PORT: u16 = 8080;

    let database = database_connetion()
        .await
        .expect("Failed to connect to database");
    println!("Database connection established");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(database.clone()))
            .route("/api/v1/signup", post().to(User::create_user))
            .route("/api/v1/signin", post().to(User::signin_user))
            .route("/api/v1/content", post().to(Content::create_content))
            // .route("/api/v1/content", get().to(handler))
            // .route("/api/v1/content", delete().to(handler))
            // .route("/api/v1/brain/share", post().to(handler))
            // .route("/api/v1/brain/:shareLink", get().to(handler))
    })
    .bind(("127.0.0.1", PORT))?
    .run();

    println!("Server is running on port {}", PORT);
    server.await
}
