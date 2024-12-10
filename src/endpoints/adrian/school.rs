use actix_web::{post, web, Error, HttpResponse};
use mongodb::Database;
use tracing::instrument;

/// All things regarding grade, teachers, classes, and pictures

#[post("/grade")]
#[instrument(
    name = "Adrian",
    level = "info",
    target = "aj_studying",
    skip(data, pool)
)]
pub async fn grades(
    data: web::Json<Grade>,
    pool: web::Data<Database>,
) -> Result<HttpResponse, Error> {
    let grade = Grade {
        grade: data.grade.clone(),
        teacher: data.teacher.clone(),
        class: data.class.clone(),
        picture: data.picture.clone(),
    };

    let res = web::block(move || grade.save(&conn)).await;

    match res {
        Ok(_) => Ok(HttpResponse::Ok().json("Grade added")),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    }
}

struct Grade {
    grade: String,
    teacher: String,
    class: String,
    picture: String,
}

impl Grade {
    fn save(
        &self,
        conn: &deadpool_redis::Connection,
    ) -> Result<(), deadpool_redis::redis::RedisError> {
        let _: () = redis::cmd("SET")
            .arg(&self.grade)
            .arg(&self.teacher)
            .arg(&self.class)
            .arg(&self.picture)
            .query(conn)?;

        Ok(())
    }
}
