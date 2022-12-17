use crate::{
    db::utils,
    routes::util::{sanitize_and_validate, validate_username, LoginResponse},
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
    match utils::app::create_user(
        &mut utils::app::establish_connection(),
        Uuid::new_v4(),
        signup.username.as_str(),
        signup.password.as_str(),
    ) {
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
    #[validate(length(min = 3), custom = "validate_username")]
    username: String,
    #[sanitize(trim)]
    password: String,
}
