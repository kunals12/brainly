use actix_web::{web::Data, App, HttpServer};
mod routes;
use routes::*;
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
            .service(create_user)
            .service(signin_user)
    })
    .bind(("127.0.0.1", PORT))?
    .run();

    println!("Server is running on port {}", PORT);
    server.await
}
