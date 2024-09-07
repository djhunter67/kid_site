use deadpool_redis::redis::{aio, AsyncCommands, RedisError};
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

use crate::{settings, types::tokens::ConfirmationToken};

const SESSION_KEY_PREFIX: &str = "valid_session_key_for_{}";

/// Issues a pasetor token to a user. The token has the user's id encoded.
/// A session_key is also encoded. This key is used to destroy the token
/// as soon as it's been verified. Depending on its usage, the token issued
/// has at most an hour to live. Which means, it is destroyed after its time-to-live.
#[tracing::instrument(name = "Generating and issuing a token", skip(redis_connection))]
pub async fn issue_confirmation_token(
    user_id: ObjectId,
    redis_connection: &mut aio::MultiplexedConnection,
    is_for_password_change: Option<bool>,
) -> Result<String, RedisError> {
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

    redis_connection
        .set(redis_key.clone(),
	     String::new(),
	).await
	.map_err(|err| {
	    tracing::event!(target: "aj_studying", tracing::Level::ERROR, "RedisError (set): {err:?}");
	    err
	})?;

    let settings = settings::get().expect("Cannot load settings");
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

    redis_connection
        .expire(
	    redis_key.clone(),
	    time_to_live.num_seconds(),
	)
        .await
    .map_err(|err| {
		tracing::event!(target: "aj_studying", tracing::Level::ERROR, "RedisError (expire): {err:?}");
		err
    })?;

    let mut claims = Claims::new().expect("Cannot create claims");

    // Set default expiry time for the token; 1 hour
    claims
        .expiration(&dt.to_rfc3339())
        .expect("Cannot set expiration using rfc3339 standard");
    claims
        .add_additional("user_id", serde_json::json!(user_id))
        .expect("Cannot add additional claims");
    claims
        .add_additional("session_key", serde_json::json!(session_key))
        .expect("Cannot add additional claims");

    let sk = SymmetricKey::<V4>::from(settings.secret.secret_key.as_bytes())
        .expect("Cannot create symmetric key");

    Ok(local::encrypt(
        &sk,
        &claims,
        None,
        Some(settings.secret.hmac_secret.as_bytes()),
    )
    .expect("Cannot encrypt claims"))
}

/// Verifies and destroys a token. A token is destroyed immediately
/// it has successfully been verified and all encoded data extracted.
/// Redis is used for such destruction.
#[tracing::instrument(name = "Verifying pasetors token", skip(redis_connection))]
pub async fn verify_confirmation_token_pasetor(
    token: &str,
    redis_connection: &mut aio::MultiplexedConnection,
    is_password: Option<bool>,
) -> Result<ConfirmationToken, String> {
    let settings = settings::get().expect("Cannot load settings");
    let sk = SymmetricKey::<V4>::from(settings.secret.secret_key.as_bytes())
        .expect("Cannot create symmetric key");

    let validation_rules = ClaimsValidationRules::new();
    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
        .map_err(|err| format!("TokenValidator: {err}"))?;

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

                if redis_connection
                    .get::<_, Option<String>>(redis_key.clone())
                    .await
                    .map_err(|err| format!("RedisError (get): {err}"))?
                    .is_none()
                {
                    return Err("Token has been used or expired".to_string());
                }
                redis_connection
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
