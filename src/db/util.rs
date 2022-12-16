use crate::{
    db::{
        delete_from_section,
        models::app::{NewUser, User},
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

// TODO: Decide whether to keep this or not
pub fn get_user_uid(conn: &mut SqliteConnection, uid: Uuid) -> Option<User> {
    use schema::users::dsl::*;

    match users.find(uid.to_string()).first::<User>(conn) {
        Ok(u) => Some(u),
        Err(_) => None,
    }
}

pub fn remove_post(conn: &mut SqliteConnection, section: Sections, post_id: String) -> QueryResult<()> {
    delete_from_section!(
        posts { section, post_id, conn },
        Islamism => islamism,
        Modernity => modernity,
        Secularism => secularism,
        Feminism => feminism
    );

    Ok(())
}
