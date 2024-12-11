use askama::Template;
use deadpool_redis::redis::aio;
use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use mongodb::bson::oid::ObjectId;
use tracing::{debug, error, info, instrument};

use crate::{auth::tokens::issue_confirmation_token, endpoints::templates::EmailPage, settings};

/// # Results
///   - Ok(()) if the email was sent successfully.
/// # Errors
///   - Err(String) if the email could not be sent.
/// # Panics
///   - If the settings could not be retrieved.
#[instrument(
    name = "Send email",
    level = "info",
    skip(
        sender_email,
        recipient_email,
        recipient_first_name,
        recipient_last_name,
        subject,
        html_content,
        text_content
    )
)]
pub async fn send_email(
    sender_email: Option<String>,
    recipient_email: String,
    recipient_first_name: String,
    recipient_last_name: String,
    subject: impl Into<String> + Send + Sync,
    html_content: impl Into<String> + Send + Sync,
    text_content: impl Into<String> + Send + Sync,
) -> Result<(), String> {
    info!("Send email function called.");
    let settings = settings::get().expect("Could not get settings.");

    let email = Message::builder()
        .from(
            format!(
                "{} <{}>",
                "JohnWrites",
                if sender_email.is_some() {
                    sender_email.expect("Could not get sender email.")
                } else {
                    settings.email.host_user.clone()
                }
            )
            .parse()
            .expect("Could not parse sender email."),
        )
        .to(format!(
            "{} <{}>",
            [recipient_first_name, recipient_last_name].join(" "),
            recipient_email
        )
        .parse()
        .expect("Could not parse recipient email."))
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text_content.into()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html_content.into()),
                ),
        )
        .expect("Could not build email.");

    let creds = Credentials::new(settings.email.host_user, settings.email.host_user_password);

    // Open a remote connection to the mail server
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.email.host)
            .expect("Could not create AsyncSmtpMailer mailer.")
            .credentials(creds)
            .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => {
            info!("Email SUCCESSFULLY sent!.");
            Ok(())
        }
        Err(err) => {
            error!("Email FAILED to send: {err}.");
            Err(format!("Could not send email: {err:?}"))
        }
    }
}

/// # Results
///   - Ok(()) if the multi-part email was sent successfully.
/// # Errors
///   - Err(String) if the email could not be sent.
/// # Panics
///   - If the settings could not be retrieved.
#[instrument(
    name = "Send multipart email",
    level = "info",
    skip(
        subject,
        user_id,
        recipient_email,
        recipient_first_name,
        recipient_last_name,
        template_name,
        redis_connection
    )
)]
pub async fn send_multipart_email(
    subject: String,
    user_id: ObjectId,
    recipient_email: String,
    recipient_first_name: String,
    recipient_last_name: String,
    template_name: &str,
    redis_connection: &mut aio::MultiplexedConnection,
) -> Result<(), String> {
    info!("Send multipart email function called.");
    let settings = settings::get().expect("Could not get settings.");

    let title = subject.clone();

    let issued_token = match issue_confirmation_token(user_id, redis_connection, None).await {
        Ok(token) => token,
        Err(err) => {
            error!("Could not issue confirmation token: {err}.");
            return Err(format!("Could not issue confirmation token: {err:?}"));
        }
    };

    let web_address = {
        if settings.debug {
            format!(
                "{}:{}",
                settings.application.base_url, settings.application.port
            )
        } else {
            settings.application.base_url
        }
    };

    let confirmation_link = {
        debug!("Creating email confirmation link from multi-part template.");
        if template_name == "password_reset_email.html" {
            format!("{web_address}/users/password/confirm/change_password?token={issued_token}")
        } else {
            format!("{web_address}/users/register/confirm/?token={issued_token}")
        }
    };

    let current_date_time = chrono::Local::now();
    let dt = current_date_time + chrono::Duration::minutes(settings.secret.token_expiration);

    let email = EmailPage {
        title,
        confirmation_link: confirmation_link.clone(),
        domain: settings.frontend_url,
        expiration_time: settings.secret.token_expiration.to_string(),
        exact_time: dt.format("%A %B %d, %Y at %r").to_string(),
    };

    let template = email.render().expect("Could not render email template.");

    let text = format!("Tap the link below to confirm your email address.{confirmation_link}");

    // actix_web::rt::spawn(send_email(
    //     None,
    //     recipient_email,
    //     recipient_first_name,
    //     recipient_last_name,
    //     subject,
    //     template,
    //     text,
    // ));

    info!(
        "None, recipient_email: {}, recipient_first_name: {}, recipient_last_name: {}, subject: {}, template: {}, text: {}",
         recipient_email, recipient_first_name, recipient_last_name, subject, template, text
    );

    Ok(())
}
