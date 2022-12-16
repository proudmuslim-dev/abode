#[macro_use]
extern crate rocket;

use abode_backend::routes::{
    posts::delete_post,
    sections::{section, sections},
    sign_in::sign_in,
    sign_up::sign_up,
    submissions::{confirm_submission, new_submission, reject_submission},
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
            new_submission,
            confirm_submission,
            reject_submission,
            delete_post,
            sign_in,
            sign_up
        ],
    )
}
