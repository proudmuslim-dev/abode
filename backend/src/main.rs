#[macro_use]
extern crate rocket;

use backend::routes::{
    notifications::{delete_notification, get_notifications, patch_notifications},
    posts::{delete_post, get_author_posts, get_author_section_posts, get_post, get_section_posts},
    sections::sections,
    sign_in::sign_in,
    sign_up::sign_up,
    submissions::{
        confirm_submission, get_author_section_submissions, get_author_submissions, get_section_submissions,
        get_submission, new_submission, new_submission_image, reject_submission,
    },
};
use rocket::fs::{relative, FileServer};

/// Only here as a sanity check, will be removed by v1.0 inshaAllah
#[get("/")]
async fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                hello,
                sign_in,
                sign_up,
                get_notifications,
                patch_notifications,
                delete_notification,
                sections,
                get_section_posts,
                get_author_posts,
                get_author_section_posts,
                get_post,
                delete_post,
                get_submission,
                get_author_submissions,
                get_section_submissions,
                get_author_section_submissions,
                new_submission,
                new_submission_image,
                confirm_submission,
                reject_submission
            ],
        )
        .mount("/assets", FileServer::from(relative!("assets")))
}
