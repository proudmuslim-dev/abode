use crate::{
    db::{
        get_posts,
        models::{app::Post, pending::PendingPost},
        utils,
    },
    routes::utils::{
        headers::{AuthHeader, AuthLevel, Verifiable},
        misc::Sections,
    },
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{
    http::Status,
    serde::json::{json, Json, Value},
};

#[get("/sections")]
pub async fn sections() -> Value {
    json!(Sections::ALL)
}

// TODO: Remove the need for the `append_conn` macros by creating helpers
// + add helpers to tests

#[get("/sections/<section>", rank = 3)]
pub async fn section(section: Sections) -> Result<Json<Vec<Post>>, Status> {
    let mut conn = utils::app::establish_connection();

    macro_rules! append_conn {
        ($($variant:ty => $section:ident),*) => {
            paste::paste! {
                match section {
                    $(
                        Sections::$variant => get_posts!($section, conn),
                    )*
                }
            }
        }
    }

    append_conn!(
        Islamism => islamism,
        Modernity => modernity,
        Secularism => secularism,
        Feminism => feminism
    )
}

#[get("/sections/<section>/pending")]
pub async fn section_pending(
    auth_header: AuthHeader<{ AuthLevel::Admin }>,
    section: Sections,
) -> Result<Json<Vec<PendingPost>>, Status> {
    let _c = auth_header.verify()?;

    let mut conn = utils::pending::establish_connection();

    macro_rules! append_conn {
        ($($variant:ty => $section:ident),*) => {
            paste::paste! {
                match section {
                    $(
                        Sections::$variant => get_posts!($section, conn, pending),
                    )*
                }
            }
        }
    }

    append_conn!(
        Islamism => islamism,
        Modernity => modernity,
        Secularism => secularism,
        Feminism => feminism
    )
}
