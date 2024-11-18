use crate::routes::utils::{encrypt_password, verify_password};
use actix_web::{
    cookie::{time::Duration, Cookie, SameSite},
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

use super::jwt::generate_token;
use super::SuccessResponse;

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    id: i32, // Optional for cases like `CreateUser`
    username: String,
    password: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct PublicUser {
    pub id: i32,
    pub username: String,
}

impl User {
    // Create a new user
    pub async fn create_user(db: Data<MySqlPool>, user: Json<CreateUser>) -> impl Responder {
        let is_user_exists = Self::check_user_exists(db.clone(), &user.username).await;

        if is_user_exists {
            return HttpResponse::Conflict().json(SuccessResponse::<()> {
                success: false,
                message: "User Already Exists".to_string(),
                data: None
            });
        }

        let hash_password = encrypt_password(&user.password);

        let result = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
            .bind(&user.username)
            .bind(&hash_password)
            .execute(&**db)
            .await;

        match result {
            Ok(data) => HttpResponse::Created().json(SuccessResponse {
                success: true,
                message: "User Created".to_string(),
                data: Some(data.last_insert_id().to_string())
            }),
            Err(err) => HttpResponse::InternalServerError().json(SuccessResponse::<()> {
                success: false,
                message: err.to_string(),
                data: None
            }),
        }
    }

    // Sign in an existing user
    pub async fn signin_user(db: Data<MySqlPool>, body: Json<CreateUser>) -> impl Responder {
        let response = sqlx::query_as::<_, User>(
            "SELECT id, username, password FROM users WHERE username = ?",
        )
        .bind(&body.username)
        .fetch_one(&**db)
        .await;

        match response {
            Ok(user) => {
                let is_password_correct = verify_password(&body.password, &user.password);
                if !is_password_correct {
                    return HttpResponse::BadRequest().json(SuccessResponse::<()> {
                        success: false,
                        message: "Incorrect Password".to_string(),
                        data: None
                    });
                }

                let token = generate_token(user.id);

                let cookie = Cookie::build("auth_token", &token)
                    .path("/")
                    .http_only(true)
                    .max_age(Duration::hours(2))
                    .same_site(SameSite::Strict)
                    .finish();

                HttpResponse::Ok().cookie(cookie).json(SuccessResponse { 
                    success: true,
                    message: "Signin successfully".to_string(),
                    data: Some(token)
                 })
            }
            Err(_) => HttpResponse::NotFound().json(SuccessResponse::<()> {
                success: false,
                message: "User Not Found".to_string(),
                data: None
            }),
        }
    }

    // Check if a user exists
    async fn check_user_exists(db: Data<MySqlPool>, username: &str) -> bool {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)")
            .bind(username)
            .fetch_one(&**db)
            .await
            .unwrap_or(false)
    }
}
