use crate::db::prisma::Category;
use rocket::serde::json::{json, Value};

#[get("/sections")]
pub async fn sections() -> Value {
    json!(Category::ALL)
}
