use crate::routes::utils::{encrypt_password, verify_password};
use crate::routes::{Message, TypeDbError};
use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    id: i32,      // Optional for cases like `CreateUser`
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct PublicUser {
    id: i32,
    username: String,
}

// impl From<User> for PublicUser {
//     fn from(user: User) -> Self {
//         PublicUser {
//             id: user.id.expect("ID should be present"),
//             username: user.username,
//         }
//     }
// }

impl User {
    // Create a new user
    pub async fn create_user(db: Data<MySqlPool>, user: Json<User>) -> impl Responder {
        let is_user_exists = Self::check_user_exists(db.clone(), &user.username).await;

        if is_user_exists {
            return HttpResponse::Conflict().json(Message {
                msg: "User Already Exists".to_string(),
            });
        }

        let hash_password = encrypt_password(&user.password);

        let result = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
            .bind(&user.username)
            .bind(&hash_password)
            .execute(&**db)
            .await;

        match result {
            Ok(data) => HttpResponse::Created().json(PublicUser {
                id: data.last_insert_id() as i32,
                username: user.username.clone(),
            }),
            Err(err) => HttpResponse::InternalServerError().json(TypeDbError {
                error: err.to_string(),
            }),
        }
    }

    // Sign in an existing user
    pub async fn signin_user(db: Data<MySqlPool>, body: Json<User>) -> impl Responder {
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
                    return HttpResponse::BadRequest().json(TypeDbError {
                        error: "Incorrect Password".to_string(),
                    });
                }
                HttpResponse::Ok().json(PublicUser {
                    id: user.id,
                    username: user.username
                })
            }
            Err(_) => HttpResponse::NotFound().json(TypeDbError {
                error: "User Not Found".to_string(),
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
