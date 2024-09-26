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
use log::{debug, error, info, warn};
use mongodb::{bson::oid::ObjectId, Database};

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
            let logged_in = registered_user.password;
            let user_entered = user.password.as_bytes().to_vec();
            verify_pw(logged_in, user_entered)
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

                let template = Index { title: "Quiz site" };

                let body = template.render().expect("Index template rendering");

                HttpResponse::Ok().content_type("text/html").body(body)
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

fn session_user_id(session: &Session) -> Result<ObjectId, String> {
    info!("Retrieving user ID from session");
    match session.get(types::USER_ID_KEY) {
        Ok(user_id) => user_id.map_or_else(|| Err("You are not authenticated".to_string()), Ok),
        Err(err) => Err(err.to_string()),
    }
}
