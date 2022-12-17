use crate::routes::utils::jwt::{verify_api_token, Claims};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use std::convert::Infallible;

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
