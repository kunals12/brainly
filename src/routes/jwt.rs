use actix_web::{
    cookie::time::{Duration, OffsetDateTime}, // Used for handling token expiration time
    Error, HttpRequest,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use super::user::PublicUser; // PublicUser struct is imported from the user module

// Structure representing JWT Claims (Payload)
#[derive(Serialize, Debug, Deserialize)]
pub struct Claims {
    sub: PublicUser,    // Subject: Holds user information (ID and username)
    exp: usize          // Expiry time: When the token expires (in seconds since epoch)
}

// Secret key for signing and verifying JWT tokens
// NOTE: `b""` converts the string into a byte array. It's used because JWT libraries work with binary data.
const SECRET_KEY: &[u8] = b"my_super_secret_key"; // Use environment variables in production to avoid hardcoding secrets!

// Implementation of the Claims struct
impl Claims {
    /// Constructor for Claims
    pub fn new(sub: PublicUser, exp: usize) -> Claims {
        Claims { sub, exp }
    }
}

// Function to generate a JWT token
pub fn generate_token(public_user: PublicUser) -> String {
    // Extract user ID and username for the token payload
    let id = public_user.id;
    let username = public_user.username;

    // Calculate expiration time (current time + 2 hours)
    let expiration = OffsetDateTime::now_utc() + Duration::hours(2);

    // Create a Claims instance
    let claims = Claims::new(
        PublicUser { id, username },               // Add user info to the payload
        expiration.unix_timestamp() as usize,     // Set the token expiration time
    );

    // Encode the Claims into a JWT token
    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))
        .expect("Error generating token") // Panic if token generation fails
}

// Function to validate (verify) a JWT token
fn verify_token(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode(
        token,
        &DecodingKey::from_secret(SECRET_KEY),    // Decode using the same secret key
        &Validation::default(),                   // Use default validation (e.g., check expiry)
    )
}

// Middleware to validate JWT token for protected routes
pub async fn validate_token(req: HttpRequest) -> Result<(), Error> {
    // Check if Authorization header is present
    if let Some(auth_header) = req.headers().get("Authorization") {
        // Convert the header to a string
        if let Ok(auth_str) = auth_header.to_str() {
            // Strip the "Bearer " prefix to extract the token
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Verify the token
                if verify_token(token).is_ok() {
                    return Ok(()); // Token is valid
                }
            }
        }
    }
    // Return an error if token is missing or invalid
    Err(actix_web::error::ErrorUnauthorized("Invalid Token"))
}
