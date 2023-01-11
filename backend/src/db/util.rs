use crate::{
    db::{
        prisma,
        prisma::{
            notification, pending_post, post, user,
            user::{SetParam, UniqueWhereParam},
            Category, NotificationType, PrismaClient,
        },
    },
    routes::utils::{jwt::generate_api_token, misc::PaginationFields},
};
use async_once::AsyncOnce;
use chrono::DateTime;
use color_eyre::eyre::Context;
use lazy_static::lazy_static;
use prisma_client_rust::{Direction, QueryError};
use rocket::request::FromParam;
use std::str::FromStr;
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

pub async fn notifications<'a>() -> notification::Actions<'a> {
    PRISMA_CLIENT.get().await.notification()
}

macro_rules! find_in_posts {
    (post, $pagination:ident, $($filter:expr),*) => {
        find_in_posts!(post::confirmed_at::order(Direction::Desc), $pagination, post => $($filter),*)
    };
    (pending_post, $pagination:ident, $($filter:expr),*) => {
        find_in_posts!(pending_post::submitted_at::order(Direction::Desc), $pagination, pending_post => $($filter),*)
    };
    ($order:expr, $pagination:ident, $post:ident => $($filter:expr),*) => {
        paste::paste! {
            [<$post s>]()
                .await
                .find_many(vec![
                    $(
                        $filter,
                    )*
                ])
                .order_by($order)
                .skip($pagination.skip())
                .take($pagination.per_page.into())
                .exec()
                .await
        }
    };
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

pub async fn get_user_notifications(
    user: Uuid,
    which: WhichNotifications,
) -> Result<Vec<notification::Data>, QueryError> {
    let mut filters = vec![notification::WhereParam::RecipientIdEquals(user.to_string())];

    match which {
        WhichNotifications::Read => filters.push(notification::WhereParam::ReadEquals(true)),
        WhichNotifications::Unread => filters.push(notification::WhereParam::ReadEquals(false)),
        WhichNotifications::All => {}
    }

    notifications()
        .await
        .find_many(filters)
        .order_by(notification::created_at::order(Direction::Desc))
        .exec()
        .await
}

#[derive(Copy, Clone, PartialEq, Eq, FromFormField)]
pub enum WhichNotifications {
    Read,
    Unread,
    All,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Islamism,
        Category::Modernity,
        Category::Secularism,
        Category::Feminism,
    ];
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

pub async fn get_section_posts(
    category: Category,
    pagination: PaginationFields,
) -> Result<Vec<post::Data>, QueryError> {
    find_in_posts!(post, pagination, post::category::equals(category))
}

pub async fn get_user_posts(author_id: String, pagination: PaginationFields) -> Result<Vec<post::Data>, QueryError> {
    find_in_posts!(post, pagination, post::author_id::equals(author_id))
}

pub async fn get_user_posts_in_section(
    category: Category,
    author_id: String,
    pagination: PaginationFields,
) -> Result<Vec<post::Data>, QueryError> {
    find_in_posts!(
        post,
        pagination,
        post::category::equals(category),
        post::author_id::equals(author_id)
    )
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

pub async fn get_section_pending_posts(
    category: Category,
    pagination: PaginationFields,
) -> Result<Vec<pending_post::Data>, QueryError> {
    find_in_posts!(pending_post, pagination, pending_post::category::equals(category))
}

pub async fn get_user_pending_posts(
    author_id: String,
    pagination: PaginationFields,
) -> Result<Vec<pending_post::Data>, QueryError> {
    find_in_posts!(pending_post, pagination, pending_post::author_id::equals(author_id))
}

pub async fn get_user_pending_posts_in_section(
    category: Category,
    author_id: String,
    pagination: PaginationFields,
) -> Result<Vec<pending_post::Data>, QueryError> {
    find_in_posts!(
        pending_post,
        pagination,
        pending_post::category::equals(category),
        pending_post::author_id::equals(author_id)
    )
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

pub async fn confirm_pending_post(
    category: Category,
    id: String,
    comment: Option<String>,
) -> Result<notification::Data, QueryError> {
    let pending_post::Data {
        author_id: uid,
        excerpt,
        citation,
        submitted_at,
        ..
    } = match pending_posts()
        .await
        .find_unique(pending_post::UniqueWhereParam::IdEquals(id.clone()))
        .exec()
        .await?
    {
        Some(p) => p,
        None => {
            return Err(QueryError::Deserialize(serde_value::DeserializerError::Custom(
                "Not Found".to_owned(),
            )))
        }
    };

    remove_pending_post(category, id.clone()).await?;

    let new_id = Uuid::new_v4();

    create_post(
        category,
        new_id,
        Uuid::from_str(uid.as_str()).unwrap(),
        excerpt,
        citation,
        submitted_at,
    )
    .await?;

    let notif = NotificationContent::PostApproval {
        url: format!("/posts/{}?id={new_id}", category.to_string().to_ascii_lowercase()),
        comment,
    };

    create_notification(uid, &notif).await
}

pub async fn reject_pending_post(
    category: Category,
    id: String,
    comment: Option<String>,
) -> Result<notification::Data, QueryError> {
    let pending_post::Data {
        author_id: uid,
        excerpt,
        citation,
        ..
    } = match pending_posts()
        .await
        .find_unique(pending_post::UniqueWhereParam::IdEquals(id.clone()))
        .exec()
        .await?
    {
        Some(p) => p,
        None => {
            return Err(QueryError::Deserialize(serde_value::DeserializerError::Custom(
                "Not Found".to_owned(),
            )))
        }
    };

    remove_pending_post(category, id.clone()).await?;

    let notif = NotificationContent::PostRejection {
        comment,
        excerpt,
        citation,
    };

    create_notification(uid, &notif).await
}

pub async fn create_notification(uid: String, notif: &NotificationContent) -> Result<notification::Data, QueryError> {
    let content = serde_json::to_string(notif).unwrap();

    notifications()
        .await
        .create(UniqueWhereParam::IdEquals(uid), notif.enum_type(), content, vec![])
        .exec()
        .await
}

pub async fn update_user_notification(uid: String, id: String, read: bool) -> Result<(), QueryError> {
    notifications()
        .await
        .update_many(
            vec![
                notification::WhereParam::RecipientIdEquals(uid),
                notification::WhereParam::IdEquals(id),
            ],
            vec![notification::SetParam::SetRead(read)],
        )
        .exec()
        .await?;

    Ok(())
}

pub async fn delete_user_notification(uid: String, id: String) -> Result<(), QueryError> {
    notifications()
        .await
        .delete_many(vec![
            notification::WhereParam::RecipientIdEquals(uid),
            notification::WhereParam::IdEquals(id),
        ])
        .exec()
        .await?;

    Ok(())
}

pub async fn delete_notification(id: String) -> Result<(), QueryError> {
    notifications()
        .await
        .delete(notification::UniqueWhereParam::IdEquals(id))
        .exec()
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotificationContent {
    PostApproval {
        url: String,
        comment: Option<String>,
    },
    PostRejection {
        comment: Option<String>,
        excerpt: String,
        citation: String,
    },
}

impl NotificationContent {
    pub fn enum_type(&self) -> NotificationType {
        match self {
            NotificationContent::PostApproval { .. } => NotificationType::Approval,
            NotificationContent::PostRejection { .. } => NotificationType::Rejection,
        }
    }
}
