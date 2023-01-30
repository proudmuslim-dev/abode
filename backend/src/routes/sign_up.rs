use crate::{
    db::util::create_user,
    routes::utils::{
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
use uuid::Uuid;
use validator::Validate;

#[post("/sign-up", data = "<signup>")]
pub async fn sign_up(signup: Form<Strict<SignUpForm>>) -> Result<LoginResponse, Status> {
    let signup = match sanitize_and_validate(signup) {
        Some(s) => s,
        None => return Err(Status::BadRequest),
    };

    // TODO: Improve error handling, notify if username is taken
    match create_user(Uuid::new_v4(), signup.username, signup.password).await {
        Ok(token) => Ok(LoginResponse { token }),
        Err(e) => {
            dbg!(e);
            Err(Status::InternalServerError)
        }
    }
}

#[derive(FromForm, Deserialize, Validate, Sanitize)]
pub struct SignUpForm {
    #[sanitize(trim, lower_case)]
    #[validate(length(min = 3, max = 50), custom = "validate_username")]
    username: String,
    #[sanitize(trim)]
    password: String,
}
