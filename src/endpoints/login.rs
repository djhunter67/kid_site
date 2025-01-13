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
        error::render_error,
        structure::Login,
        templates::{ErrorPage, Index, LoginPage},
    },
    models::mongo::{MongoRepo, User},
    types::Types,
};

#[allow(clippy::future_not_send)]
#[get("/")]
#[instrument(
    name = "Login page",
    level = "info",
    target = "kid_data",
    skip(_db, session)
)]
pub async fn login(session: Session, _db: Data<Database>) -> HttpResponse {
    info!("Rendering login page");

    warn!("The Session: {:#?}", session.status());

    let template = LoginPage {
        title: "Child Data",
    };

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

    let pool = MongoRepo::new(&pool.as_ref().to_owned(), None);

    match pool.get_user(None, Some(&user.email)).await {
        Ok(logged_in_user) => match tasker(logged_in_user.clone())
            .await
            .expect("Async blocking failed")
            .await
        {
            Ok(()) => {
                info!("User logged in successfully.");
                debug!("Renewing cookie session");
                session.renew();
                match session.insert(
                    Types::UserIdKey,
                    logged_in_user.id.expect("No DB ID found").to_string(),
                ) {
                    Ok(()) => {
                        info!("`user_id` inserted into session");
                        debug!("Changing user Active state to active");
                        pool.toggle_activity(logged_in_user.id.expect("No ID found"), true)
                            .await
                            .expect("User activity not updated");
                    }
                    Err(err) => error!("`user_id` cannot be inserted into session: {err:#?}"),
                }

                // match session.insert(types::USER_EMAIL_KEY, logged_in_user.email) {
                //     Ok(()) => info!("`user_email` inserted into session"),
                //     Err(err) => error!("`user_email` cannot be inserted into session: {err:#?}"),
                // }

                warn!("Session set: {:#?}", session.entries());

                let template = Index {
                    title: "Child Data",
                };

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
    skip(session, pool)
)]
pub async fn logout(session: Session, pool: Data<Database>) -> HttpResponse {
    info!("Logout endpoint");
    match session_user_id(&session) {
        Ok(user_id) => {
            info!("User retreived from db.");
            session.purge();
            let pool = MongoRepo::new(&pool.as_ref().to_owned(), None);
            match pool.toggle_activity(user_id, false).await {
                Ok(_) => info!("user activity updated"),
                Err(err) => {
                    if pool
                        .get_user(Some(user_id), None)
                        .await
                        .expect("User not found")
                        .is_active
                        .expect("No activity available")
                    {
                        let template = LoginPage {
                            title: "Child Data",
                        };

                        let body = match template.render() {
                            Ok(body) => body,
                            Err(err) => {
                                error!("Failed to render login page: {err:#?}",);
                                return HttpResponse::InternalServerError().finish();
                            }
                        };
                        info!("Login page rendered successfully");

                        return HttpResponse::Ok().content_type("text/html").body(body);
                    }
                    return render_error(
                        StatusCode::NO_CONTENT,
                        "No user ID found in cookie data",
                        Some(&format!("Logout Error: {err}")),
                    );
                }
            }
            let template = LoginPage {
                title: "Child Data",
            };

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
        Err(err) => {
            error!("Failed to get user from session: {err:#?}");

            let error_template = ErrorPage {
                title: "Child Data",
                code: 500,
                message: "Unable to logout user. Please try again.",
                error: "Internal Server Error",
            };

            let body = error_template.render().expect("Error template rendering");

            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body)
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
    match session.get(&Types::UserIdKey.to_string()) {
        Ok(user_id) => user_id.map_or_else(|| Err("You are not authenticated".to_string()), Ok),
        Err(err) => Err(err.to_string()),
    }
}

#[instrument(
    name = "Send to login page if no cookie",
    level = "info",
    target = "kid_data",
    skip(session)
)]
pub fn validate_session(session: Session) -> Option<HttpResponse> {
    if session.entries().is_empty() {
        let template = LoginPage {
            title: "Not logged in",
        };

        let body = match template.render() {
            Ok(body) => body,
            Err(err) => {
                error!("Failed to render login page: {err:#?}",);
                return Some(HttpResponse::InternalServerError().finish());
            }
        };
        info!("Login page rendered successfully");

        return Some(HttpResponse::Ok().content_type("text/html").body(body));
    }
    None
}
