use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use log::{error, warn};
pub(crate) use rand::rngs::OsRng;
use sha3::{Digest, Sha3_256};

/// # Result
///   - A string containing the hash of the password
/// # Panics
///   - If the password cannot be hashed
pub async fn pw(password: String) -> String {
    let salt = SaltString::from_b64("salt-has-to-be-longer--but-how-long")
        .expect("Failed to generate salt");
    let salt = salt.as_ref().as_bytes();
    let mut pwd_buffer = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut pwd_buffer)
        .expect("Failed to hash password");

    pwd_buffer.iter().fold(String::new(), |mut acc, byte| {
        acc.push_str(&byte.to_string());
        acc
    })

    // let password_hashed = Sha3_256::digest(password);

    // password_hashed.iter().fold(String::new(), |mut acc, byte| {
    //     acc.push_str(&byte.to_string());
    //     acc
    // })
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
) -> Result<(), password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password = pw(user_attempt).await;

    warn!("Hashing pw");
    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hw) => hw.to_string(),
        Err(err) => {
            error!("Failed to hash user-attempt: {err}");
            return Err(err);
        }
    };

    let pword_to_check = match argon2.hash_password(registered_creds.as_bytes(), &salt) {
        Ok(hw) => hw.to_string(),
        Err(err) => {
            error!("Failed to hash registered pw: {err}");
            return Err(err);
        }
    };

    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to cast: {err}");
            return Err(err);
        }
    };

    let unused_parse = match PasswordHash::new(&pword_to_check) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to cast: {err}");
            return Err(err);
        }
    };

    warn!("Hash Passwords Check: {}", password_hash == pword_to_check);
    warn!("Hash Pass_1: {}", password_hash);
    warn!("Hash Pass_2: {}", pword_to_check);
    warn!("Pre-Hash PW Check: {}", password == registered_creds);
    warn!("Password 1    : {}", password);
    warn!("Password from : {}", registered_creds);
    warn!("Parsed Hash: {:?}", parsed_hash);
    warn!("Unused Parse: {:?}", unused_parse);

    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .inspect_err(|err| {
            error!("Failed to verify password: {:?}", err);
        })
}
