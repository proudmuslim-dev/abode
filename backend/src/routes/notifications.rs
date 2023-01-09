use crate::{
    db::util::{get_user_notifications, WhichNotifications},
    routes::utils::{
        headers::{AuthHeader, Verifiable},
        jwt::Claims,
        responses::Notification,
    },
};
use rocket::{http::Status, serde::json::Json};

// TODO: Consider paginating
#[get("/notifications?<which>")]
pub async fn get_notifications(
    auth_header: AuthHeader,
    which: Option<WhichNotifications>,
) -> Result<Json<Vec<Notification>>, Status> {
    let which = which.unwrap_or(WhichNotifications::Unread);
    let Claims { sub: user, .. } = auth_header.verify()?;

    let notifs: Vec<Notification> = get_user_notifications(user, which)
        .await
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .map(Notification::from)
        .collect();

    Ok(Json(notifs))
}

// TODO: Mark as read/unread
