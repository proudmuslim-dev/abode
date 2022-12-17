use diesel::result::Error as DieselError;
use rocket::{
    form::{Form, Strict},
    http::Status,
    request::FromParam,
    serde::{Deserialize, Serialize},
};
use sanitizer::Sanitize;
use std::str::FromStr;
use strum::{Display, EnumString};
use validator::{Validate, ValidationError};

pub fn db_err_to_status(e: DieselError) -> Status {
    match e {
        DieselError::NotFound => Status::NotFound,
        _ => Status::InternalServerError,
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let mut username = username.to_owned();

    username.retain(|c| !c.is_whitespace());

    // Don't waste time if it's a junk req
    if username.is_empty() {
        return Err(ValidationError::new("Invalid username"));
    }

    Ok(())
}

pub fn sanitize_and_validate<T>(form: Form<Strict<T>>) -> Option<T>
where
    T: Validate + Sanitize,
{
    let mut form = form.into_inner().into_inner();

    form.validate().ok()?;
    form.sanitize();

    Some(form)
}

#[derive(Copy, Clone, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[strum(ascii_case_insensitive)]
pub enum Sections {
    Islamism,
    Modernity,
    Secularism,
    Feminism,
}

impl<'a> FromParam<'a> for Sections {
    type Error = strum::ParseError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(param)
    }
}

impl Sections {
    pub const ALL: [Sections; 4] = [
        Sections::Islamism,
        Sections::Modernity,
        Sections::Secularism,
        Sections::Feminism,
    ];
}
