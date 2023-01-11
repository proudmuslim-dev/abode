use crate::{
    db,
    db::{
        prisma::{post, Category},
        util::{get_user_posts, get_user_posts_in_section, remove_post},
    },
    routes::utils::{
        headers::{AuthHeader, AuthLevel, Verifiable},
        misc::{PaginationFields, UuidField},
    },
};
use rocket::{
    http::Status,
    serde::json::{json, Json, Value},
};
use uuid::Uuid;

#[get("/posts/<section>?<id>", rank = 1)]
pub async fn get_post(section: Category, id: UuidField) -> Result<Json<post::Data>, Status> {
    let post = db::util::get_post(section, id.to_string())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if let Some(post) = post {
        Ok(Json(post))
    } else {
        Err(Status::NotFound)
    }
}

#[get("/posts/<section>?<pagination..>", rank = 3)]
pub async fn get_section_posts(
    section: Category,
    pagination: PaginationFields,
) -> Result<Json<Vec<post::Data>>, Status> {
    Ok(Json(
        db::util::get_section_posts(section, pagination)
            .await
            .map_err(|_| Status::InternalServerError)?,
    ))
}

#[get("/posts?<author>&<pagination..>")]
pub async fn get_author_posts(
    author: UuidField,
    pagination: PaginationFields,
) -> Result<Json<Vec<post::Data>>, Status> {
    let posts = get_user_posts(author.to_string(), pagination)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(posts))
}

#[get("/posts/<section>?<author>&<pagination..>", rank = 2)]
pub async fn get_author_section_posts(
    section: Category,
    author: UuidField,
    pagination: PaginationFields,
) -> Result<Json<Vec<post::Data>>, Status> {
    let posts = get_user_posts_in_section(section, author.to_string(), pagination)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(posts))
}

#[delete("/posts/<section>", data = "<post>")]
pub async fn delete_post(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    post: Json<PostDeletion>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    let id = post.id.to_string();

    remove_post(section, id.clone())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(json!({ "id": id }))
}

#[derive(Deserialize)]
pub struct PostDeletion {
    pub(crate) id: Uuid,
}
