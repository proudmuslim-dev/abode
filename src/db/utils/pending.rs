use crate::{
    db::{
        delete_from_section,
        models::pending::{
            NewPendingFeminismEntry, NewPendingIslamismEntry, NewPendingModernityEntry, NewPendingPost,
            NewPendingSecularismEntry, PendingIslamismEntry, PendingPost,
        },
        schemas::pending as schema,
    },
    routes::utils::misc::Sections,
};
use diesel::{result::Error, Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("pending.db").expect("Failed to connect to db!")
}

pub fn create_pending_post(
    conn: &mut SqliteConnection,
    section: Sections,
    id: Uuid,
    author_id: Uuid,
    excerpt: &str,
    citation: &str,
) -> QueryResult<()> {
    use schema::pending_posts;

    let id = id.to_string();
    let author_id = author_id.to_string();

    let new_post = NewPendingPost {
        id: id.clone(),
        author_id: author_id.as_str(),
        excerpt,
        citation,
    };

    diesel::insert_into(pending_posts::table)
        .values(new_post)
        .execute(conn)?;

    macro_rules! insert_into_section {
        ($($variant:ident => $section:ident),*) => {
            paste::paste! {
                match section {
                    $(
                        Sections::$variant => {
                            use crate::db::schemas::pending::$section;

                            diesel::insert_into($section::table)
                                .values([<New Pending $variant Entry>] {
                                    post_id: id.as_str(),
                                })
                                .execute(conn)?;
                        }
                    )*
                }
            }
        };
    }

    insert_into_section!(
        Islamism => islamism,
        Modernity => modernity,
        Secularism => secularism,
        Feminism => feminism
    );

    Ok(())
}

pub fn get_pending_post(conn: &mut SqliteConnection, section: Sections, pid: String) -> QueryResult<PendingPost> {
    use schema::pending_posts::{dsl::pending_posts, id};

    macro_rules! get_from_section {
        ($($variant:ident => $section:ident),*) => {
            paste::paste! {
                match section {
                    $(
                        Sections::$variant => {
                            use schema::$section::{dsl::$section, post_id};

                            let s_matches = $section.filter(post_id.eq(pid.clone())).limit(1).load::<PendingIslamismEntry>(conn)?;

                            if s_matches.is_empty() {
                                return Err(Error::NotFound);
                            }

                            let matches = pending_posts.filter(id.eq(pid)).limit(1).load::<PendingPost>(conn)?;

                            if matches.is_empty() {
                                Err(Error::NotFound)
                            } else {
                                Ok(matches[0].clone())
                            }
                        }
                    )*
                }
            }
        };
    }

    get_from_section!(
        Islamism => islamism,
        Modernity => modernity,
        Secularism => secularism,
        Feminism => feminism
    )
}

pub fn remove_pending_post(conn: &mut SqliteConnection, section: Sections, post_id: String) -> QueryResult<()> {
    delete_from_section!(
        pending_posts { section, post_id, conn },
        Islamism => islamism,
        Modernity => modernity,
        Secularism => secularism,
        Feminism => feminism
    );

    Ok(())
}

pub fn get_and_remove_pending_post(
    conn: &mut SqliteConnection,
    section: Sections,
    post_id: String,
) -> QueryResult<PendingPost> {
    let post = get_pending_post(conn, section, post_id.clone())?;

    remove_pending_post(conn, section, post_id)?;

    Ok(post)
}
