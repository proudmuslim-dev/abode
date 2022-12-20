use crate::{
    db::{
        models::app::Post,
        utils,
        utils::app::{establish_connection, remove_post},
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

#[get("/sections/<section>?<id>")]
pub async fn get_post(section: Sections, id: UuidField) -> Result<Json<Post>, Status> {
    let mut conn = establish_connection();

    let post = utils::app::get_post(&mut conn, section, id.to_string()).map_err(db_err_to_status)?;

    Ok(Json(post))
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
