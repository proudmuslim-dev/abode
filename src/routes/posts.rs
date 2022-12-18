use crate::{
    db::utils::app::{establish_connection, remove_post},
    routes::{
        submissions::PostConfirmation,
        utils::{
            headers::{AuthHeader, AuthLevel, Verifiable},
            misc::{db_err_to_status, Sections},
        },
    },
};
use rocket::{
    form::{Form, Strict},
    http::Status,
    serde::json::{json, Value},
};

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
