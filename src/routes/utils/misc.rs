use diesel::result::Error as DieselError;
use rocket::{
    form::{Form, FromFormField, Strict, ValueField},
    http::Status,
    request::FromParam,
    serde::{Deserialize, Serialize},
};
use sanitizer::Sanitize;
use std::{ops::Deref, str::FromStr};
use strum::{Display, EnumString};
use uuid::Uuid;
use validator::{Validate, ValidationError};

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

#[derive(Copy, Clone)]
pub struct UuidField(pub(crate) Uuid);

impl Deref for UuidField {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'v> FromFormField<'v> for UuidField {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let val = field
            .value
            .chars()
            .filter(|c| !c.is_whitespace() && c.is_ascii())
            .collect::<String>();

        let id = Uuid::from_str(val.as_str()).map_err(|_| rocket::form::Error::validation("invalid uuid"))?;

        Ok(UuidField(id))
    }
}

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
