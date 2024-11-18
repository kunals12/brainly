use super::SuccessResponse;
use actix_web::{
    cookie::time::{Duration, OffsetDateTime}, // Used for handling token expiration time
    HttpRequest, HttpResponse
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize}; // PublicUser struct is imported from the user module

// Structure representing JWT Claims (Payload)
#[derive(Serialize, Debug, Deserialize)]
pub struct Claims {
    sub: i32, // Subject: Holds user information (ID and username)
    exp: usize,      // Expiry time: When the token expires (in seconds since epoch)
}

// Secret key for signing and verifying JWT tokens
// NOTE: `b""` converts the string into a byte array. It's used because JWT libraries work with binary data.
const SECRET_KEY: &[u8] = b"my_super_secret_key"; // Use environment variables in production to avoid hardcoding secrets!

// Implementation of the Claims struct
impl Claims {
    /// Constructor for Claims
    pub fn new(sub: i32, exp: usize) -> Claims {
        Claims { sub, exp }
    }
}

// Function to generate a JWT token
pub fn generate_token(id: i32) -> String {

    // Calculate expiration time (current time + 2 hours)
    let expiration = OffsetDateTime::now_utc() + Duration::hours(2);

    // Create a Claims instance
    let claims = Claims::new(
        id,          // Add user info to the payload
        expiration.unix_timestamp() as usize // Set the token expiration time
    );

    // Encode the Claims into a JWT token
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .expect("Error generating token") // Panic if token generation fails
}

// Function to validate (verify) a JWT token
fn verify_token(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode(
        token,
        &DecodingKey::from_secret(SECRET_KEY), // Decode using the same secret key
        &Validation::default(),                // Use default validation (e.g., check expiry)
    )
}


// Middleware-like function to validate token and extract user data
pub async fn validate_token(req: HttpRequest) -> Result<i32, HttpResponse> {
    if let Some(cookie) = req.cookie("auth_token") {
        let token = cookie.value();
        let verified_token = verify_token(token);
        match verified_token {
            Ok(data) => {
                let user_id = data.claims.sub;
                Ok(user_id) // Return user ID and username
            }
            Err(e) => {
                // Handle invalid token
                Err(HttpResponse::Unauthorized().json(SuccessResponse::<()> {
                    success: false,
                    message: format!("Invalid token: {}", e), // Serialize the error
                    data: None,
                }))
            }
        }
    } else {
        // Handle missing token
        Err(HttpResponse::Unauthorized().json(SuccessResponse::<()> {
            success: false,
            message: "Missing token".to_string(),
            data: None,
        }))
    }
}