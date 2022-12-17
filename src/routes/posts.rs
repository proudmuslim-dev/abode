use crate::{
    db::utils::app::{establish_connection, remove_post},
    routes::{
        submissions::PostConfirmation,
        util::{db_err_to_status, AuthHeader, AuthLevel, Sections, Verifiable},
    },
};

use rocket::{
    form::{Form, Strict},
    http::Status,
    serde::json::{json, Value},
};
use sanitizer::Sanitize;
use validator::Validate;

#[delete("/sections/<section>", data = "<post>")]
pub async fn delete_post(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Sections,
    mut post: Form<Strict<PostConfirmation>>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    post.validate().map_err(|_| Status::BadRequest)?;
    post.sanitize();

    let mut conn = establish_connection();

    remove_post(&mut conn, section, post.id.clone()).map_err(db_err_to_status)?;

    Ok(json!({ "id": post.id }))
}
