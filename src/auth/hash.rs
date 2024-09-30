use argon2::{
    password_hash::{self, SaltString},
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::rngs::OsRng;
use tracing::{error, instrument, warn};

/// # Result
///   - A string containing the hash of the password
/// # Errors
///   - ``password_hash::Error`` if the password cannot be hashed
/// # Panics
///   - If the password cannot be hashed
#[instrument(
    name = "Password hashing",
    level = "info",
    target = "aj_studying",
    skip(password)
)]
pub async fn pw(password: String) -> Result<String, password_hash::Error> {
    // OS RNG
    let salt = SaltString::generate(&mut OsRng);
    // let salt = salt.as_ref().as_bytes();
    // let mut pwd_buffer: [u8; 32] = [0; 32];
    let pw_buffer = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15_000, 2, 1, None).expect("Failed to create Argon2 parameters"),
    )
    .hash_password(password.as_bytes(), &salt)
    .expect("Failed to hash password");

    match PasswordHash::new(&pw_buffer.to_string()) {
        Ok(hash) => {
            warn!("Password hash created: {}", hash.to_string());
            Ok(hash.to_string())
        }
        Err(err) => {
            error!("Failed to make password hash object: {err}");
            Err(password_hash::Error::Password)
        }
    }
}

/// # Result
///   - Ok(()) if the password is correct
/// # Errors
///   - ``password_hash::Error`` if the password is incorrect
/// # Panics
///   - If the hash is not parsable
#[instrument(
    name = "Password verification",
    level = "info",
    target = "aj_studying",
    skip(registered_creds, user_attempt)
)]
pub async fn verify_pw(
    registered_creds: String,
    user_attempt: String,
) -> Result<(), password_hash::Error> {
    let argon2 = Argon2::default();

    let registered_creds =
        PasswordHash::new(&registered_creds).expect("Failed to parse password hash");

    argon2
        .verify_password(user_attempt.as_bytes(), &registered_creds)
        .inspect_err(|err| error!("Failed to verify password: {err}"))
}
