use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    HttpResponse,
};
use log::{debug, error, info, warn};
use mongodb::bson::oid::ObjectId;

use crate::models::mongo::{MongoRepo, User};

#[post("/user")]
pub async fn create(client: Data<mongodb::Database>, new_user: Json<User>) -> HttpResponse {
    info!("Creating user API endpoint");
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
pub async fn get_user(client: Data<mongodb::Database>, path: Path<User>) -> HttpResponse {
    info!("Getting user API endpoint");
    let db = MongoRepo::new(client.collection("users"));

    let user_details = db.get_user(Some(path.id.expect("No ID found")), None).await;

    user_details.map_or_else(
        |err| {
            error!("Error getting user: {err:#?}");
            let error_message = format!("Error getting user: {err:#?}");

            HttpResponse::InternalServerError().json(error_message)
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
    info!("Updating user API endpoint");
    let db = MongoRepo::new(client.collection("users"));

    let user_id = path.into_inner();

    if user_id.is_empty() {
        error!("No ID found");
        return HttpResponse::BadRequest().into();
    }

    let data = User {
        id: Some(ObjectId::parse_str(&user_id).expect("Invalid ID")),
        first_name: new_user.first_name.clone(),
        last_name: new_user.last_name.clone(),
        thumbnail: None,
        is_active: Some(false),
        email: new_user.email.clone(),
        sign_up_date: new_user.sign_up_date,
        password: String::from("************"),
    };

    let update_result = db
        .update_user(
            ObjectId::from_bytes(data.id.expect("No ID found").bytes()),
            data.clone(),
        )
        .await;

    match update_result {
        Ok(update) => {
            if update.matched_count == 1 {
                let update_user_info = db.get_user(Some(data.id.expect("No ID found")), None).await;
                return match update_user_info {
                    Ok(user) => HttpResponse::Ok().json(user),
                    Err(err) => HttpResponse::InternalServerError()
                        .json(format!("Error getting user: {err:#?})")),
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
    info!("Deleting user API endpoint");
    let db = MongoRepo::new(client.collection("users"));
    let user_id = path.into_inner();

    if user_id.is_empty() {
        error!("No ID found");
        return HttpResponse::BadRequest().into();
    }

    debug!("Deleting user");
    let delete_result = db.delete_user(user_id.clone()).await;

    match delete_result {
        Ok(delete) => {
            if delete.deleted_count == 1 {
                debug!("User deleted successfully");
                return HttpResponse::Ok().body("User deleted successfully");
            }

            warn!("User not found");
            return HttpResponse::NotFound().body("User not found");
        }
        Err(err) => {
            error!("Error deleting user: {err:#?}");
            HttpResponse::InternalServerError().body(format!("Error deleting user: {err:#?}"))
        }
    };
    error!("User not found");
    HttpResponse::NotFound().body("User not found")
}

#[get("/users")]
pub async fn get_users(client: Data<mongodb::Database>) -> HttpResponse {
    info!("Getting all users API endpoint");
    let db = MongoRepo::new(client.collection("users"));
    debug!("Getting all users");
    let users = db.get_all_users().await;

    users.map_or_else(
        |err| {
            error!("Error getting users: {err:#?}");
            HttpResponse::InternalServerError().body(format!("Error getting users: {err:#?}"))
        },
        |users| HttpResponse::Ok().json(users),
    )
}
