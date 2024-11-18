pub mod user;
pub use user::User;
pub use serde::Serialize;
pub mod utils;
pub mod content;
pub use content::Content;
pub mod jwt;

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}