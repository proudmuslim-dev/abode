// @generated automatically by Diesel CLI.

diesel::table! {
    feminism (post_id) {
        post_id -> Text,
    }
}

diesel::table! {
    islamism (post_id) {
        post_id -> Text,
    }
}

diesel::table! {
    modernity (post_id) {
        post_id -> Text,
    }
}

diesel::table! {
    pending_posts (id) {
        id -> Text,
        author_id -> Text,
        excerpt -> Text,
        citation -> Text,
    }
}

diesel::table! {
    secularism (post_id) {
        post_id -> Text,
    }
}

diesel::joinable!(feminism -> pending_posts (post_id));
diesel::joinable!(islamism -> pending_posts (post_id));
diesel::joinable!(modernity -> pending_posts (post_id));
diesel::joinable!(secularism -> pending_posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(feminism, islamism, modernity, pending_posts, secularism,);
