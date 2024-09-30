use actix_web::cookie::Cookie;
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, DateTime, Document},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use tracing::{debug, error, info, instrument, warn};

use crate::{auth::hash::pw, endpoints::register::CreateNewUser};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub first_name: String,
    pub last_name: String,
    pub is_active: Option<bool>,
    pub thumbnail: Option<String>,
    pub sign_up_date: Option<DateTime>,
    pub email: String,
    pub password: String,
}

// impl User {
//     #[must_use]
//     pub async fn new(
//         first_name: String,
//         last_name: String,
//         sign_up_date: DateTime,
//         email: String,
//         password: &str,
//     ) -> Self {
//         Self {
//             id: None,
//             first_name,
//             last_name,
//             is_active: Some(false),
//             thumbnail: None,
//             sign_up_date: Some(sign_up_date),
//             email,
//             password: pw(String::from(password))
//                 .await
//                 .expect("Password hashing failed"),
//         }
//     }
// }

impl From<CreateNewUser> for User {
    #[instrument(name = "Create user", level = "debug", target = "aj_studying", fields(id = %new_user.email))]
    fn from(new_user: CreateNewUser) -> Self {
        debug!("Extracting user data from the new user");
        Self {
            id: None,
            first_name: new_user.first_name,
            last_name: new_user.last_name,
            is_active: Some(false),
            thumbnail: None,
            sign_up_date: Some(DateTime::now()),
            email: new_user.email,
            password: new_user.password,
        }
    }
}

impl Display for User {
    #[instrument(
        name = "Display user",
        level = "debug",
        target = "aj_studying",
        skip(self, f),
	fields(
	    id = %self.email
	))]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "User {{ id: {:?}, first_name: {}, last_name: {}, sign_up_date: {:#?}, email: {}, password: *********** }}",
            self.id, self.first_name, self.last_name, self.sign_up_date, self.email
        )
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct MongoRepo {
    collection: Collection<User>,
}

