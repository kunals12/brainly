use std::str::FromStr;

use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, MySqlPool};
use strum_macros::{Display, EnumString};

use crate::routes::utils::generate_random_string;

use super::{jwt::validate_token, user, SuccessResponse};

// Define the possible content types using an enum
#[derive(Serialize, Deserialize, Debug, Display, EnumString)]
#[strum(serialize_all = "PascalCase")] // PascalCase serialization for compatibility with external systems
enum ContentType {
    Image,
    Video,
    Article,
    Audio,
}

// Struct representing the content creation request payload
#[derive(Serialize, Deserialize)]
pub struct Content {
    type_: ContentType, // Type of content (e.g., Image, Video)
    title: String,      // Title of the content
}

// Struct representing the response for a single content item
#[derive(Serialize)]
pub struct ContentResponse {
    type_: ContentType, // Type of content
    title: String,      // Title of the content
    link: String,       // Generated unique link for the content
}

// Struct representing content data stored in the database
#[derive(Serialize, FromRow)]
pub struct UserContents {
    id: i32,            // Unique ID of the content
    title: String,      // Title of the content
    type_: ContentType, // Type of content
    link: String,       // Link to access the content
}

// Implement helper functions for ContentType
impl ContentType {
    // Convert enum variant to its string representation
    pub fn enum_to_string(&self) -> String {
        match self {
            ContentType::Image => "Image".to_owned(),
            ContentType::Video => "Video".to_owned(),
            ContentType::Article => "Article".to_owned(),
            ContentType::Audio => "Audio".to_owned(),
        }
    }

    // Convert a string to an enum variant, if valid
    pub fn enum_from_string(value: &str) -> Option<ContentType> {
        match value {
            "Image" => Some(ContentType::Image),
            "Video" => Some(ContentType::Video),
            "Article" => Some(ContentType::Article),
            "Audio" => Some(ContentType::Audio),
            _ => None,
        }
    }
}

// Implement CRUD operations for the Content struct
impl Content {
    // Create new content and save it to the database
    pub async fn create_content(
        db: Data<MySqlPool>,    // Database connection pool
        req: HttpRequest,       // Incoming HTTP request
        content: Json<Content>, // JSON payload for the content
    ) -> impl Responder {
        // Validate the JWT token and extract user ID
        let token_data = validate_token(req).await;
        match token_data {
            Ok(user_id) => {
                let random_link = generate_random_string(16); // Generate a unique random link

                // Insert the new content into the database
                let result = sqlx::query(
                    "INSERT INTO contents (link, type_, title, user_id) VALUES (?, ?, ?, ?)",
                )
                .bind(random_link.clone())
                .bind(content.type_.to_string())
                .bind(content.title.clone())
                .bind(user_id)
                .execute(&**db)
                .await;

                // Handle database insertion result
                match result {
                    Ok(_) => {
                        let type_to_string = ContentType::enum_to_string(&content.type_);
                        HttpResponse::Created().json(SuccessResponse {
                            success: true,
                            message: "Content created successfully".to_string(),
                            data: Some(ContentResponse {
                                link: random_link.clone(),
                                type_: ContentType::enum_from_string(&type_to_string)
                                    .expect("Type Not Found"),
                                title: content.title.clone(),
                            }),
                        })
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError().body(format!("Database error: {}", e))
                    }
                }
            }
            Err(err) => err, // Return token validation error response
        }
    }

    // Fetch content by its ID
    pub async fn get_content_by_id(
        db: Data<MySqlPool>, // Database connection pool
        req: HttpRequest,    // Incoming HTTP request
        params: Path<i32>,   // Extracted content ID from the URL path
    ) -> impl Responder {
        let verify_token = validate_token(req).await; // Validate token
        match verify_token {
            Ok(user_id) => {
                // Query the database for content with the given ID and user ID
                let content: Result<(i32, String, String, String), sqlx::Error> = sqlx::query_as(
                    "SELECT id, title, type_, link FROM contents WHERE id = ? AND user_id = ?",
                )
                .bind(params.into_inner())
                .bind(user_id)
                .fetch_one(&**db)
                .await;

                // Handle query result
                match content {
                    Ok(content) => HttpResponse::Ok().json(SuccessResponse {
                        success: true,
                        message: "Content fetched successfully".to_string(),
                        data: Some(content),
                    }),
                    Err(_) => HttpResponse::NotFound().json(SuccessResponse::<()> {
                        success: false,
                        message: "Not Found".to_string(),
                        data: None,
                    }),
                }
            }
            Err(e) => e, // Return token validation error response
        }
    }

