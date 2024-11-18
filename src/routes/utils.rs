use bcrypt::{hash, DEFAULT_COST, verify};
use rand::{distributions::Alphanumeric, Rng};

pub fn encrypt_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Error Hashing Password")
}

pub fn verify_password(password: &str, hash_password: &str) -> bool {
    verify(password, hash_password).unwrap_or(false)
}

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}