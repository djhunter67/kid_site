use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse,
};
use askama::Template;
use log::error;

use crate::{
    endpoints::templates::Index,
    models::mongo::{MongoRepo, User},
};

#[get("/")]
pub async fn index() -> HttpResponse {
    let template = Index { title: "AJ Quiz" };

    let body = match template.render() {
        Ok(body) => body,
        Err(err) => {
            error!("Error rendering template: {err:#?}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().body(body)
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
