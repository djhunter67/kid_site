use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use rand::rngs::OsRng;
use tracing::error;

#[tracing::instrument(name = "Hashing user password", skip(password))]
pub async fn hash_pw(password: &[u8]) -> String {
    let salt = SaltString::generate(OsRng);
    let salt = salt.as_ref().as_bytes();
    let mut pwd_buffer = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, salt, &mut pwd_buffer)
        // .hash_password(password, &salt)
        .expect("Failed to hash password");

    // hex::encode(pwd_buffer)
    // turn the buffer into a string
    let mut hash = String::new();
    for byte in &pwd_buffer {
        hash.push_str(&format!("{byte}"));
    }
    hash
}

#[tracing::instrument(name = "Verifying user password", skip(password, hash))]
pub async fn verify_pw(hash: String, password: Vec<u8>) -> Result<(), password_hash::Error> {
    let parsed_hash = PasswordHash::new(&hash).expect("Failed to parse hash");
    Argon2::default()
        .verify_password(&password, &parsed_hash)
        .map_err(|err| {
            error!("Failed to verify password: {:?}", err);
            err
        })
}
