use crate::{
    db::{
        prisma,
        prisma::{
            pending_post, post, user,
            user::{SetParam, UniqueWhereParam},
            Category, PrismaClient,
        },
    },
    routes::utils::jwt::generate_api_token,
};
use async_once::AsyncOnce;
use chrono::DateTime;
use color_eyre::eyre::Context;
use lazy_static::lazy_static;
use prisma_client_rust::QueryError;
use rocket::request::FromParam;
use uuid::Uuid;

lazy_static! {
    pub static ref PRISMA_CLIENT: AsyncOnce<PrismaClient> =
        AsyncOnce::new(async { prisma::new_client().await.unwrap() });
}

pub async fn users<'a>() -> user::Actions<'a> {
    PRISMA_CLIENT.get().await.user()
}

pub async fn posts<'a>() -> post::Actions<'a> {
    PRISMA_CLIENT.get().await.post()
}

pub async fn pending_posts<'a>() -> pending_post::Actions<'a> {
    PRISMA_CLIENT.get().await.pending_post()
}

pub async fn create_user(id: Uuid, username: String, password: String) -> Result<String, Box<dyn std::error::Error>> {
    users()
        .await
        .create(username, password, vec![SetParam::SetId(id.to_string())])
        .exec()
        .await?;

    Ok(generate_api_token(id, false).context("Error generating JWT")?)
}

pub async fn get_user(username: String) -> Option<user::Data> {
    users()
        .await
        .find_unique(UniqueWhereParam::UsernameEquals(username))
        .exec()
        .await
        .unwrap()
}

impl<'a> FromParam<'a> for Category {
    type Error = strum::ParseError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(match param.to_uppercase().as_str() {
            "ISLAMISM" => Self::Islamism,
            "MODERNITY" => Self::Modernity,
            "SECULARISM" => Self::Secularism,
            "FEMINISM" => Self::Feminism,
            _ => return Err(strum::ParseError::VariantNotFound),
        })
    }
}

pub async fn create_post(
    category: Category,
    id: Uuid,
    author_id: Uuid,
    excerpt: String,
    citation: String,
    submitted_at: DateTime<chrono::FixedOffset>,
) -> Result<post::Data, QueryError> {
    posts()
        .await
        .create(
            UniqueWhereParam::IdEquals(author_id.to_string()),
            category,
            excerpt,
            citation,
            vec![
                post::SetParam::SetId(id.to_string()),
                post::SetParam::SetSubmittedAt(submitted_at),
            ],
        )
        .exec()
        .await
}

pub async fn get_post(category: Category, id: String) -> Result<Option<post::Data>, QueryError> {
    posts()
        .await
        .find_first(vec![post::category::equals(category), post::id::equals(id)])
        .exec()
        .await
}

pub async fn get_section_posts(category: Category) -> Result<Vec<post::Data>, QueryError> {
    posts()
        .await
        .find_many(vec![post::category::equals(category)])
        .exec()
        .await
}

pub async fn get_user_posts(author_id: String) -> Result<Vec<post::Data>, QueryError> {
    posts()
        .await
        .find_many(vec![post::author_id::equals(author_id)])
        .exec()
        .await
}

pub async fn get_user_posts_in_section(category: Category, author_id: String) -> Result<Vec<post::Data>, QueryError> {
    posts()
        .await
        .find_many(vec![
            post::category::equals(category),
            post::author_id::equals(author_id),
        ])
        .exec()
        .await
}

pub async fn remove_post(category: Category, id: String) -> Result<i64, QueryError> {
    posts()
        .await
        .delete_many(vec![post::category::equals(category), post::id::equals(id)])
        .exec()
        .await
}

// TODO: Minimize code duplication

pub async fn create_pending_post(
    category: Category,
    id: Uuid,
    author_id: Uuid,
    excerpt: String,
    citation: String,
) -> Result<pending_post::Data, QueryError> {
    pending_posts()
        .await
        .create(
            UniqueWhereParam::IdEquals(author_id.to_string()),
            category,
            excerpt,
            citation,
            vec![pending_post::SetParam::SetId(id.to_string())],
        )
        .exec()
        .await
}

pub async fn get_pending_post(category: Category, id: String) -> Result<Option<pending_post::Data>, QueryError> {
    pending_posts()
        .await
        .find_first(vec![
            pending_post::category::equals(category),
            pending_post::id::equals(id),
        ])
        .exec()
        .await
}

pub async fn get_section_pending_posts(category: Category) -> Result<Vec<pending_post::Data>, QueryError> {
    pending_posts()
        .await
        .find_many(vec![pending_post::category::equals(category)])
        .exec()
        .await
}

pub async fn get_user_pending_posts(author_id: String) -> Result<Vec<pending_post::Data>, QueryError> {
    pending_posts()
        .await
        .find_many(vec![pending_post::author_id::equals(author_id)])
        .exec()
        .await
}

pub async fn get_user_pending_posts_in_section(
    category: Category,
    author_id: String,
) -> Result<Vec<pending_post::Data>, QueryError> {
    pending_posts()
        .await
        .find_many(vec![
            pending_post::category::equals(category),
            pending_post::author_id::equals(author_id),
        ])
        .exec()
        .await
}

pub async fn remove_pending_post(category: Category, id: String) -> Result<i64, QueryError> {
    pending_posts()
        .await
        .delete_many(vec![
            pending_post::category::equals(category),
            pending_post::id::equals(id),
        ])
        .exec()
        .await
}
