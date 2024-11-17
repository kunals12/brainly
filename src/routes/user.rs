use crate::routes::utils::encrypt_password;
use crate::routes::{Message, TypeDbError};
use crate::utils::verify_password;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Serialize, Deserialize)]
pub struct CreateUser {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, FromRow)]
struct SigninUserQueryResponse {
    id: i32,
    username: String,
    password: String,
}

#[derive(Serialize)]
struct SigninUserResponse {
    id: i32,
    username: String,
}

#[post("/api/v1/signup")]
pub async fn create_user(db: Data<MySqlPool>, user: Json<CreateUser>) -> impl Responder {
    let is_user_exists = check_user_exists(db.clone(), &user.username).await;

    // Check if the user already exists and return a response early
    if is_user_exists {
        return HttpResponse::Conflict().json(Message {
            msg: "User Already Exists".to_string(),
        });
    }

    // If user doesn't exist, proceed with hashing password and inserting new user
    let hash_password: String = encrypt_password(&user.password);

    let result = sqlx::query("INSERT INTO users (username, password) VALUES (?,?)")
        .bind(user.username.clone())
        .bind(hash_password)
        .execute(&**db)
        .await;

    match result {
        Ok(data) => HttpResponse::Created().json(SigninUserResponse {
            id: data.last_insert_id() as i32,
            username: user.username.clone(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(TypeDbError {
            error: err.to_string(),
        }),
    }
}

#[post("/api/v1/signin")]
pub async fn signin_user(db: Data<MySqlPool>, body: Json<CreateUser>) -> impl Responder {
    // let is_password_correct: bool = verify_password(&body.password, &body.password);

    let response = sqlx::query_as::<_, SigninUserQueryResponse>(
        "SELECT id, username, password FROM users WHERE username = ?",
    )
    .bind(body.username.clone())
    .fetch_one(&**db)
    .await;

    match response {
        Ok(user) => {
            let is_password_correct = verify_password(&body.password, &user.password);

            if !is_password_correct {
                return HttpResponse::BadRequest().json(TypeDbError {
                    error: "Incorrect Password".to_string(),
                });
            }

            HttpResponse::BadRequest().json(SigninUserResponse {
                id: user.id,
                username: user.username,
            })
        }
        Err(_) => HttpResponse::NotFound().json(TypeDbError {
            error: "User Not Found".to_string(),
        }),
    }
}

async fn check_user_exists(db: Data<MySqlPool>, username: &str) -> bool {
    let is_user_exists =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)")
            .bind(username)
            .fetch_one(&**db)
            .await
            .unwrap_or(false);

    is_user_exists
}
