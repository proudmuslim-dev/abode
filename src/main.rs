#[macro_use]
extern crate rocket;

use abode_backend::routes::{
    posts::{delete_post, get_author_posts, get_post},
    sections::{section, section_pending, sections},
    sign_in::sign_in,
    sign_up::sign_up,
    submissions::{confirm_submission, get_author_submissions, get_submission, new_submission, reject_submission},
};

#[get("/")]
async fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            hello,
            sections,
            section,
            section_pending,
            new_submission,
            confirm_submission,
            reject_submission,
            get_submission,
            get_author_submissions,
            delete_post,
            get_post,
            get_author_posts,
            sign_in,
            sign_up
        ],
    )
}
