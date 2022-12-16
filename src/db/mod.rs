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
