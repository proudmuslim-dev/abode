use crate::{
    db::{
        util,
        util::{get_user_notifications, update_user_notification, WhichNotifications},
    },
    routes::utils::{
        headers::{AuthHeader, Verifiable},
        jwt::Claims,
        misc::PaginationFields,
        responses::NotificationBody,
    },
};
use rocket::{http::Status, serde::json::Json};
use uuid::Uuid;

#[get("/notifications?<which>&<pagination..>")]
pub async fn get_notifications(
    auth_header: AuthHeader,
    which: Option<WhichNotifications>,
    pagination: PaginationFields,
) -> Result<Json<Vec<NotificationBody>>, Status> {
    let notifs: Vec<NotificationBody> = {
        let which = which.unwrap_or(WhichNotifications::Unread);
        let Claims { sub: user, .. } = auth_header.verify()?;

        get_user_notifications(user, which, pagination)
            .await
            .map_err(|_| Status::InternalServerError)?
            .into_iter()
            .map(NotificationBody::from)
            .collect()
    };

    Ok(Json(notifs))
}

// NOTE: Can't use patch on frontend because of form limitations
#[patch("/notifications", data = "<patches>")]
pub async fn patch_notifications(
    auth_header: AuthHeader,
    patches: Json<Vec<PatchNotificationBody>>,
) -> Result<(), Status> {
    let Claims { sub: user, .. } = auth_header.verify()?;

    for p in patches.iter() {
        update_user_notification(user.to_string(), p.notification_id.to_string(), p.read)
            .await
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(())
}

#[delete("/notifications", data = "<form>")]
pub async fn delete_notification(auth_header: AuthHeader, form: Json<DeleteNotificationForm>) -> Result<(), Status> {
    let Claims { sub: user, admin, .. } = auth_header.verify()?;

    if !admin {
        return util::delete_user_notification(user.to_string(), form.id.to_string())
            .await
            .map_err(|_| Status::InternalServerError);
    }

    util::delete_notification(form.id.to_string())
        .await
        .map_err(|_| Status::InternalServerError)
}

#[derive(Deserialize)]
pub struct PatchNotificationBody {
    notification_id: Uuid,
    read: bool,
}

#[derive(Deserialize)]
pub struct DeleteNotificationForm {
    id: Uuid,
}
