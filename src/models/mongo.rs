use std::env;

use mongodb::{
    bson::{self, doc, extjson::de::Error, oid::ObjectId},
    results::InsertOneResult,
    Client, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub sign_up_date: String,
    pub username: String,
}

#[allow(clippy::module_name_repetitions)]
pub struct MongoRepo {
    collection: Collection<User>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv::dotenv().ok();
        let Ok(uri) = env::var("MONGOURI") else {
            panic!("MONGOURI not found in .env")
        };
        let client = Client::with_uri_str(&uri).await.unwrap();
        let db = client.database("study_pwa");
        let collection: Collection<User> = db.collection("users");

        Self { collection }
    }

    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: None,
            name: new_user.name,
            sign_up_date: new_user.sign_up_date,
            username: new_user.username,
        };

        let user = self
            .collection
            .insert_one(new_doc)
            .await
            .expect("Failed to insert document into collection");

        Ok(user)
    }

    pub async fn get_user(&self, id: &str) -> Result<User, Error> {
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": object_id };

        let user = self
            .collection
            .find_one(filter)
            .await
            .expect("Failed to find document in collection");

        Ok(user.unwrap())
    }
}
