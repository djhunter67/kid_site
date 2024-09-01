use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    HttpResponse,
};
use mongodb::bson::oid::ObjectId;
use tracing::{debug, error, info};

use crate::models::mongo::{MongoRepo, User};

#[post("/user")]
pub async fn create(client: Data<mongodb::Database>, new_user: Json<User>) -> HttpResponse {
    info!("Creating user");
    let db = MongoRepo::new(client.collection("users"));
    let data = new_user.into_inner();
    debug!("Creating user: {:#?}", data);

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
pub async fn get_user(client: Data<mongodb::Database>, path: Path<String>) -> HttpResponse {
    let db = MongoRepo::new(client.collection("users"));
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
    client: Data<mongodb::Database>,
    path: Path<String>,
    new_user: Json<User>,
) -> HttpResponse {
    let db = MongoRepo::new(client.collection("users"));

    let user_id = path.into_inner();

    if user_id.is_empty() {
        return HttpResponse::BadRequest().into();
    }

    let data = User {
        id: Some(ObjectId::parse_str(&user_id).expect("Invalid ID")),
        name: new_user.name.clone(),
        email: new_user.email.clone(),
        sign_up_date: new_user.sign_up_date.clone(),
        password: String::from("************"),
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
pub async fn delete_user(client: Data<mongodb::Database>, path: Path<String>) -> HttpResponse {
    let db = MongoRepo::new(client.collection("users"));
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
pub async fn get_users(client: Data<mongodb::Database>) -> HttpResponse {
    let db = MongoRepo::new(client.collection("users"));
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
