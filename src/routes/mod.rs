pub mod user;
pub use user::User;
pub use serde::{Deserialize, Serialize};
pub mod utils;
pub mod content;
pub use content::Content;

// Struct to send error messages in JSON format if a database error occurs
#[derive(Serialize, Deserialize)]
struct TypeDbError {
    error: String, // Stores the error message to return in the response
}

#[derive(Serialize, Deserialize)]
struct Message {
    msg: String,
}
