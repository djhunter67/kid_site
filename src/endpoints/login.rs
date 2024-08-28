use actix_web::{
    cookie::Cookie,
    get, post,
    web::{Data, Form},
    HttpResponse,
};
use askama::Template;
use log::{debug, error, info, warn};
use mongodb::bson::datetime;

use crate::{
    endpoints::{
        structure::Login,
        templates::{ErrorPage, Index, LoginPage, RegisterPage},
    },
    models::mongo::{MongoRepo, User},
};

use super::structure::Registration;

#[get("/")]
pub async fn login<'a>(_db: Data<MongoRepo>) -> HttpResponse {
    // let cookie = cookie.value();

    // let confirm_cookie: Cookie = db.get_cookie(cookie).await;

    let template = LoginPage { title: "Quiz site" };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().content_type("text/html").body(body)
}

#[post("/login")]
pub async fn submit_login(db: Data<MongoRepo>, Form(credential): Form<Login>) -> HttpResponse {
    warn!(
        "User ID: {}, Password: {}",
        credential.email, credential.password
    );
    // Authorization logic
    let user: User = match db.get_user(&credential.email).await {
        Ok(user) => user,
        Err(err) => {
            error!(
                "Error getting user: {} -- ERROR: {err:#?}",
                credential.email
            );

            let template = ErrorPage {
                title: "Login Error",
                code: 500,
                message: "Invalid email or password",
                error: "login error",
            };

            let body = match template.render() {
                Ok(body) => body,
                Err(err) => {
                    error!("Error rendering template: {err:#?}");
                    return HttpResponse::InternalServerError().finish();
                }
            };
            return HttpResponse::Unauthorized()
                .content_type("text/html")
                .body(body);
        }
    };

    info!("User found: {user}");

    // Check if user exists
    // if !user.username.eq(&credential.user_id) {
    // return HttpResponse::Unauthorized().body("Invalid username or password");
    // }

    // Check if password is correct
    // if user.password != credential.password {
    // return HttpResponse::Unauthorized().body("Invalid username or password");
    // }

    let secure_cookie: Cookie = actix_web::cookie::Cookie::build("session", "token")
        .http_only(true)
        .secure(true)
        .finish();

    // Save the cookie to the database for the user
    match db.save_cookie(user, secure_cookie.clone()).await {
        Ok(_) => {
            info!("Cookie saved to database");
        }
        Err(err) => {
            error!("Error saving cookie to database: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let template = Index { title: "AJ Quiz" };
    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .cookie(secure_cookie)
        .append_header(("Authorization", "Bearer token"))
        .body(body)
}

#[get("/registration")]
pub async fn registration() -> HttpResponse {
    let template = RegisterPage {
        title: "Registration",
    };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            let template = ErrorPage {
                title: "Login Error",
                code: 500,
                message: "Invalid username or password",
                error: &err.to_string(),
            };

            let body = match template.render() {
                Ok(body) => body,
                Err(err) => {
                    error!("Error rendering template: {err:#?}");
                    return HttpResponse::InternalServerError().finish();
                }
            };

            return HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body);
        }
    };

    HttpResponse::Ok().content_type("text/html").body(body)
}

#[post("/register")]
pub async fn register(db: Data<MongoRepo>, Form(credential): Form<Registration>) -> HttpResponse {
    if (credential.password.is_empty() || credential.password_confirm.is_empty())
        || (credential.email.is_empty())
        || (credential.password != credential.password_confirm)
    {
        let template = ErrorPage {
            title: "Registration Error",
            code: 500,
            message: "Registration Error",
            error: "Passwords do not match or email is empty",
        };

        let body = match template.render() {
            Ok(body) => body,
            Err(err) => {
                error!("Error rendering template: {err:#?}");
                return HttpResponse::InternalServerError().finish();
            }
        };

        return HttpResponse::InternalServerError()
            .content_type("text/html")
            .body(body);
    }

    warn!(
        "User ID: {}, Password: {}, Password Confirm: {}",
        credential.email, credential.password, credential.password_confirm
    );

    debug!(
        "Password Match: {}",
        credential.password == credential.password_confirm
    );

    if credential.password != credential.password_confirm {
        let template = ErrorPage {
            title: "Registration Error",
            code: 500,
            message: "Passwords do not match",
            error: "Passwords do not match",
        };

        let body = match template.render() {
            Ok(body) => body,
            Err(err) => {
                error!("Error rendering template: {err:#?}");
                return HttpResponse::InternalServerError().finish();
            }
        };
        // User Error
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body(body);
    }

    let user = User::new(
        String::new(),
        datetime::DateTime::now(),
        credential.password.clone(),
        credential.email.clone(),
    );

    match db.create_user(user).await {
        Ok(_) => {
            let template = Index { title: "AJ Quiz" };
            let body = match template.render() {
                Ok(body) => body,
                Err(err) => {
                    error!("Error rendering template: {err:#?}");
                    return HttpResponse::InternalServerError().finish();
                }
            };

            HttpResponse::Ok().content_type("text/html").body(body)
        }
        Err(err) => {
            error!("Error creating user: {err:#?}");
            let template = ErrorPage {
                title: "Registration Error",
                code: 500,
                message: "Error creating user",
                error: &err.to_string(),
            };

            let body = match template.render() {
                Ok(body) => body,
                Err(err) => {
                    error!("Error rendering template: {err:#?}");
                    return HttpResponse::InternalServerError().finish();
                }
            };

            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body)
        }
    }
}

#[post("/logout")]
pub async fn logout(db: Data<MongoRepo>) -> HttpResponse {
    let secure_cookie: Cookie = actix_web::cookie::Cookie::build("session", "token")
        .http_only(true)
        .secure(true)
        .finish();

    match db.delete_cookie(secure_cookie.clone()).await {
        Ok(_) => {
            info!("Cookie deleted from database");
        }
        Err(err) => {
            error!("Error deleting cookie from database: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let template = LoginPage { title: "Quiz site" };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .cookie(secure_cookie)
        .body(body)
}
