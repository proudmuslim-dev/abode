use rocket::form::{Form, FromFormField, Strict, ValueField};
use sanitizer::Sanitize;
use std::{ops::Deref, str::FromStr};

use uuid::Uuid;
use validator::{Validate, ValidationError};

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