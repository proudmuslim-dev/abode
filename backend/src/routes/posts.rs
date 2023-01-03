use crate::{
    db,
    db::{
        prisma::{post, Category},
        util::{get_user_posts_in_section, remove_post},
    },
    routes::{
        submissions::PostConfirmation,
        utils::{
            headers::{AuthHeader, AuthLevel, Verifiable},
            misc::UuidField,
        },
    },
};
use rocket::{
    form::{Form, Strict},
    http::Status,
    serde::json::{json, Json, Value},
};

#[get("/sections/<section>?<id>", rank = 1)]
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

#[get("/sections/<section>?<author>", rank = 2)]
pub async fn get_author_posts(section: Category, author: UuidField) -> Result<Json<Vec<post::Data>>, Status> {
    let posts = get_user_posts_in_section(section, author.to_string())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(posts))
}

#[delete("/sections/<section>", data = "<post>")]
pub async fn delete_post(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
    post: Form<Strict<PostConfirmation>>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    let id = post.id.to_string();

    remove_post(section, id.clone())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(json!({ "id": id }))
}
