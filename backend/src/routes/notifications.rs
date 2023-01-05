use crate::{
    db::{
        prisma::notification,
        util::{get_user_notifications, WhichNotifications},
    },
    routes::utils::{
        headers::{AuthHeader, Verifiable},
        jwt::Claims,
    },
};
use rocket::{http::Status, serde::json::Json};

// TODO: Consider paginating
#[get("/notifications?<which>")]
pub async fn get_notifications(
    auth_header: AuthHeader,
    which: Option<WhichNotifications>,
) -> Result<Json<Vec<notification::Data>>, Status> {
    let which = which.unwrap_or(WhichNotifications::Unread);
    let Claims { sub: user, .. } = auth_header.verify()?;

    Ok(Json(
        get_user_notifications(user, which)
            .await
            .map_err(|_| Status::InternalServerError)?,
    ))
}

// TODO: Mark as read/unread
