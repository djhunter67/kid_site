use actix_session::Session;
use actix_web::{
    get,
    http::StatusCode,
    post,
    rt::task,
    web::{Data, Form},
    HttpResponse,
};
use askama::Template;
use mongodb::{bson::oid::ObjectId, Database};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    auth::hash::verify_pw,
    endpoints::{
        health::render_error,
        structure::Login,
        templates::{Index, LoginPage},
    },
    models::mongo::{MongoRepo, User},
    types,
};

#[get("/")]
#[instrument(name = "Login page", level = "info", target = "kid_data", skip(_db))]
pub async fn login(_db: Data<Database>) -> HttpResponse {
    info!("Rendering login page");

    let template = LoginPage { title: "Quiz site" };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Failed to render login page: {err:#?}",);
            return HttpResponse::InternalServerError().finish();
        }
    };
    info!("Login page rendered successfully");

    HttpResponse::Ok().content_type("text/html").body(body)
}

#[allow(clippy::future_not_send)]
#[post("/login")]
#[instrument(
    name = "Login user",
    level = "debug",
    target = "kid_data",
    skip(pool, user, session)
)]
pub async fn login_user(
    pool: Data<Database>,
    Form(user): Form<Login>,
    session: Session,
) -> HttpResponse {
    info!("Login endpoint");
    // Authorization logic

    let tasker = |registered_user: User| {
        debug!("Creating spawn_blocking task to verify password.");
        task::spawn_blocking(move || {
            let registered_creds = registered_user.password;
            let user_attempt = user.password;
            verify_pw(registered_creds, user_attempt)
        })
    };

    let pool = MongoRepo::new(&pool.as_ref().to_owned());

    match pool.get_user(None, Some(&user.email)).await {
        Ok(logged_in_user) => match tasker(logged_in_user.clone())
            .await
            .expect("Async blocking failed")
            .await
        {
            Ok(()) => {
                info!("User logged in successfully.");
                session.renew();
                // match session.insert(types::USER_ID_KEY, logged_in_user.id) {
                //     Ok(()) => info!("`user_id` inserted into session"),
                //     Err(err) => error!("`user_id` cannot be inserted into session: {err:#?}"),
                // }
                // match session.insert(types::USER_EMAIL_KEY, logged_in_user.email) {
                //     Ok(()) => info!("`user_email` inserted into session"),
                //     Err(err) => error!("`user_email` cannot be inserted into session: {err:#?}"),
                // }

                match session.insert(
                    logged_in_user.id.expect("failed to get creds").to_string(),
                    logged_in_user.id,
                ) {
                    Ok(()) => info!("user_id inserted into session"),
                    Err(err) => error!("user_id cannot be inserted into session: {err:#?}"),
                }

                match session.insert(user.email, logged_in_user.email) {
                    Ok(()) => info!("user_email inserted into session"),
                    Err(err) => error!("user_email cannot be inserted into session: {err:#?}"),
                }

                warn!("Session: {:#?}", session.get::<String>("logged_in_user"));

                let template = Index { title: "Quiz site" };

                let body = template.render().expect("Index template rendering");

                HttpResponse::Ok()
                    .content_type("text/html")
                    .append_header(("Authorization", "Bearer token"))
                    .body(body)
            }
            Err(err) => {
                error!("Basic User login failed: {err:#?}",);
                render_error(
                    StatusCode::UNAUTHORIZED,
                    "Invalid email or password",
                    Some("login error"),
                )
            }
        },
        Err(err) => {
            warn!("PW verification failed");
            error!("User login failed: {err:#?}");

            render_error(
                StatusCode::UNAUTHORIZED,
                "Invalid email or password",
                Some("login error"),
            )
        }
    }
}

#[allow(clippy::future_not_send)]
#[post("/logout")]
#[instrument(
    name = "Logout user",
    level = "info",
    target = "kid_data",
    skip(session)
)]
pub async fn logout(session: Session) -> HttpResponse {
    info!("Logout endpoint");
    match session_user_id(&session) {
        Ok(_) => {
            info!("User retreived from db.");
            session.purge();
            HttpResponse::Ok()
                .content_type("text/html")
                .body("<h1>User logged out successfully</h1>")
        }
        Err(err) => {
            error!("Failed to get user from session: {err:#?}");
            HttpResponse::BadRequest()
                .content_type("text/html")
                .body("<h1>We currently have some issues. Kindly try again and ensure you are logged in.</h1>")
        }
    }
}

#[instrument(
    name = "Get user ID from session",
    level = "info",
    target = "kid_data",
    skip(session)
)]
fn session_user_id(session: &Session) -> Result<ObjectId, String> {
    info!("Retrieving user ID from session");
    match session.get(types::USER_ID_KEY) {
        Ok(user_id) => user_id.map_or_else(|| Err("You are not authenticated".to_string()), Ok),
        Err(err) => Err(err.to_string()),
    }
}
