use std::fmt::{self, Display, Formatter};

use serde::Serialize;

pub mod tokens;

#[derive(Serialize)]
pub enum Types {
    UserIdKey,
    UserEmailKey,
}

impl Display for Types {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::UserIdKey => write!(f, "user_id"),
            Self::UserEmailKey => write!(f, "email"),
        }
    }
}

impl From<String> for Types {
    fn from(s: String) -> Self {
        match s.as_str() {
            "user_id" => Self::UserIdKey,
            "email" => Self::UserEmailKey,
            _ => panic!("Invalid type"),
        }
    }
}

impl Into<String> for Types {
    fn into(self) -> String {
        match self {
            Self::UserIdKey => "user_id".to_string(),
            Self::UserEmailKey => "email".to_string(),
        }
    }
}

// pub const USER_ID_KEY: &str = "user_id";

// pub const USER_EMAIL_KEY: &str = "email";
