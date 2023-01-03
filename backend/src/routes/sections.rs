use crate::{
    db::{
        prisma::{pending_post, post, Category},
        util::{get_section_pending_posts, get_section_posts},
    },
    routes::utils::headers::{AuthHeader, AuthLevel, Verifiable},
};
use rocket::{
    http::Status,
    serde::json::{json, Json, Value},
};

#[get("/sections")]
pub async fn sections() -> Value {
    // TODO: Move this list to an impl block
    json!([
        Category::Islamism,
        Category::Modernity,
        Category::Secularism,
        Category::Feminism
    ])
}

#[get("/sections/<section>", rank = 3)]
pub async fn section(section: Category) -> Result<Json<Vec<post::Data>>, Status> {
    Ok(Json(
        get_section_posts(section)
            .await
            .map_err(|_| Status::InternalServerError)?,
    ))
}

#[get("/sections/<section>/pending")]
pub async fn section_pending(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Category,
) -> Result<Json<Vec<pending_post::Data>>, Status> {
    let _c = auth_header.verify()?;

    Ok(Json(
        get_section_pending_posts(section)
            .await
            .map_err(|_| Status::InternalServerError)?,
    ))
}
