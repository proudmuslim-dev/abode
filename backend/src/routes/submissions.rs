use crate::{
    db::{
        prisma::{pending_post, post, Category},
        util::{
            create_pending_post, create_post, get_pending_post, get_user_pending_posts_in_section, remove_pending_post,
        },
    },
    routes::utils::{
        headers::{AuthHeader, AuthLevel, Verifiable},
        misc::UuidField,
    },
};
use crate::routes::utils::misc::PaginationFields;
use ammonia::clean;
use pulldown_cmark::{html, Parser};
use rocket::{
    form::{Form, Strict},
    http::Status,
    response::Responder,
    serde::json::{json, Json, Value},
    Request, Response,
};
use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[get("/sections/<section>/pending?<id>", rank = 1)]
pub async fn get_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    id: UuidField,
) -> Result<Json<pending_post::Data>, Status> {
    let _c = auth_header.verify()?;

    let ret = get_pending_post(section, id.to_string())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if let Some(ret) = ret {
        Ok(Json(ret))
    } else {
        Err(Status::NotFound)
    }
}

// TODO: Route for username instead of UUID
#[get("/sections/<section>/pending?<author>&<pagination..>", rank = 2)]
pub async fn get_author_submissions(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    author: UuidField,
    pagination: PaginationFields,
) -> Result<Json<Vec<pending_post::Data>>, Status> {
    let _c = auth_header.verify()?;

    let posts = get_user_pending_posts_in_section(section, author.to_string(), pagination)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(posts))
}

/// Returns the new post's [`Uuid`]
#[post("/sections/<section>/submit", data = "<post>")]
pub async fn new_submission(
    auth_header: AuthHeader,
    section: Category,
    mut post: Form<Strict<PostSubmission>>,
) -> Result<PostSubmissionResponse, Status> {
    let c = auth_header.verify()?;

    post.sanitize();

    let id = Uuid::new_v4();

    create_pending_post(section, id, c.sub, post.excerpt.clone(), post.citation.clone())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(PostSubmissionResponse { id: id.to_string() })
}

#[post("/sections/<section>/confirm", data = "<post>")]
pub async fn confirm_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    post: Form<Strict<PostConfirmation>>,
) -> Result<Json<post::Data>, Status> {
    let _c = auth_header.verify()?;

    let id = post.id.to_string();

    let pending_post = {
        let p = get_pending_post(section, id.clone())
            .await
            .map_err(|_| Status::InternalServerError)?;

        if p.is_some() {
            remove_pending_post(section, id)
                .await
                .map_err(|_| Status::InternalServerError)?;

            p.unwrap()
        } else {
            return Err(Status::NotFound);
        }
    };

    let id = Uuid::new_v4();

    let confirmed = create_post(
        section,
        id,
        Uuid::from_str(pending_post.author_id.as_str()).unwrap(),
        pending_post.excerpt,
        pending_post.citation,
        pending_post.submitted_at,
    )
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(confirmed))
}

// TODO: Work out system for notifying user of rejection
#[delete("/sections/<section>/reject", data = "<post>")]
pub async fn reject_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    post: Form<Strict<PostConfirmation>>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    let id = post.id.to_string();

    remove_pending_post(section, id.clone())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(json!({ "id": id }))
}

#[derive(FromForm)]
pub struct PostConfirmation {
    pub(crate) id: UuidField,
}

#[derive(FromForm, Deserialize, Sanitize)]
pub struct PostSubmission {
    #[sanitize(trim, custom(convert_and_sanitize))]
    #[field(validate = len(10..1500))]
    pub excerpt: String,
    #[sanitize(trim, custom(convert_and_sanitize))]
    #[field(validate = len(10..200))]
    pub citation: String,
}

fn convert_and_sanitize(s: &str) -> String {
    let md_parse = Parser::new(s);
    let mut unsafe_html = String::new();
    html::push_html(&mut unsafe_html, md_parse);

    clean(unsafe_html.as_str())
}

#[derive(Serialize, Deserialize)]
pub struct PostSubmissionResponse {
    pub id: String,
}

impl<'r> Responder<'r, 'r> for PostSubmissionResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(&self).respond_to(request)?)
            .status(Status::Created)
            .ok()
    }
}
