#[macro_use]
extern crate rocket;

use backend::routes::{
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
            sign_in,
            sign_up,
            section,
            sections,
            section_pending,
            get_author_posts,
            get_post,
            delete_post,
            get_submission,
            get_author_submissions,
            new_submission,
            confirm_submission,
            reject_submission
        ],
    )
}