    // Fetch all content for a user
    pub async fn get_all_content(db: Data<MySqlPool>, req: HttpRequest) -> impl Responder {
        let token_data = validate_token(req).await; // Validate token

        match token_data {
            Ok(user_id) => {
                // Query the database for all content belonging to the user
                let content: Result<Vec<(i32, String, String, String)>, _> =
                    sqlx::query_as("SELECT id, title, type_, link FROM contents WHERE user_id = ?")
                        .bind(user_id)
                        .fetch_all(&**db)
                        .await;

                // Handle query result
                match content {
                    Ok(rows) => {
                        let contents: Vec<UserContents> = rows
                            .into_iter()
                            .filter_map(|(id, title, type_, link)| {
                                if let Ok(type_enum) = ContentType::from_str(&type_) {
                                    Some(UserContents {
                                        id,
                                        title,
                                        type_: type_enum,
                                        link,
                                    })
                                } else {
                                    None // Skip invalid content type
                                }
                            })
                            .collect();

                        HttpResponse::Ok().json(SuccessResponse {
                            success: true,
                            message: "Content fetched successfully".to_string(),
                            data: Some(contents),
                        })
                    }
                    Err(err) => HttpResponse::InternalServerError().json(SuccessResponse::<()> {
                        success: false,
                        message: err.to_string(),
                        data: None,
                    }),
                }
            }
            Err(err) => err, // Return token validation error response
        }
    }

    // Delete content by its ID
    pub async fn delete_content(
        db: Data<MySqlPool>, // Database connection pool
        params: Path<i32>,   // Extracted content ID from the URL path
        req: HttpRequest,    // Incoming HTTP request
    ) -> impl Responder {
        let token_data = validate_token(req).await; // Validate token

        match token_data {
            Ok(user_id) => {
                // Run DELETE query to remove content by ID and user ID
                let response = sqlx::query("DELETE FROM contents WHERE id = ? AND user_id = ?")
                    .bind(params.into_inner()) // Content ID
                    .bind(user_id) // User ID
                    .execute(&**db)
                    .await;

                match response {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            HttpResponse::Ok().json(SuccessResponse::<()> {
                                success: true,
                                message: "Content deleted".to_string(),
                                data: None,
                            })
                        } else {
                            HttpResponse::NotFound().json(SuccessResponse::<()> {
                                success: false,
                                message: "Content not found or not owned by user".to_string(),
                                data: None,
                            })
                        }
                    }
                    Err(e) => HttpResponse::InternalServerError().json(SuccessResponse::<()> {
                        success: false,
                        message: e.to_string(),
                        data: None,
                    }),
                }
            }
            Err(e) => e, // Return token validation error response
        }
    }

    pub async fn get_content_by_link(
        db: Data<MySqlPool>,  // Shared database connection pool
        params: Path<String>, // Path parameter, representing the unique content link
        req: HttpRequest,     // HTTP request object, used for token validation
    ) -> impl Responder {
        // Returns a response that implements the Responder trait
        // Step 1: Validate the JWT token from the request
        match validate_token(req).await {
            Ok(_) => {
                // Step 2: Query the database to fetch content using the provided link
                let content: Result<(i32, String, String, String), sqlx::Error> =
                    sqlx::query_as("SELECT id, title, type_, link FROM contents WHERE link = ?")
                        .bind(params.into_inner()) // Bind the link from the path parameter
                        .fetch_one(&**db) // Execute the query on the database
                        .await;

                // Step 3: Handle the query result
                match content {
                    Ok(content) => {
                        // If the content is found, return a successful response with content data
                        HttpResponse::Ok().json(SuccessResponse {
                            success: true,
                            message: "Content Fetch Success".to_string(),
                            data: Some(content),
                        })
                    }
                    Err(e) => {
                        // If the content is not found, return a 404 Not Found response
                        HttpResponse::NotFound().json(SuccessResponse::<()> {
                            success: false,
                            message: "Content Not Found".to_string(),
                            data: None,
                        })
                    }
                }
            }
            Err(e) => e, // If token validation fails, return the validation error response
        }
    }
}
