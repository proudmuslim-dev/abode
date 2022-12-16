use crate::routes::util::jwt::{verify_api_token, Claims};
use rocket::{
    form::{Form, Strict},
    http::Status,
    request::{FromParam, FromRequest, Outcome},
    response::Responder,
    serde::json::Json,
    Request, Response,
};
use sanitizer::Sanitize;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, str::FromStr};
use strum::{Display, EnumString};
use validator::{Validate, ValidationError};

pub struct AuthHeader<const T: AuthLevel = { AuthLevel::User }> {
    pub(crate) token: String,
}

macro_rules! impl_from_req {
    ($($t:stmt),*) => {
        paste::paste! {
            $(
                #[rocket::async_trait]
                impl<'a> FromRequest<'a> for AuthHeader<{ $t }> {
                    // TODO: Better error handling
                    type Error = Infallible;

                    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
                        Outcome::Success(AuthHeader {
                            token: request
                                .headers()
                                .get_one("Authorization")
                                .unwrap()
                                .replace("Bearer ", ""),
                        })
                    }
                }
            )*
        }
    }
}

impl_from_req!(AuthLevel::User, AuthLevel::Admin);

pub trait Verifiable {
    fn verify(&self) -> Result<Claims, Status>;
}

impl Verifiable for AuthHeader<{ AuthLevel::User }> {
    fn verify(&self) -> Result<Claims, Status> {
        verify_api_token(self.token.as_str()).map_err(|_| Status::Unauthorized)
    }
}

impl Verifiable for AuthHeader<{ AuthLevel::Admin }> {
    fn verify(&self) -> Result<Claims, Status> {
        let claims = verify_api_token(self.token.as_str()).map_err(|_| Status::Unauthorized)?;

        if claims.admin {
            Ok(claims)
        } else {
            Err(Status::Unauthorized)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AuthLevel {
    User,
    Admin,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

impl<'r> Responder<'r, 'r> for LoginResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(&self).respond_to(request)?).ok()
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

pub mod jwt {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{errors::Result, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use std::env;
    use uuid::Uuid;

    lazy_static! {
        static ref ENCODING_SECRET: String = env::var("ENCODING_SECRET").unwrap();
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        /// The subject
        pub sub: Uuid,
        /// Expiry date of the token
        pub exp: usize,
        /// Whether or not the subject is an admin
        pub admin: bool,
    }

    pub fn generate_api_token(subject: Uuid, admin: bool) -> Result<String> {
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(6))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: subject,
            exp: exp as usize,
            admin,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(ENCODING_SECRET.as_bytes()),
        )
    }

    pub fn verify_api_token(token: &str) -> Result<Claims> {
        match jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(ENCODING_SECRET.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(t) => Ok(t.claims),
            Err(e) => Err(e),
        }
    }
}
