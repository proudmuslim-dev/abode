use crate::{
    db::{
        prisma::{notification, pending_post, post, Category},
        util::{
            create_pending_post, create_post, get_pending_post, get_section_pending_posts, get_user_pending_posts,
            get_user_pending_posts_in_section, reject_pending_post, remove_pending_post,
        },
    },
    routes::utils::{
        headers::{AuthHeader, AuthLevel, Verifiable},
        misc::{PaginationFields, UuidField},
    },
};
use ammonia::clean;
use prisma_client_rust::QueryError;
use pulldown_cmark::{html, Parser};
use rocket::{
    form::{Form, Strict},
    http::Status,
    response::Responder,
    serde::json::{Json},
    Request, Response,
};
use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
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

#[post("/submissions/<section>/confirm", data = "<post>")]
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

// TODO: Notify user of approval as well
#[delete("/submissions/<section>/reject", data = "<rejection>")]
pub async fn reject_submission(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    rejection: Form<Strict<PostRejection>>,
) -> Result<Json<notification::Data>, Status> {
    let _c = auth_header.verify()?;

    let id = rejection.submission_id.to_string();

    let ret = reject_pending_post(section, id, rejection.reason.clone())
        .await
        .map_err(|e| match e {
            QueryError::Deserialize(serde_value::DeserializerError::Custom(err)) => {
                if err.eq("Not Found") {
                    Status::NotFound
                } else {
                    Status::InternalServerError
                }
            }
            _ => Status::InternalServerError,
        })?;

    Ok(Json(ret))
}

#[derive(FromForm)]
pub struct PostConfirmation {
    pub(crate) id: UuidField,
}

#[derive(FromForm)]
pub struct PostRejection {
    #[field(name = uncased("id"))]
    pub(crate) submission_id: UuidField,
    pub(crate) reason: String,
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
