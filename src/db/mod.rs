pub mod models;
pub mod schemas;
pub mod utils;

macro_rules! delete_from_section {
    ($posts:ident { $post_section:ident, $post_id:ident, $conn:ident }, $($variant:ident => $section:ident),*) => {
        paste::paste! {
            use schema::$posts::{dsl::$posts, id};

            match $post_section {
                $(
                    Sections::$variant => {
                        use schema::$section::{dsl::$section, post_id as pid};

                        diesel::delete($section.filter(pid.eq($post_id.clone()))).execute($conn)?;

                        diesel::delete($posts.filter(id.eq($post_id))).execute($conn)?;
                    }
                )*
            }
        }
    };
}

macro_rules! get_posts {
    ($section:ident, $conn:ident, pending) => {
        get_posts!(pending, pending_posts { crate::db::models::pending::PendingPost }, $section, $conn)
    };
    ($section:ident, $conn:ident) => {
        get_posts!(app, posts { crate::db::models::app::Post }, $section, $conn)
    };
    ($db:ident, $posts:ident { $post_type:ty }, $section:ident, $conn:ident) => {
        paste::paste! {
            {
                use crate::db::schemas::$db::$section::dsl::*;

                let post_ids: Vec<String> = match $section.select(post_id).load::<String>(&mut $conn) {
                    Ok(a) => a,
                    Err(_) => return Err(Status::InternalServerError),
                };

                let results: Vec<QueryResult<$post_type>> = post_ids.into_iter().map(|s| {
                    use crate::db::schemas::$db::$posts::dsl::{$posts, id};

                    $posts.filter(id.eq(s)).first(&mut $conn)
                }).collect();

                let mut posts = vec![];

                for x in results {
                    match x {
                        Ok(p) =>  posts.push(p),
                        Err(_) => return Err(Status::InternalServerError)
                    }
                }

                Ok(Json(posts))
            }
        }
    };
}

pub(crate) use delete_from_section;
pub(crate) use get_posts;

#[cfg(test)]
mod tests {
    use super::{models::pending::PendingPost, *};
    use crate::routes::utils::misc::Sections;
    use std::error::Error;
    use uuid::Uuid;

    #[test]
    fn test_util() -> Result<(), Box<dyn Error>> {
        let mut conn = utils::app::establish_connection();
        let uid = Uuid::new_v4();
        let username = uid.to_string();
        let pass = "testing".to_owned();

        // Needed to ensure that the create_user function doesn't panic due to the fact
        // that it generates a JWT.
        std::env::set_var("ENCODING_SECRET", "jivcwtuR5QIHvAuNMnK7rrtB");
        utils::app::create_user(&mut conn, uid, username.as_str(), pass.as_str())?;

        utils::app::get_user(&mut conn, username.as_str()).ok_or("Failed to get user!")?;
        utils::app::get_user_uid(&mut conn, uid).ok_or("Failed to get user via UUID!")?;

        // pid would be too confusing lol
        let pending_post_id = Uuid::new_v4();
        let post = PendingPost {
            id: pending_post_id.to_string(),
            author_id: uid.to_string(),
            excerpt: "We do a little trolling".to_owned(),
            citation: "Shiqaq-e-dimagh p. [redacted]".to_owned(),
        };
        let section = Sections::Islamism;

        conn = utils::pending::establish_connection();

        utils::pending::create_pending_post(
            &mut conn,
            section,
            pending_post_id,
            uid,
            post.excerpt.as_str(),
            post.citation.as_str(),
        )?;

        let _post = utils::pending::get_and_remove_pending_post(&mut conn, section, pending_post_id.to_string())?;

        assert_eq!(post, _post);

        conn = utils::app::establish_connection();

        let np = post.as_new_post();
        np.insert(&mut conn, section)?;

        utils::app::remove_post(&mut conn, section, np.id)?;

        // TODO: Delete user helper + route

        Ok(())
    }
}
