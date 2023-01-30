use rocket_dyn_templates::{context, Template};

#[get("/sign-in")]
pub async fn sign_in_page() -> Template {
    Template::render("login", context! {})
}

#[get("/browse")]
pub async fn browse() -> Template {
    let posts = vec![
        Post {
            excerpt: "Passed from code",
            citation: "Abode: main.rs",
        },
        Post {
            excerpt: "Lorem ipsum dolor sit amet",
            citation: "Shiqaq-e-dimagh p. [redacted]",
        },
    ];

    Template::render("browse", context! { posts })
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
