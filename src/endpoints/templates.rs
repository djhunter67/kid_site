use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginPage<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterPage<'a> {
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "errors.html")]
pub struct ErrorPage<'a> {
    pub title: &'a str,
    pub code: u16,
    pub error: &'a str,
    pub message: &'a str,
}
