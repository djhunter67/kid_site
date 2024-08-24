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
