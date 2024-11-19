// Import necessary modules and functions
use actix_web::{
    web::{delete, get, post, Data}, // HTTP methods and shared state
    App, HttpServer,
};
mod routes; // Module containing route handlers for users and content
use routes::{Content, User}; // Import Content and User route handlers
mod database; // Module for database connection
use database::database_connetion; // Function to establish a database connection

#[tokio::main] // Macro to designate the main function as an asynchronous Tokio runtime
async fn main() -> std::io::Result<()> {
    const PORT: u16 = 8080; // Server port

    // Step 1: Establish a database connection
    let database = database_connetion()
        .await
        .expect("Failed to connect to database"); // Panic if the database connection fails
    println!("Database connection established");

    // Step 2: Configure and run the HTTP server
    let server = HttpServer::new(move || {
        // Define routes and handlers
        App::new()
            .app_data(Data::new(database.clone())) // Share the database connection across handlers
            // User routes
            .route("/api/v1/signup", post().to(User::create_user)) // User signup endpoint
            .route("/api/v1/signin", post().to(User::signin_user)) // User signin endpoint

            // Content routes
            .route("/api/v1/content", post().to(Content::create_content)) // Create content
            .route("/api/v1/user/content", get().to(Content::get_all_content)) // Get all user content
            .route("/api/v1/content/{id}", get().to(Content::get_content_by_id)) // Get content by ID
            .route("/api/v1/content/{id}", delete().to(Content::delete_content)) // Delete content by ID
            .route("/api/v1/content/link/{link}", get().to(Content::get_content_by_link)) // Get content by link
    })
    .bind(("127.0.0.1", PORT))? // Bind the server to localhost and the specified port
    .run(); // Start the server

    println!("Server is running on port {}", PORT); // Log the server's status
    server.await // Await the server's completion
}
