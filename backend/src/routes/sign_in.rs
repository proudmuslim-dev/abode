use crate::{
    db::{prisma::Role, util::get_user},
    routes::utils::{
        jwt::generate_api_token,
        misc::{sanitize_and_validate, validate_username},
        responses::LoginResponse,
    },
};
use rocket::{
    form::{Form, Strict},
    http::Status,
};
use sanitizer::prelude::*;
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

#[post("/sign-in", data = "<login>")]
pub async fn sign_in(login: Form<Strict<LoginForm>>) -> Result<LoginResponse, Status> {
    let login = match sanitize_and_validate(login) {
        Some(l) => l,
        None => return Err(Status::BadRequest),
    };

    let db_res = match get_user(login.username).await {
        Some(res) => res,
        None => return Err(Status::Unauthorized),
    };

    if db_res.password != login.password {
        return Err(Status::Unauthorized);
    }

    let id = match Uuid::from_str(db_res.id.as_str()) {
        Ok(i) => i,
        Err(_) => return Err(Status::InternalServerError),
    };

    let admin = matches!(db_res.role, Role::Admin);

    match generate_api_token(id, admin) {
        Ok(token) => Ok(LoginResponse { token }),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[derive(FromForm, Debug, Deserialize, Validate, Sanitize)]
pub struct LoginForm {
    #[sanitize(trim, lower_case)]
    #[validate(length(min = 3, max = 50), custom = "validate_username")]
    username: String,
    #[sanitize(trim)]
    password: String,
}
