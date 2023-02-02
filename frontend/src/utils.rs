use lazy_static::lazy_static;
use rocket::request::FromParam;
use std::env;

lazy_static! {
    pub static ref BACKEND_URL: String =
        env::var("BACKEND_URL").expect("BACKEND_URL env var must be passed for app to work");
    pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum Category {
    Islamism,
    Modernity,
    Secularism,
    Feminism,
}

impl ToString for Category {
    fn to_string(&self) -> String {
        match self {
            Self::Islamism => "Islamism".to_owned(),
            Self::Modernity => "Modernity".to_owned(),
            Self::Secularism => "Secularism".to_owned(),
            Self::Feminism => "Feminism".to_owned(),
        }
    }
}

impl<'a> FromParam<'a> for Category {
    type Error = strum::ParseError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(match param.to_uppercase().as_str() {
            "ISLAMISM" => Self::Islamism,
            "MODERNITY" => Self::Modernity,
            "SECULARISM" => Self::Secularism,
            "FEMINISM" => Self::Feminism,
            _ => return Err(strum::ParseError::VariantNotFound),
        })
    }
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Islamism,
        Category::Modernity,
        Category::Secularism,
        Category::Feminism,
    ];
}

#[derive(Serialize, Deserialize)]
pub struct BrowsePost {
    pub id: String,
    pub excerpt: String,
    pub citation: String,
}