impl MongoRepo {
    #[must_use]
    #[instrument(
        name = "Create new MongoRepo",
        level = "debug",
        target = "aj_studying",
        skip(collection)
    )]
    pub fn new(collection: &Database) -> Self {
        let collection = collection.collection("users");

        Self { collection }
    }

    /// # Results
    ///   - Returns an `InsertOneResult` if the document is successfully inserted into the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to insert into the collection
    /// # Panics
    ///   - If the document fails to insert into the collection
    #[instrument(
        name = "Create user",
        level = "debug",
        target = "aj_studying",
        skip(self, new_user)
    )]
    pub async fn create_user(&self, new_user: CreateNewUser) -> Result<InsertOneResult, Error> {
        let mut new_doc = User::from(new_user);
        debug!("Extracted user data from the new user: {}", new_doc.email);

        new_doc.password = pw(new_doc.password).await.expect("Password hashing failed");
        info!("Clear text Password hashed");

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
    #[instrument(
        name = "Get user",
        level = "debug",
        target = "aj_studying",
        skip(self, object_id, email)
    )]
    pub async fn get_user(
        &self,
        object_id: Option<ObjectId>,
        email: Option<&str>,
    ) -> Result<User, Error> {
        info!("Get users endpoint hit");
        let filter: Document = object_id.map_or_else(
            || {
                doc! {
                "email": email.expect("Failed to find email")
                            }
            },
            |id| doc! { "_id": id },
        );

        let user = match self.collection.find_one(filter).await {
            Ok(user) => user,
            Err(err) => {
                error!("Failed to search collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to find document in collection".to_string(),
                });
            }
        };

        Ok(if let Some(user) = user {
            user
        } else {
            error!("Failed to find user");
            return Err(Error::DeserializationError {
                message: "Failed to find user".to_string(),
            });
        })
    }

    /// # Results
    ///   - Returns a `User` if the document, filtered on email,  has ``is_active`` == true in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to find in the collection
    /// # Panics
    ///   - If the document fails to find in the collection
    #[instrument(
        name = "Get active user",
        level = "debug",
        target = "aj_studying",
        skip(self, email)
    )]
    pub async fn get_active_user(&self, email: &str) -> Result<User, Error> {
        let filter = doc! { "email": email, "is_active": false };

        debug!("Filter for search: {:#?}", filter);

        let user = match self.collection.find_one(filter).await {
            Ok(user) => {
                debug!("Finder filter returned");
                user
            }
            Err(err) => {
                error!("Failed to find document in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to find document in collection".to_string(),
                });
            }
        };

        Ok(if let Some(user) = user {
            user
        } else {
            error!("Failed to find active user: {user:#?}");
            return Err(Error::DeserializationError {
                message: "Failed to find user".to_string(),
            });
        })
    }

    /// # Results
    ///   - Returns an `UpdateResult` if the document is successfully updated in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to update in the collection
    /// # Panics
    ///   - If the document fails to update in the collection
    #[instrument(
        name = "Update user",
        level = "debug",
        target = "aj_studying",
        skip(self, object_id, new_user),
	fields(user_to_update = %new_user.email)
    )]
    pub async fn update_user(
        &self,
        object_id: ObjectId,
        new_user: User,
    ) -> Result<UpdateResult, Error> {
        info!("Update user endpoint hit");
        let filter = doc! { "_id": object_id };
        let new_doc = doc! {
            "$set": {
        "first_name": new_user.first_name,
        "last_name": new_user.last_name,
        "email": new_user.email,
        "thumbnail": new_user.thumbnail,
        "sign_up_date": new_user.sign_up_date,
        "is_active": new_user.is_active,
        "password": pw(new_user.password).await.expect("Password hashing failed"),
            }
        };

        let updated_doc = match self.collection.update_one(filter, new_doc).await {
            Ok(doc) => {
                debug!("User updated");
                doc
            }
            Err(err) => {
                error!("Failed to update document in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to update document in collection".to_string(),
                });
            }
        };

        Ok(updated_doc)
    }

    /// # Results
    ///   - Returns a `DeleteResult` if the document is successfully deleted from the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to delete from the collection
    /// # Panics
    ///   - If the document fails to delete from the collection
    #[instrument(
        name = "Delete user",
        level = "debug",
        target = "aj_studying",
        skip(self, id)
    )]
    pub async fn delete_user(&self, id: String) -> Result<DeleteResult, Error> {
        info!("Delete user endpoint hit");
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

        let deleted_doc = match self.collection.delete_one(filter).await {
            Ok(doc) => {
                debug!("User deleted");
                doc
            }
            Err(err) => {
                error!("Failed to delete document in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to delete document in collection".to_string(),
                });
            }
        };

        Ok(deleted_doc)
    }

    /// # Results
    ///   - Returns a `Vec<User>` if the documents are successfully found in the collection
    /// # Errors
    ///   - Returns an `Error` if the documents fail to find in the collection
    /// # Panics
    ///   - If the documents fail to find in the collection
    #[instrument(
        name = "Get all users",
        level = "debug",
        target = "aj_studying",
        skip(self)
    )]
    pub async fn get_all_users(&self) -> Result<Vec<User>, Error> {
        info!("Get all users endpoint hit");
        let mut users: Vec<User> = Vec::new();

        let cursor = match self.collection.find(doc! {}).await {
            Ok(cursor_data) => cursor_data,
            Err(err) => {
                error!("Failed to find documents in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to find documents in collection".to_string(),
                });
            }
        };

        let cursor_count = match self.collection.count_documents(doc! {}).await {
            Ok(count) => count,
            Err(err) => {
                error!("Failed to count documents in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to count documents in collection".to_string(),
                });
            }
        };

        for _ in 0..cursor_count {
            let user = match cursor.deserialize_current() {
                Ok(user_data) => user_data,
                Err(err) => {
                    error!("Failed to deserialize document in collection: {err}");
                    return Err(Error::DeserializationError {
                        message: "Failed to deserialize document in collection".to_string(),
                    });
                }
            };

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
    #[instrument(
        name = "Save cookie",
        level = "debug",
        target = "aj_studying",
        skip(self, user_id, cookie)
    )]
    pub async fn save_cookie(
        &self,
        user_id: User,
        cookie: Cookie<'_>,
    ) -> Result<UpdateResult, Error> {
        info!("Save cookie endpoint hit");
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

        let updated_doc = match self.collection.update_one(filter, new_doc).await {
            Ok(doc) => {
                debug!("Cookie updated");
                doc
            }
            Err(err) => {
                error!("Failed to update document in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to update document in collection".to_string(),
                });
            }
        };

        Ok(updated_doc)
    }

    #[allow(dead_code)]
    /// # Results
    ///   - Returns a `User` if the cookie is successfully found in the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to find in the collection
    /// # Panics
    ///   - If the document fails to find in the collection
    #[instrument(
        name = "Get cookie",
        level = "debug",
        target = "aj_studying",
        skip(self, cookie)
    )]
    pub async fn get_cookie(&self, cookie: Cookie<'_>) -> Result<User, Error> {
        info!("Get cookie endpoint hit");
        let filter = doc! { "cookie": cookie.value() };

        let user = match self.collection.find_one(filter).await {
            Ok(user) => {
                debug!("User found");
                user
            }
            Err(err) => {
                error!("Failed to find document in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to find document in collection".to_string(),
                });
            }
        };

        Ok(if let Some(user) = user {
            user
        } else {
            error!("Failed to find user");
            return Err(Error::DeserializationError {
                message: "Failed to find user".to_string(),
            });
        })
    }

    /// # Results
    ///   - Returns a `DeleteResult` if the cookie is successfully deleted from the collection
    /// # Errors
    ///   - Returns an `Error` if the document fails to delete from the collection
    /// # Panics
    ///   - If the document fails to delete from the collection
    #[instrument(
        name = "Delete cookie",
        level = "debug",
        target = "aj_studying",
        skip(self, cookie)
    )]
    pub async fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<DeleteResult, Error> {
        info!("Delete cookie endpoint hit");
        let filter = doc! { "cookie": cookie.value() };

        let deleted_doc = match self.collection.delete_one(filter).await {
            Ok(doc) => {
                debug!("Cookie deleted");
                doc
            }
            Err(err) => {
                error!("Failed to delete document in collection: {err}");
                return Err(Error::DeserializationError {
                    message: "Failed to delete document in collection".to_string(),
                });
            }
        };

        Ok(deleted_doc)
    }
}
