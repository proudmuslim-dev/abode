use crate::db::{
    prisma::{notification, NotificationType},
    util::NotificationContent,
};
use rocket::{
    response::Responder,
    serde::{json::Json, Deserialize, Serialize},
    Request, Response,
};

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

impl<'r> Responder<'r, 'r> for LoginResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(&self).respond_to(request)?).ok()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub recipient_id: String,
    pub read: bool,
    pub n_type: NotificationType,
    pub content: NotificationContent,
}

impl From<notification::Data> for Notification {
    fn from(value: notification::Data) -> Self {
        let content = serde_json::from_str::<NotificationContent>(value.content.as_str()).unwrap();

        macro_rules! _self {
            { $($f:ident),* } => {
                paste::paste! {
                    Self {
                        content,
                        $(
                            $f: value.$f,
                        )*
                    }
                }
            }
        }

        _self! { id, created_at, recipient_id, read, n_type }
    }
}
