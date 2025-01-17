use actix_web::{
    delete, get,
    http::StatusCode,
    post, put,
    web::{Data, Json, Path},
    HttpResponse,
};
use mongodb::{bson::oid::ObjectId, Database};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    endpoints::{error::render_error, register::CreateNewUser},
    models::mongo::{MongoRepo, User},
};

#[post("/user")]
#[instrument(name = "Create user", level = "debug", target = "kid_data", skip(client), fields(id = %new_user.email))]
pub async fn create(client: Data<Database>, new_user: Json<CreateNewUser>) -> HttpResponse {
    info!("Creating user API endpoint");
    let db = MongoRepo::new(&client.as_ref().to_owned(), None);
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
#[instrument(name = "Get user", level = "info", target = "kid_data", skip(client, path), fields(id = %path.email))]
pub async fn get_user(client: Data<Database>, path: Path<User>) -> HttpResponse {
    info!("Getting user API endpoint");
    let db = MongoRepo::new(&client.as_ref().to_owned(), None);

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
#[instrument(
    name = "Update user",
    level = "info",
    target = "kid_data",
    skip(client, path, new_user)
)]
pub async fn update_user(
    client: Data<Database>,
    path: Path<String>,
    new_user: Json<User>,
) -> HttpResponse {
    info!("Updating user API endpoint");
    let db = MongoRepo::new(&client.as_ref().to_owned(), None);

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
#[instrument(
    name = "Delete user",
    level = "debug",
    target = "kid_data",
    skip(client, path)
)]
pub async fn delete_user(client: Data<Database>, path: Path<String>) -> HttpResponse {
    info!("Deleting user API endpoint");
    let db = MongoRepo::new(&client.as_ref().to_owned(), None);
    let user_id = path.into_inner();

    if user_id.is_empty() {
        error!("No ID found");
        return HttpResponse::BadRequest().into();
    }

    debug!("Deleting user");
    let delete_result = db.delete_user(user_id.clone()).await;

    let deleted = delete_result;
    if deleted.expect("Database error").deleted_count == 1 {
        debug!("User deleted successfully");
        return HttpResponse::Ok().body("<h1>User deleted successfully</h1>");
    }
    warn!("User not found");
    render_error(StatusCode::NOT_FOUND, "User not found", None)
}

#[get("/users")]
#[instrument(
    name = "Get all users",
    level = "debug",
    target = "kid_data",
    skip(client)
)]
pub async fn get_users(client: Data<Database>) -> HttpResponse {
    info!("Getting all users API endpoint");
    let db = MongoRepo::new(&client.as_ref().to_owned(), None);
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
