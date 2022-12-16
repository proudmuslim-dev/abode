use crate::{
    db::{models::app::Post, util::establish_connection},
    routes::util::Sections,
};

use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};

use rocket::{
    http::Status,
    serde::json::{json, Value},
};

#[get("/sections")]
pub async fn sections() -> Value {
    json!(Sections::ALL)
}

/// Returns a list of [`Post`] instances
#[get("/sections/<section>")]
pub async fn section(section: Sections) -> Result<Value, Status> {
    let mut conn = establish_connection();

    macro_rules! get_posts {
        ($name:ident) => {
            paste::paste! {
                {
                    use crate::db::schema::$name::dsl::*;

                    let post_ids: Vec<String> = match $name.select(post_id).load::<String>(&mut conn) {
                        Ok(a) => a,
                        Err(_) => return Err(Status::InternalServerError),
                    };

                    let results: Vec<QueryResult<Post>> = post_ids.into_iter().map(|s| {
                        use crate::db::schema::posts::dsl::{posts, id};

                        posts.filter(id.eq(s)).first(&mut conn)
                    }).collect();

                    let mut posts = vec![];

                    for x in results {
                        match x {
                            Ok(p) =>  posts.push(p),
                            Err(_) => return Err(Status::InternalServerError)
                        }
                    }

                    Ok(json!(posts))
                }
            }
        };
    }

    match section {
        Sections::Islamism => get_posts!(islamism),
        Sections::Modernity => get_posts!(modernity),
        Sections::Secularism => get_posts!(secularism),
        Sections::Feminism => get_posts!(feminism),
    }
}


