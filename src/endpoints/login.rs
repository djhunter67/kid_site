use actix_session::Session;
use actix_web::{
    get, post,
    rt::task,
    web::{Data, Form},
    HttpResponse,
};
use askama::Template;
use log::{error, info};
use mongodb::bson::oid::ObjectId;

use crate::{
    auth::hash::verify_pw,
    endpoints::{
        structure::Login,
        templates::{ErrorPage, Index, LoginPage},
    },
    models::mongo::MongoRepo,
    types,
};

#[get("/")]
pub async fn login() -> HttpResponse {
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
    pool: Data<MongoRepo>,
    Form(user): Form<Login>,
    session: Session,
) -> HttpResponse {
    // Authorization logic

    match pool.get_active_user(&user.email).await {
        Ok(logged_in_user) => match task::spawn_blocking(move || {
            let logged_in = logged_in_user.password.clone();
            let user_entered = user.password;
            verify_pw(logged_in, user_entered.as_bytes().to_vec())
        })
        .await
        .expect("Async blocking failed")
        .await
        {
            Ok(()) => {
                info!("User logged in successfully.");
                session.renew();
                session
                    .insert(types::USER_ID_KEY, logged_in_user.id)
                    .expect("`user_id` cannot be inserted into session");
                session
                    .insert(types::USER_EMAIL_KEY, logged_in_user.email)
                    .expect("`user_email` cannot be inserted into session");

                let template = Index { title: "Quiz site" };

                let body = template.render().expect("Index template rendering");

                HttpResponse::Ok().content_type("text/html").body(body)
            }
            Err(err) => {
                error!("User login failed: {err:#?}",);
                HttpResponse::BadRequest()
                    .content_type("text/html")
                    .body("<h1>Unauthorized</h1>")
            }
        },
        Err(err) => {
            error!("User login failed: {err:#?}");

            let template = ErrorPage {
                title: "Login Error",
                code: 500,
                message: "Invalid email or password",
                error: "login error",
            };

            let body = template.render().expect("Login Error template rendering");
            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body)
        }
    }
}

#[post("/logout")]
pub async fn logout(session: Session) -> HttpResponse {
    info!("Logout endpoint");
    match session_user_id(&session).await {
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

async fn session_user_id(session: &Session) -> Result<ObjectId, String> {
    info!("Retrieving user ID from session");
    match session.get(types::USER_ID_KEY) {
        Ok(user_id) => user_id.map_or_else(|| Err("You are not authenticated".to_string()), Ok),
        Err(err) => Err(err.to_string()),
    }
}
