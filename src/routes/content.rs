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

#[derive(Serialize, Deserialize, Debug, Display, EnumString)]
#[strum(serialize_all = "PascalCase")]
enum ContentType {
    Image,
    Video,
    Article,
    Audio,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    type_: ContentType,
    title: String,
}

#[derive(Serialize)]
pub struct ContentResponse {
    type_: ContentType,
    title: String,
    link: String,
}

#[derive(Serialize, FromRow)]
pub struct UserContents {
    id: i32,
    title: String,
    type_: ContentType,
    link: String,
}

impl ContentType {
    pub fn enum_to_string(&self) -> String {
        match self {
            ContentType::Image => "Image".to_owned(),
            ContentType::Video => "Video".to_owned(),
            ContentType::Article => "Article".to_owned(),
            ContentType::Audio => "Audio".to_owned(),
        }
    }

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

impl Content {
    // Function to create content
    pub async fn create_content(
        db: Data<MySqlPool>,
        req: HttpRequest,
        content: Json<Content>,
    ) -> impl Responder {
        // Step 1: Validate the token and extract user ID and username
        let token_data = validate_token(req).await;
        match token_data {
            Ok(user_id) => {
                let random_link = generate_random_string(16);

                // Step 2: Insert content into the database
                let result = sqlx::query(
                    "INSERT INTO contents (link, type_, title, user_id) VALUES (?, ?, ?, ?)",
                )
                .bind(random_link.clone())
                .bind(content.type_.to_string())
                .bind(content.title.clone())
                .bind(user_id)
                .execute(&**db)
                .await;

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
            Err(err) => err, // Propagate the error response (e.g., invalid or missing token)
        }
    }

    fn get_content_by_id(db: Data<MySqlPool>) {}

    pub async fn get_all_content(db: Data<MySqlPool>, req: HttpRequest) -> impl Responder {
        let token_data = validate_token(req).await;

        match token_data {
            Ok(user_id) => {
                let content: Result<Vec<(i32, String, String, String)>, _> =
                    sqlx::query_as("SELECT id, title, type_, link FROM contents WHERE user_id = ?")
                        .bind(user_id)
                        .fetch_all(&**db)
                        .await;

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
                                    None
                                }
                            })
                            .collect();

                        HttpResponse::Ok().json(SuccessResponse {
                            success: true,
                            message: "Content fetch successfully".to_string(),
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
            Err(err) => err,
        }
    }

    pub async fn delete_content(
        db: Data<MySqlPool>,
        params: Path<i32>, // Content ID from path
        req: HttpRequest,  // Request to validate token
    ) -> impl Responder {
        let token_data = validate_token(req).await;

        match token_data {
            Ok(user_id) => {
                // Run DELETE query
                let response = sqlx::query("DELETE FROM contents WHERE id = ? AND user_id = ?")
                    .bind(params.into_inner()) // Content ID
                    .bind(user_id) // User ID
                    .execute(&**db) // Execute query
                    .await;

                match response {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            HttpResponse::Ok().json(SuccessResponse::<()> {
                                success: true,
                                message: "Content Deleted".to_string(),
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
            Err(e) => e, // Return validation error response
        }
    }
}
