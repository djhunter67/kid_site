use argon2::{
    password_hash::{self, SaltString},
    Argon2, Params,
};

/// # Result
///   - A string containing the hash of the password
/// # Panics
///   - If the password cannot be hashed
pub async fn pw(password: String, salt: SaltString) -> String {
    let salt = salt.as_ref().as_bytes();
    let mut pwd_buffer = [0u8; 32];
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15_000, 2, 1, None).expect("Failed to create Argon2 parameters"),
    )
    .hash_password_into(password.as_bytes(), salt, &mut pwd_buffer)
    .expect("Failed to hash password");

    pwd_buffer.iter().fold(String::new(), |mut acc, byte| {
        acc.push_str(&byte.to_string());
        acc
    })
}

/// # Result
///   - Ok(()) if the password is correct
/// # Errors
///   - ``password_hash::Error`` if the password is incorrect
/// # Panics
///   - If the hash is not parsable
pub async fn verify_pw(
    registered_creds: String,
    user_attempt: String,
    salt: String,
) -> Result<(), password_hash::Error> {
    let password = pw(user_attempt, SaltString::from_b64(&salt)?).await;

    if password == registered_creds {
        Ok(())
    } else {
        Err(password_hash::Error::Password)
    }
}
