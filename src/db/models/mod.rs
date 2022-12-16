pub mod app;
pub mod pending;

macro_rules! section {
    ($name:ident, $db:ident) => {
        paste::paste! {
            #[derive(Queryable)]
            pub struct [<$name Entry>] {
                pub post_id: String,
            }

            #[derive(Insertable)]
            #[diesel(table_name = $db)]
            pub struct [<New $name Entry>]<'a> {
                pub post_id: &'a str,
            }
        }
    };
}

pub(crate) use section;
