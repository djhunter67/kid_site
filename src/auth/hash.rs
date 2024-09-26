use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use log::{error, warn};
use rand::rngs::OsRng;

/// # Result
///   - A string containing the hash of the password
/// # Panics
///   - If the password cannot be hashed
pub async fn pw(password: &[u8]) -> String {
    let salt = SaltString::generate(OsRng);
    let salt = salt.as_ref().as_bytes();
    let mut pwd_buffer = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, salt, &mut pwd_buffer)
        .expect("Failed to hash password");

    // hex::encode(pwd_buffer)
    // turn the buffer into a string
    let mut hash = String::new();
    for byte in &pwd_buffer {
        hash.push_str(&format!("{byte}"));
    }
    hash
}

/// # Result
///   - Ok(()) if the password is correct
/// # Errors
///   - ``password_hash::Error`` if the password is incorrect
/// # Panics
///   - If the hash is not parsable
pub async fn verify_pw(logged_in: String, password: Vec<u8>) -> Result<(), password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password = pw(password.as_slice()).await;

    warn!("Hashing pw");
    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hw) => hw.to_string(),
        Err(err) => {
            error!("Failed to hash pw: {err}");
            return Err(err);
        }
    };

    let pword_to_check = match argon2.hash_password(logged_in.as_bytes(), &salt) {
        Ok(hw) => hw.to_string(),
        Err(err) => {
            error!("Failed to hash pw: {err}");
            return Err(err);
        }
    };

    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to hash pw: {err}");
            return Err(err);
        }
    };

    let pword_hash = match PasswordHash::new(&pword_to_check) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to hash pw: {err}");
            return Err(err);
        }
    };

    warn!("Passwords Check: {}", password_hash == pword_to_check);
    warn!("Pass_1: {}", password_hash);
    warn!("Pass_2: {}", pword_to_check);

    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .inspect_err(|err| {
            error!("Failed to verify password: {:?}", err);
        })
}
