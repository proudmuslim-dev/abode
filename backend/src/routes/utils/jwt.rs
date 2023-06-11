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
    let claims = {
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(6))
            .expect("valid timestamp")
            .timestamp();

        Claims {
            sub: subject,
            exp: exp as usize,
            admin,
        }
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(ENCODING_SECRET.as_bytes()),
    )
}

pub fn verify_api_token(token: &str) -> Result<Claims> {
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(ENCODING_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(decoded.claims)
}
