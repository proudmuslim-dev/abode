use crate::db::{
    models::{app::NewPost, section},
    pending::schema::{feminism, islamism, modernity, pending_posts, secularism},
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, PartialEq, Eq, Debug, Clone)]
pub struct PendingPost {
    pub id: String,
    pub author_id: String,
    pub excerpt: String,
    pub citation: String,
}

impl PendingPost {
    pub(crate) fn as_new_post(&self) -> NewPost<'_> {
        NewPost {
            id: Uuid::new_v4().to_string(),
            author_id: self.author_id.as_str(),
            excerpt: self.excerpt.as_str(),
            citation: self.citation.as_str(),
        }
    }
}

#[derive(Insertable, Clone)]
#[diesel(table_name = pending_posts)]
pub struct NewPendingPost<'a> {
    pub id: String,
    pub author_id: &'a str,
    pub excerpt: &'a str,
    pub citation: &'a str,
}

section!(PendingIslamism, islamism);
section!(PendingModernity, modernity);
section!(PendingSecularism, secularism);
section!(PendingFeminism, feminism);
