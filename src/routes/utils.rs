use bcrypt::{hash, DEFAULT_COST, verify};

pub fn encrypt_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Error Hashing Password")
}

pub fn verify_password(password: &str, hash_password: &str) -> bool {
    verify(password, hash_password).unwrap_or(false)
}