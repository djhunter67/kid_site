use actix_web::{
    get, post,
    web::{Data, Form},
    HttpResponse,
};
use askama::Template;
use log::{error, info, warn};
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
pub async fn login() -> HttpResponse {
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
                error: &err.to_string(),
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

        return HttpResponse::InternalServerError()
            .content_type("text/html")
            .body(body);
    }

    let user = User {
        id: None,
        name: String::from("None"),
        sign_up_date: datetime::DateTime::now().to_string(),
        email: credential.email,
        password: credential.password,
    };

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
