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
