use actix_web::cookie::time::{Duration, OffsetDateTime};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use super::user::PublicUser;

#[derive(Serialize, Debug, Deserialize)]
pub struct Claims {
    sub: PublicUser,    // Subject (e.g. userId or email)
    exp: usize      // Expiry time (in seconds since epoch)
}

const SECRET_KEY: &[u8] = b"my_super_secret_key"; // Use environment variables in production

impl Claims {
    pub fn new(sub:PublicUser, exp: usize) -> Claims {
        Claims {sub, exp}
    }
}

// Function to generate a JWT token
pub fn generate_token(public_user: PublicUser) -> String {
    let id = public_user.id;
    let username = public_user.username;
    let expiration = OffsetDateTime::now_utc() + Duration::hours(2);    // Token valid for 2 hours
    let claims = Claims::new(PublicUser{id,username}, expiration.unix_timestamp() as usize);


    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY)).expect("Error generating token")
}

// Function to validate a JWT token
pub fn verify_token(token:&str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    decode(token, &DecodingKey::from_secret(SECRET_KEY), &Validation::default())
}