use crate::{
    db,
    db::{pending, pending::util::create_pending_post},
    routes::util::{AuthHeader, Sections},
};

use crate::routes::util::{AuthLevel, Verifiable};
use ammonia::clean;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use pulldown_cmark::{html, Parser};
use rocket::{
    form::{Form, Strict},
    http::Status,
    serde::json::{json, Value},
};
use sanitizer::prelude::*;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;
use crate::db::pending::util::remove_pending_post;

// TODO: 201 response
/// Returns the new post's [`Uuid`]
#[post("/sections/<section>/submit", data = "<post>")]
pub async fn new_submission(
    auth_header: AuthHeader,
    section: Sections,
    mut post: Form<Strict<PostSubmission>>,
) -> Result<Value, Status> {
    let c = auth_header.verify()?;

    post.validate().map_err(|_| Status::BadRequest)?;
    post.sanitize();

    let mut id = Uuid::new_v4();
    let mut conn = pending::util::establish_connection();

    let excerpt = post.excerpt.as_str();
    let citation = post.citation.as_str();

    match create_pending_post(&mut conn, section, id, c.sub, excerpt, citation) {
        Ok(_) => {}
        Err(e) => match e {
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                id = Uuid::new_v4();

                if create_pending_post(&mut conn, section, id, c.sub, excerpt, citation).is_err() {
                    return Err(Status::InternalServerError);
                }
            }
            _ => return Err(Status::InternalServerError),
        },
    }

    Ok(json!({ "id": id }))
}

#[post("/sections/<section>/confirm", data = "<post>")]
pub async fn confirm_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Sections,
    mut post: Form<Strict<PostConfirmation>>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    post.sanitize();
    post.validate().map_err(|_| Status::BadRequest)?;

    let mut conn = pending::util::establish_connection();

    let pending_post = pending::util::get_and_remove_pending_post(&mut conn, section, post.id.clone()).map_err(db_err_to_status)?;

    let new_post = pending_post.as_new_post();
    let id = new_post.id.clone();

    // TODO: Remove entry from pending db

    let mut conn = db::util::establish_connection();

    new_post
        .insert(section, &mut conn)
        .map_err(|_| Status::InternalServerError)?;

    Ok(json!({ "id": id }))
}

// TODO: Work out system for notifying user of rejection
#[delete("/sections/<section>/reject", data = "<post>")]
pub async fn reject_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Sections,
    mut post: Form<Strict<PostConfirmation>>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    post.sanitize();
    post.validate().map_err(|_| Status::BadRequest)?;

    let mut conn = pending::util::establish_connection();

    remove_pending_post(&mut conn, section, post.id.clone()).map_err(db_err_to_status)?;

    Ok(json!({ "id": post.id }))
}

fn db_err_to_status(e: DieselError) -> Status {
    match e {
        DieselError::NotFound => Status::NotFound,
        _ => Status::InternalServerError
    }
}

#[derive(FromForm, Deserialize, Validate, Sanitize)]
pub struct PostConfirmation {
    // The length of a UUID v4 with dashes.
    #[sanitize(custom(sanitize_uuid))]
    #[validate(length(equal = 36))]
    id: String,
}

fn sanitize_uuid(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace() && c.is_ascii()).collect()
}

#[derive(FromForm, Deserialize, Validate, Sanitize)]
pub struct PostSubmission {
    #[sanitize(trim, custom(convert_and_sanitize))]
    #[validate(length(min = 10))]
    pub excerpt: String,
    #[sanitize(trim, custom(convert_and_sanitize))]
    #[validate(length(min = 10))]
    pub citation: String,
}

fn convert_and_sanitize(s: &str) -> String {
    let md_parse = Parser::new(s);
    let mut unsafe_html = String::new();
    html::push_html(&mut unsafe_html, md_parse);

    clean(unsafe_html.as_str())
}
