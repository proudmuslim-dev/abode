#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde;

mod pages;

use crate::pages::*;
use rocket_dyn_templates::Template;
use rocket::fs::{FileServer, relative};

#[launch]
async fn launch() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![sign_in_page, browse, about])
        .mount("/", FileServer::from(relative!("assets")))
}

