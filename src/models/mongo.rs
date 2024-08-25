use log::error;
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub sign_up_date: String,
    pub email: String,
    pub password: String,
}

impl User {
    pub const fn new(
        name: String,
        sign_up_date: String,
        email: String,
        password: String,
    ) -> Self {
        Self {
            id: None,
            name,
            sign_up_date,
            email,
            password,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "User {{ id: {:?}, name: {}, sign_up_date: {}, email: {}, password: {} }}",
            self.id, self.name, self.sign_up_date, self.email, self.password
        )
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct MongoRepo {
    collection: Collection<User>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv::dotenv().ok();
        let uri = match env::var("MONGOURI") {
            Ok(url) => url,
            Err(err) => {
                error!("MONGOURI not found in .env: {err}");
                std::process::exit(1);
            }
        };
        let client = match Client::with_uri_str(&uri).await {
            Ok(client) => client,
            Err(err) => {
                error!("Failed to connect to MongoDB: {err}\nExiting...");
                std::process::exit(1);
            }
        };
        let db = client.database("study_pwa");
        let collection: Collection<User> = db.collection("users");

        Self { collection }
    }

    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: None,
            name: new_user.name,
            sign_up_date: new_user.sign_up_date,
            email: new_user.email,
            password: new_user.password,
        };

        let user = self
            .collection
            .insert_one(new_doc)
            .await
            .expect("Failed to insert document into collection");

        Ok(user)
    }

    pub async fn get_user(&self, id: &str) -> Result<User, Error> {
        let Ok(object_id) = ObjectId::parse_str(id) else {
            return Err(Error::DeserializationError {
                message: "Failed to parse ObjectId".to_string(),
            });
        };

        let filter = doc! { "_id": object_id };

        let user = self
            .collection
            .find_one(filter)
            .await
            .expect("Failed to find document in collection");

        Ok(user.expect("Failed to find user"))
    }

    pub async fn update_user(&self, id: String, new_user: User) -> Result<UpdateResult, Error> {
        let obj_id = match ObjectId::parse_str(id) {
            Ok(id_data) => id_data,
            Err(err) => {
                error!("Failed to parse ObjectId: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to parse ObjectId".to_string(),
                });
            }
        };

        let filter = doc! { "_id": obj_id };
        let new_doc = doc! {
            "$set": {
            "name": new_user.name,
            "sign_up_date": new_user.sign_up_date,
            "username": new_user.email,
            }
        };

        let updated_doc = self
            .collection
            .update_one(filter, new_doc)
            .await
            .expect("Failed to update document in collection");

        Ok(updated_doc)
    }

    pub async fn delete_user(&self, id: String) -> Result<DeleteResult, Error> {
        let obj_id = match ObjectId::parse_str(id) {
            Ok(id_data) => id_data,
            Err(err) => {
                error!("Failed to parse ObjectId: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to parse ObjectId".to_string(),
                });
            }
        };

        let filter = doc! { "_id": obj_id };

        let deleted_doc = self
            .collection
            .delete_one(filter)
            .await
            .expect("Failed to delete user in collection");

        Ok(deleted_doc)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, Error> {
        let mut users: Vec<User> = Vec::new();

        let cursor = self
            .collection
            .find(doc! {})
            .await
            .expect("Failed to find documents in collection");

        let cursor_count = self
            .collection
            .count_documents(doc! {})
            .await
            .expect("Failed to count documents in collection");

        for _ in 0..cursor_count {
            let user = cursor
                .deserialize_current()
                .expect("Failed to extract user");
            users.push(user);
        }

        Ok(users)
    }
}
