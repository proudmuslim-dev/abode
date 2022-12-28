use crate::{
    db::{
        models::section,
        schemas::app::{feminism, islamism, modernity, posts, secularism, users},
    },
    routes::utils::misc::Sections,
};
use chrono::NaiveDateTime;
use diesel::{Insertable, QueryResult, RunQueryDsl, SqliteConnection};
use serde::Serialize;

#[derive(Queryable, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub admin: bool,
}

#[derive(Queryable, Serialize, Clone)]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub excerpt: String,
    pub citation: String,
    pub creation: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: String,
    pub username: &'a str,
    pub password: &'a str,
    pub admin: bool,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub id: String,
    pub author_id: &'a str,
    pub excerpt: &'a str,
    pub citation: &'a str,
}

impl<'a> NewPost<'a> {
    /// Inserts the post into the posts table
    pub(crate) fn insert(&self, conn: &mut SqliteConnection, section: Sections) -> QueryResult<()> {
        // TODO: Duplicate UUID handling
        self.insert_into(posts::table).execute(conn)?;

        macro_rules! insert_into_section {
            ($($variant:ident => $section:ident),*) => {
                paste::paste! {
                    match section {
                        $(
                            Sections::$variant => {
                                diesel::insert_into($section::table)
                                    .values([<New $variant Entry>] {
                                        post_id: self.id.as_str(),
                                    })
                                    .execute(conn)?;
                            }
                        )*
                    }
                }
            };
        }

        insert_into_section!(
            Islamism => islamism,
            Modernity => modernity,
            Secularism => secularism,
            Feminism => feminism
        );

        Ok(())
    }
}

section!(Islamism, islamism);
section!(Modernity, modernity);
section!(Secularism, secularism);
section!(Feminism, feminism);
