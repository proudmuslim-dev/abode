use crate::utils::{BrowsePost, Category, BACKEND_URL, REQWEST_CLIENT};
use rocket::http::Status;
use rocket_dyn_templates::{context, Template};

#[get("/sign-in")]
pub async fn sign_in_page() -> Template {
    Template::render("login", context! {})
}

#[get("/browse")]
pub async fn browse() -> Template {
    Template::render("browse_menu", context! { categories: Category::ALL })
}

#[get("/browse/<category>")]
pub async fn browse_category(category: Category) -> Result<Template, Status> {
    let posts = REQWEST_CLIENT
        .get(format!(
            "http://{}/posts/{}",
            BACKEND_URL.as_str(),
            category.to_string().to_ascii_lowercase()
        ))
        .send()
        .await
        .map_err(|_| Status::InternalServerError)?
        .json::<Vec<BrowsePost>>()
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render("browse", context! { posts }))
}

#[get("/about")]
pub async fn about() -> Template {
    Template::render("about", context! {})
}

#[derive(Serialize)]
pub struct Post<'a> {
    excerpt: &'a str,
    citation: &'a str,
}
