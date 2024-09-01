use actix_web::cookie::Cookie;
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, DateTime},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Collection,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub sign_up_date: Option<DateTime>,
    pub email: String,
    pub password: String,
}

impl User {
    #[must_use]
    pub const fn new(
        name: String,
        sign_up_date: DateTime,
        email: String,
        password: String,
    ) -> Self {
        Self {
            id: None,
            name,
            sign_up_date: Some(sign_up_date),
            email,
            password,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "User {{ id: {:?}, name: {}, sign_up_date: {:#?}, email: {}, password: {} }}",
            self.id, self.name, self.sign_up_date, self.email, self.password
        )
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct MongoRepo {
    collection: Collection<User>,
}

impl MongoRepo {
    #[must_use]
    pub const fn new(collection: Collection<User>) -> Self {
        Self { collection }
    }

    /// # Results
    ///   - Returns an `InsertOneResult` if the document is successfully inserted into the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to insert into the collection
    /// # Panics
    ///   - If the document fails to insert into the collection
    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: None,
            name: new_user.name,
            sign_up_date: Some(DateTime::now()),
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

    /// # Results
    ///   - Returns a `User` if the document is successfully found in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to find in the collection
    /// # Panics
    ///   - If the document fails to find in the collection
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

    /// # Results
    ///   - Returns an `UpdateResult` if the document is successfully updated in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to update in the collection
    /// # Panics
    ///   - If the document fails to update in the collection
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

    /// # Results
    ///   - Returns a `DeleteResult` if the document is successfully deleted from the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to delete from the collection
    /// # Panics
    ///   - If the document fails to delete from the collection
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

    /// # Results
    ///   - Returns a `Vec<User>` if the documents are successfully found in the collection
    /// # Errors
    ///   - Returns an `Error` if the documents fail to find in the collection
    /// # Panics
    ///   - If the documents fail to find in the collection
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

    /// # Results
    ///   - Returns an `UpdateResult` if the cookie is successfully updated in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to update in the collection
    /// # Panics
    ///   - If the document fails to update in the collection
    pub async fn save_cookie(
        &self,
        user_id: User,
        cookie: Cookie<'_>,
    ) -> Result<UpdateResult, Error> {
        let obj_id = match ObjectId::parse_str(user_id.id.unwrap_or_default().to_string().as_str())
        {
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
            "cookie": cookie.value(),
            }
        };

        let updated_doc = self
            .collection
            .update_one(filter, new_doc)
            .await
            .expect("Failed to update document in collection");

        Ok(updated_doc)
    }

    #[allow(dead_code)]
    /// # Results
    ///   - Returns a `User` if the cookie is successfully found in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to find in the collection
    /// # Panics
    ///   - If the document fails to find in the collection
    pub async fn get_cookie(&self, cookie: Cookie<'_>) -> Result<User, Error> {
        let filter = doc! { "cookie": cookie.value() };

        let user = self
            .collection
            .find_one(filter)
            .await
            .expect("Failed to find document in collection");

        Ok(user.expect("Failed to find user"))
    }

    /// # Results
    ///   - Returns a `DeleteResult` if the cookie is successfully deleted from the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to delete from the collection
    /// # Panics
    ///   - If the document fails to delete from the collection
    pub async fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<DeleteResult, Error> {
        let filter = doc! { "cookie": cookie.value() };

        let deleted_doc = self
            .collection
            .delete_one(filter)
            .await
            .expect("Failed to delete user in collection");

        Ok(deleted_doc)
    }
}
