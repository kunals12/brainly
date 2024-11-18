use actix_web::{web::{Data, Json}, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::routes::utils::generate_random_string;

use super::jwt::validate_token;

#[derive(Serialize, Deserialize)]
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
    link: String
}

impl ContentType {
    pub fn to_db_value(&self) -> i32 {
        match self {
            ContentType::Image => 1,
            ContentType::Video => 2,
            ContentType::Article => 3,
            ContentType::Audio => 4,
        }
    }

    pub fn from_db_value(value: i32) -> Option<ContentType> {
        match value {
            1 => Some(ContentType::Image),
            2 => Some(ContentType::Video),
            3 => Some(ContentType::Article),
            4 => Some(ContentType::Audio),
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
    println!("{:?}", token_data);
    match token_data {
        Ok(user_id) => {
            println!("User ID: {}", user_id); // Debugging info

            let random_link = generate_random_string(16);

            // Step 2: Convert ContentType to database value
            let content_type_value = ContentType::to_db_value(&content.type_);
            println!("Content Type {}",content_type_value);

            // Step 2: Insert content into the database
            let result = sqlx::query(
                "INSERT INTO contents (link, type_, title, user_id) VALUES (?, ?, ?, ?)"
            ).bind(random_link.clone()).bind(content_type_value).bind(content.title.clone()).bind(user_id)
            .execute(&**db)
            .await;

            match result {
                Ok(_) => HttpResponse::Created().json(ContentResponse {
                    link: random_link.clone(),
                    type_: (ContentType::from_db_value(content_type_value).expect("Null")),
                    title: content.title.clone()
                }),
                Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
            }
        }
        Err(err) => err, // Propagate the error response (e.g., invalid or missing token)
    }
}

fn get_content_by_id() {}

fn get_content_by_user_id() {}

fn delete_content() {}

}