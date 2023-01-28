use crate::{
    db::{
        prisma::{pending_post, Category},
        util::{
            confirm_pending_post, create_pending_post, get_pending_post, get_section_pending_posts,
            get_user_pending_posts, get_user_pending_posts_in_section, reject_pending_post,
        },
    },
    routes::utils::{
        headers::{AuthHeader, AuthLevel, Verifiable},
        misc::{PaginationFields, UuidField},
        responses::Notification,
    },
};
use ammonia::clean;
use prisma_client_rust::QueryError;
use pulldown_cmark::{html, Parser};
use rocket::{
    form::{Form, Strict},
    http::Status,
    response::Responder,
    serde::json::Json,
    Request, Response,
};
use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    db::{
        prisma::pending_image,
        util::{create_pending_image, get_pending_post_by_id},
    },
    routes::utils::misc::ImageField,
};
use uuid::Uuid;

#[get("/submissions/<section>?<id>", rank = 1)]
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

#[get("/submissions/<section>?<pagination..>")]
pub async fn get_section_submissions(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    pagination: PaginationFields,
) -> Result<Json<Vec<pending_post::Data>>, Status> {
    let _c = auth_header.verify()?;

    Ok(Json(
        get_section_pending_posts(section, pagination)
            .await
            .map_err(|_| Status::InternalServerError)?,
    ))
}

#[get("/submissions?<author>&<pagination..>")]
pub async fn get_author_submissions(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    author: UuidField,
    pagination: PaginationFields,
) -> Result<Json<Vec<pending_post::Data>>, Status> {
    let _c = auth_header.verify()?;

    let posts = get_user_pending_posts(author.to_string(), pagination)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(posts))
}

#[get("/submissions/<section>?<author>&<pagination..>", rank = 2)]
pub async fn get_author_section_submissions(
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
#[post("/submissions/<section>/submit", data = "<post>")]
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

#[post("/submissions/images", data = "<form>")]
pub async fn new_submission_image(
    auth_header: AuthHeader,
    form: Form<Strict<ImageSubmission>>,
) -> Result<Json<pending_image::Data>, Status> {
    let c = auth_header.verify()?;

    match get_pending_post_by_id(form.post_id.to_string())
        .await
        .map_err(|_| Status::InternalServerError)?
    {
        Some(pending_post::Data { author_id, .. }) => {
            if !c.admin {
                let author_id = Uuid::from_str(author_id.as_str()).map_err(|_| Status::InternalServerError)?;

                if author_id != c.sub {
                    return Err(Status::Unauthorized);
                }
            }
        }
        None => return Err(Status::NotFound),
    }

    let ret = {
        let width = i32::try_from(form.image.width).map_err(|_| Status::BadRequest)?;
        let height = i32::try_from(form.image.height).map_err(|_| Status::BadRequest)?;
        let path = form.image.persist().map_err(|_| Status::InternalServerError)?;

        create_pending_image(form.post_id.to_string(), path, width, height)
            .await
            .map_err(|_| Status::InternalServerError)?
    };

    Ok(Json(ret))
}

#[post("/submissions/<section>/confirm", data = "<post>")]
pub async fn confirm_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    post: Form<Strict<PostConfirmation>>,
) -> Result<Json<Notification>, Status> {
    let _c = auth_header.verify()?;

    let id = post.id.to_string();

    let ret = confirm_pending_post(section, id, post.comment.clone())
        .await
        .map_err(map_err)?;

    Ok(Json(Notification::from(ret)))
}

#[delete("/submissions/<section>/reject", data = "<rejection>")]
pub async fn reject_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    rejection: Json<PostRejection>,
) -> Result<Json<Notification>, Status> {
    let _c = auth_header.verify()?;

    let id = rejection.submission_id.to_string();

    let ret = reject_pending_post(section, id, rejection.comment.clone())
        .await
        .map_err(map_err)?;

    Ok(Json(Notification::from(ret)))
}

#[derive(FromForm)]
pub struct PostConfirmation {
    pub(crate) id: UuidField,
    pub(crate) comment: Option<String>,
}

#[derive(FromForm)]
pub struct ImageSubmission {
    pub(crate) post_id: UuidField,
    pub(crate) image: ImageField,
}

#[derive(Deserialize)]
pub struct PostRejection {
    #[serde(rename = "id")]
    pub(crate) submission_id: UuidField,
    pub(crate) comment: Option<String>,
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

fn map_err(e: QueryError) -> Status {
    match e {
        QueryError::Deserialize(serde_value::DeserializerError::Custom(err)) => {
            if err.eq("Not Found") {
                Status::NotFound
            } else {
                Status::InternalServerError
            }
        }
        _ => Status::InternalServerError,
    }
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
