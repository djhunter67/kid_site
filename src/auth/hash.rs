use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
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
pub async fn verify_pw(hash: String, password: Vec<u8>) -> Result<(), password_hash::Error> {
    warn!("Hashing pw");

    let salt = SaltString::generate(OsRng);

    let argon2 = Argon2::default();

    let pw_hash = argon2
                .hash_password_into(password.as_slice(), salt.as_ref().as_bytes(), &mut [0u8; 32]);

    
    let parsed_hash = match PasswordHash::new(hash.as_str()) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to hash pw: {err}");
            return Err(err);
        }
    };

    Argon2::default()
        .verify_password(&password, &parsed_hash)
        .inspect_err(|err| {
            error!("Failed to verify password: {:?}", err);
        })
}
