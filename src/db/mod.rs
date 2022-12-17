pub mod models;
pub mod pending;
pub mod schema;
pub mod util;

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

pub(crate) use delete_from_section;

#[cfg(test)]
mod tests {
    use super::{models::pending::PendingPost, *};
    use crate::routes::util::Sections;
    use std::error::Error;
    use uuid::Uuid;

    #[test]
    fn test_util() -> Result<(), Box<dyn Error>> {
        let mut conn = util::establish_connection();
        let uid = Uuid::new_v4();
        let username = uid.to_string();
        let pass = "testing".to_owned();

        // Needed to ensure that the create_user function doesn't panic due to the fact
        // that it generates a JWT.
        std::env::set_var("ENCODING_SECRET", "jivcwtuR5QIHvAuNMnK7rrtB");
        util::create_user(&mut conn, uid, username.as_str(), pass.as_str())?;

        util::get_user(&mut conn, username.as_str()).ok_or("Failed to get user!")?;
        util::get_user_uid(&mut conn, uid).ok_or("Failed to get user via UUID!")?;

        // pid would be too confusing lol
        let pending_post_id = Uuid::new_v4();
        let post = PendingPost {
            id: pending_post_id.to_string(),
            author_id: uid.to_string(),
            excerpt: "We do a little trolling".to_owned(),
            citation: "Shiqaq-e-dimagh p. [redacted]".to_owned(),
        };
        let section = Sections::Islamism;

        conn = pending::util::establish_connection();

        pending::util::create_pending_post(
            &mut conn,
            section,
            pending_post_id,
            uid,
            post.excerpt.as_str(),
            post.citation.as_str(),
        )?;

        let _post = pending::util::get_and_remove_pending_post(&mut conn, section, pending_post_id.to_string())?;

        assert_eq!(post, _post);

        conn = util::establish_connection();

        let np = post.as_new_post();
        np.insert(&mut conn, section)?;

        util::remove_post(&mut conn, section, np.id)?;

        // TODO: Delete user helper + route

        Ok(())
    }
}
