use crate::{
    db::{
        models::app::Post,
        utils,
        utils::app::{establish_connection, get_user_posts, remove_post},
    },
    routes::{
        submissions::PostConfirmation,
        utils::{
            headers::{AuthHeader, AuthLevel, Verifiable},
            misc::{db_err_to_status, Sections, UuidField},
        },
    },
};
use rocket::{
    form::{Form, Strict},
    http::Status,
    serde::json::{json, Json, Value},
};

#[get("/sections/<section>?<id>", rank = 1)]
pub async fn get_post(section: Sections, id: UuidField) -> Result<Json<Post>, Status> {
    let mut conn = establish_connection();

    let post = utils::app::get_post(&mut conn, section, id.to_string()).map_err(db_err_to_status)?;

    Ok(Json(post))
}

#[get("/sections/<section>?<author>", rank = 2)]
pub async fn get_author_posts(section: Sections, author: UuidField) -> Result<Json<Vec<Post>>, Status> {
    let mut conn = establish_connection();

    let posts = get_user_posts(&mut conn, section, author.to_string()).map_err(|_| Status::InternalServerError)?;

    Ok(Json(posts))
}

#[delete("/sections/<section>", data = "<post>")]
pub async fn delete_post(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Sections,
    post: Form<Strict<PostConfirmation>>,
) -> Result<Value, Status> {
    let _c = auth_header.verify()?;

    let id = post.id.to_string();

    let mut conn = establish_connection();

    remove_post(&mut conn, section, id.clone()).map_err(db_err_to_status)?;

    Ok(json!({ "id": id }))
}
