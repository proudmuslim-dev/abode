pub mod models;
pub mod schemas;
pub mod utils;

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
                use crate::db::schemas::$db::$posts::{dsl::$posts, id, creation};

                let post_ids: Vec<String> = $section.select(post_id).load::<String>(&mut $conn).map_err(|_| Status::InternalServerError)?;

                let posts_vec: Vec<$post_type> = $posts.filter(id.eq_any(post_ids)).order(creation.desc()).load(&mut $conn).map_err(|_| Status::InternalServerError)?;

                Ok(Json(posts_vec))
            }
        }
    };
}

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

macro_rules! get_user_posts {
    ($db:ident, $posts:ident { $post_type:ty }, $sec:ident, $conn:ident, $uid:ident) => {
        paste::paste! {
            {
                use crate::db::schemas::$db::$posts::{author_id, dsl::$posts, id, creation};
                use crate::routes::utils::misc::Sections;

                let matches: Vec<String>;

                get_section_posts!(
                    $db,
                    $sec,
                    matches,
                    $conn,
                    Islamism => islamism,
                    Modernity => modernity,
                    Secularism => secularism,
                    Feminism => feminism
                );

                let user_posts: Vec<$post_type> = $posts
                    .filter(author_id.eq($uid.clone()))
                    .filter(id.eq_any(matches))
                    .order(creation.desc())
                    .load($conn)?;

                user_posts
            }
        }
    };
}

macro_rules! get_section_posts {
    ($db:ident, $sec:ident, $matches:ident, $conn:ident, $($variant:ident => $section:ident),*) => {
        match $sec {
            $(
                Sections::$variant => {
                    use crate::db::schemas::$db::$section::{dsl::$section, post_id};

                    $matches = $section.select(post_id).load::<String>($conn)?;

                    if $matches.is_empty() {
                        return Ok(vec![]);
                    }
                }
            )*
        }
    }
}

pub(crate) use delete_from_section;
pub(crate) use get_posts;
pub(crate) use get_section_posts;
pub(crate) use get_user_posts;

#[cfg(test)]
mod tests {
    use super::{models::pending::PendingPost, *};
    use crate::routes::utils::misc::Sections;
    use chrono::NaiveDateTime;
    use color_eyre::eyre::{Context, ContextCompat};
    use std::error::Error;
    use uuid::Uuid;

    #[test]
    fn test_util() -> Result<(), Box<dyn Error>> {
        let mut conn = utils::app::establish_connection();
        let uid = Uuid::new_v4();
        let username = uid.to_string();
        let pass = "testing".to_owned();

        let excerpt = "We do a little trolling".to_owned();
        let citation = "Shiqaq-e-dimagh p. [redacted]".to_owned();

        // Needed to ensure that the create_user function doesn't panic due to the fact
        // that it generates a JWT.
        std::env::set_var("ENCODING_SECRET", "jivcwtuR5QIHvAuNMnK7rrtB");
        utils::app::create_user(&mut conn, uid, username.as_str(), pass.as_str())?;

        utils::app::get_user(&mut conn, username.as_str()).context("Failed to get user!")?;
        utils::app::get_user_uid(&mut conn, uid).context("Failed to get user via UUID!")?;

        // pid would be too confusing lol
        let pending_post_id = Uuid::new_v4();
        // TODO: Fix this
        let mut post = PendingPost {
            id: pending_post_id.to_string(),
            author_id: uid.to_string(),
            excerpt: excerpt.clone(),
            citation: citation.clone(),
            creation: NaiveDateTime::default(),
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

        let matches = utils::pending::get_user_pending_posts(&mut conn, section, uid.to_string())
            .context("Failed to get pending posts")?;

        assert_eq!(matches[0].excerpt.as_str(), excerpt.as_str());
        assert_eq!(matches[0].citation.as_str(), citation.as_str());

        let _post = utils::pending::get_and_remove_pending_post(&mut conn, section, pending_post_id.to_string())?;
        post.creation = _post.creation;

        assert_eq!(post, _post);

        conn = utils::app::establish_connection();

        let np = post.as_new_post();
        np.insert(&mut conn, section)?;

        let matches = utils::app::get_user_posts(&mut conn, section, uid.to_string()).context("Failed to get posts")?;

        assert_eq!(matches[0].excerpt.as_str(), excerpt.as_str());
        assert_eq!(matches[0].citation.as_str(), citation.as_str());

        utils::app::remove_post(&mut conn, section, np.id)?;

        // TODO: Delete user helper + route

        Ok(())
    }
}
