use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    HttpResponse,
};
use askama::Template;
use log::{error, info};
use mongodb::bson::oid::ObjectId;

use crate::{
    endpoints::templates::{Index, LoginPage},
    models::mongo::{MongoRepo, User},
};

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
pub async fn submit_login(db: Data<MongoRepo>, user_id: String, password: String) -> HttpResponse {
    // Authorization logic
    let user: User = match db.get_user(&user_id).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid username or password"),
    };

    // Check if user exists
    if !user.username.eq(&user_id) {
        return HttpResponse::Unauthorized().body("Invalid username or password");
    }

    // Check if password is correct
    if user.password != password {
        return HttpResponse::Unauthorized().body("Invalid username or password");
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .append_header(("Authorization", "Bearer token"))
        .body("Login submitted")
}

#[get("/main")]
pub async fn index() -> HttpResponse {
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

#[post("/user")]
pub async fn create_user(db: Data<MongoRepo>, new_user: Json<User>) -> HttpResponse {
    let data = new_user.into_inner();

    let user_details = db.create_user(data).await;

    user_details.map_or_else(
        |err| {
            error!("Error creating user: {err:#?}");
            HttpResponse::InternalServerError().finish()
        },
        |user| HttpResponse::Ok().json(user),
    )
}

#[get("/user/{id}")]
pub async fn get_user(db: Data<MongoRepo>, path: Path<String>) -> HttpResponse {
    let user_id = path.into_inner();

    if user_id.is_empty() {
        return HttpResponse::BadRequest().body("Invalid user ID");
    }

    let user_details = db.get_user(&user_id).await;

    user_details.map_or_else(
        |err| {
            error!("Error getting user: {err:#?}");
            HttpResponse::InternalServerError().finish()
        },
        |user| HttpResponse::Ok().json(user),
    )
}

#[put("/user/{id}")]
pub async fn update_user(
    db: Data<MongoRepo>,
    path: Path<String>,
    new_user: Json<User>,
) -> HttpResponse {
    let user_id = path.into_inner();

    if user_id.is_empty() {
        return HttpResponse::BadRequest().into();
    }

    let data = User {
        id: Some(ObjectId::parse_str(&user_id).expect("Invalid ID")),
        name: new_user.name.clone(),
        username: new_user.username.clone(),
        sign_up_date: new_user.sign_up_date.clone(),
	password: new_user.password.clone(),
    };

    let update_result = db.update_user(user_id.clone(), data).await;

    match update_result {
        Ok(update) => {
            if update.matched_count == 1 {
                let update_user_info = db.get_user(&user_id).await;
                return match update_user_info {
                    Ok(user) => HttpResponse::Ok().json(user),
                    Err(err) => HttpResponse::InternalServerError()
                        .body(format!("Error getting user: {err:#?})")),
                };
            }

            return HttpResponse::NotFound().body("User not found");
        }
        Err(err) => {
            error!("Error updating user: {err:#?}");
            HttpResponse::InternalServerError().body(format!("Error updating user: {err:#?}"))
        }
    };
    HttpResponse::NotFound().body("User not found")
}

#[delete("/user/{id}")]
pub async fn delete_user(db: Data<MongoRepo>, path: Path<String>) -> HttpResponse {
    let user_id = path.into_inner();

    if user_id.is_empty() {
        return HttpResponse::BadRequest().into();
    }

    let delete_result = db.delete_user(user_id.clone()).await;

    match delete_result {
        Ok(delete) => {
            if delete.deleted_count == 1 {
                return HttpResponse::Ok().body("User deleted successfully");
            }

            return HttpResponse::NotFound().body("User not found");
        }
        Err(err) => {
            error!("Error deleting user: {err:#?}");
            HttpResponse::InternalServerError().body(format!("Error deleting user: {err:#?}"))
        }
    };
    HttpResponse::NotFound().body("User not found")
}

#[get("/users")]
pub async fn get_users(db: Data<MongoRepo>) -> HttpResponse {
    info!("Getting all users");
    let users = db.get_all_users().await;

    users.map_or_else(
        |err| {
            error!("Error getting users: {err:#?}");
            HttpResponse::InternalServerError().body(format!("Error getting users: {err:#?}"))
        },
        |users| HttpResponse::Ok().json(users),
    )
}
