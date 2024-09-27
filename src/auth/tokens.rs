use deadpool_redis::redis::{aio, AsyncCommands, RedisError};
use log::{debug, error, info, warn};
use mongodb::bson::oid::ObjectId;
use pasetors::{
    claims::{Claims, ClaimsValidationRules},
    keys::SymmetricKey,
    local,
    token::UntrustedToken,
    version4::V4,
    Local,
};
use rand::{rngs::OsRng, RngCore};

use crate::{
    settings::{self, Settings},
    types::tokens::ConfirmationToken,
};

const SESSION_KEY_PREFIX: &str = "aj_studying_{}";

/// # Result
///   - Ok(String): A token has been issued successfully
/// # Errors
///   - ``RedisError``: An error occurred while interacting with redis
/// # Panics
///   - If the settings cannot be loaded
/// # Notes
/// Issues a pasetor token to a user. The token has the user's id encoded.
/// A ``session_key`` is also encoded. This key is used to destroy the token
/// as soon as it's been verified. Depending on its usage, the token issued
/// has at most an hour to live. Which means, it is destroyed after its time-to-live.
pub async fn issue_confirmation_token(
    user_id: ObjectId,
    redis_connection: &mut aio::MultiplexedConnection,
    is_for_password_change: Option<bool>,
) -> Result<String, RedisError> {
    info!("issue_confirmation_token called");
    let session_key: String = {
        let mut buff = [0u8; 128];
        OsRng.fill_bytes(&mut buff);
        hex::encode(buff)
    };

    let redis_key = {
        if is_for_password_change.is_some() {
            format!("{SESSION_KEY_PREFIX}{session_key}is_for_password_change")
        } else {
            format!("{SESSION_KEY_PREFIX}{user_id}")
        }
    };

    let () = redis_connection
        .set(redis_key.clone(), String::new())
        .await
        .map_err(|err| {
            error!("RedisError (set): {err:?}");
            err
        })?;

    let settings: Settings = settings::get().expect("Cannot load settings for token issuance");

    let current_date_time = chrono::Local::now();
    let dt = {
        if is_for_password_change.is_some() {
            current_date_time + chrono::Duration::hours(1)
        } else {
            current_date_time + chrono::Duration::hours(settings.secret.token_expiration)
        }
    };

    let time_to_live = {
        if is_for_password_change.is_some() {
            chrono::Duration::hours(1)
        } else {
            chrono::Duration::minutes(settings.secret.token_expiration)
        }
    };

    let () = redis_connection
        .expire(redis_key.clone(), time_to_live.num_seconds())
        .await
        .map_err(|err| {
            error!("RedisError (expire): {err:?}");
            err
        })?;

    let mut claims = match Claims::new() {
        Ok(claims) => claims,
        Err(err) => {
            error!("Cannot create claims: {err:?}");
            return Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot create claims",
            )));
        }
    };

    // Set default expiry time for the token; 1 hour
    match claims.expiration(&dt.to_rfc3339()) {
        Ok(claims) => claims,
        Err(err) => {
            error!("Cannot set expiration time: {err:?}");
            return Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot set expiration time",
            )));
        }
    }

    match claims.add_additional("user_id", serde_json::json!(user_id)) {
        Ok(claims) => claims,
        Err(err) => {
            error!("Cannot add additional claims: {err:?}");
            return Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot add additional claims",
            )));
        }
    }

    match claims.add_additional("session_key", serde_json::json!(session_key)) {
        Ok(claims) => claims,
        Err(err) => {
            error!("Cannot add additional claims: {err:?}");
            return Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot add additional claims",
            )));
        }
    }

    let secret_key = {
        if settings.secret.secret_key.as_bytes().len() > 32 {
            warn!(
                "The secret key is longer than 32 bytes: {}. It will be truncated to 32 bytes",
                settings.secret.secret_key.as_bytes().len()
            );
            &settings.secret.secret_key.as_bytes()[0..32]
        } else {
            warn!("The secret key is less than 32 bytes.");
            settings.secret.secret_key.as_bytes()
        }
    };

    let sk = match SymmetricKey::<V4>::from(secret_key) {
        Ok(sk) => sk,
        Err(err) => {
            error!("Cannot create symmetric key: {err:?}");
            return Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot create symmetric key",
            )));
        }
    };

    Ok(
        match local::encrypt(
            &sk,
            &claims,
            None,
            Some(settings.secret.hmac_secret.as_bytes()),
        ) {
            Ok(token) => token,
            Err(err) => {
                error!("Cannot encrypt token: {err:?}");
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot encrypt token",
                )));
            }
        },
    )
}

/// # Result
///   - Ok(ConfirmationToken): The token has been verified and destroyed
/// # Errors
///   - ``String``: An error occurred while verifying the token
/// # Panics
///   - If the settings cannot be loaded
/// # Notes
/// Verifies and destroys a token. A token is destroyed immediately
/// it has successfully been verified and all encoded data extracted.
/// Redis is used for such destruction.
pub async fn verify_confirmation_token_pasetor(
    token: &str,
    redis_connection: &mut aio::MultiplexedConnection,
    is_password: Option<bool>,
) -> Result<ConfirmationToken, String> {
    info!("verify_confirmation_token_pasetor called");
    let settings = settings::get().expect("Cannot load settings");
    let sk = SymmetricKey::<V4>::from(settings.secret.secret_key.as_bytes())
        .expect("Cannot create symmetric key");

    let validation_rules = ClaimsValidationRules::new();
    debug!("Creating untrusted token");
    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
        .map_err(|err| format!("TokenValidator: {err}"))?;

    debug!("Decrypting token");
    let trusted_token = local::decrypt(
        &sk,
        &untrusted_token,
        &validation_rules,
        None,
        Some(settings.secret.hmac_secret.as_bytes()),
    )
    .map_err(|err| format!("Pasetor: {err}"))?;

    let claims = trusted_token
        .payload_claims()
        .expect("Cannot get payload claims");

    let uid = serde_json::to_value(claims.get_claim("user_id").expect("Cannot get claim"))
        .map_err(|err| format!("Cannot serialize user_id: {err}"))?;

    debug!("Saving the session key to db");
    match serde_json::from_value::<String>(uid) {
        Ok(uuid_string) => match ObjectId::parse_str(&uuid_string) {
            Ok(user_uuid) => {
                let sss_key = serde_json::to_value(
                    claims.get_claim("session_key").expect("Cannot get claim"),
                )
                .expect("Cannot serialize session_key");
                let session_key = match serde_json::from_value::<String>(sss_key) {
                    Ok(session_key) => session_key,
                    Err(err) => return Err(format!("Cannot deserialize session_key: {err}")),
                };

                let redis_key = {
                    if is_password.is_some() {
                        format!("{SESSION_KEY_PREFIX}{session_key}is_for_password_change")
                    } else {
                        format!("{SESSION_KEY_PREFIX}{session_key}")
                    }
                };

                let _ = redis_connection.get::<_, String>(redis_key.clone()).await;

                let () = redis_connection
                    .del(redis_key.clone())
                    .await
                    .map_err(|err| format!("RedisError (del): {err}"))?;

                Ok(ConfirmationToken { user_id: user_uuid })
            }
            Err(err) => Err(format!("Cannot parse user_id: {err}")),
        },

        Err(err) => Err(format!("Cannot parse user_id: {err}")),
    }
}
