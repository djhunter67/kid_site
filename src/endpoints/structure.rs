#[derive(serde::Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct Registration {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}
