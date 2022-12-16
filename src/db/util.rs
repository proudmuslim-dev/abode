use crate::{
    db::{
        models::app::{NewPost, NewUser, User},
        schema,
    },
    routes::util::{jwt::generate_api_token, Sections},
};
use color_eyre::eyre::Context;
use diesel::{Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("app.db").expect("Failed to connect to db!")
}

pub fn create_user(
    conn: &mut SqliteConnection,
    id: Uuid,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    use schema::users;

    let new_user = NewUser {
        id: id.to_string(),
        username,
        password,
        admin: false,
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .execute(conn)
        .context("Error creating new account")?;

    let token = generate_api_token(id, false).context("Error generating API token")?;

    Ok(token)
}

pub fn get_user(conn: &mut SqliteConnection, uname: &str) -> Option<User> {
    use schema::users::dsl::*;

    let matches = users
        .filter(username.eq(uname))
        .limit(1)
        .load::<User>(conn)
        .expect("Error loading users");

    if matches.is_empty() {
        None
    } else {
        Some(matches[0].clone())
    }
}

pub fn get_user_uid(conn: &mut SqliteConnection, uid: Uuid) -> Option<User> {
    use schema::users::dsl::*;

    match users.find(uid.to_string()).first::<User>(conn) {
        Ok(u) => Some(u),
        Err(_) => None,
    }
}

pub fn create_post(
    conn: &mut SqliteConnection,
    section: Sections,
    id: Uuid,
    author_id: Uuid,
    excerpt: &str,
    citation: &str,
) -> QueryResult<()> {
    let id = id.to_string();
    let author_id = author_id.to_string();

    let new_post = NewPost {
        id,
        author_id: author_id.as_str(),
        excerpt,
        citation,
    };

    new_post.insert(section, conn)
}
