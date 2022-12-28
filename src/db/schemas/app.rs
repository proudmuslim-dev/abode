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
    posts (id) {
        id -> Text,
        author_id -> Text,
        excerpt -> Text,
        citation -> Text,
        creation -> Timestamp,
    }
}

diesel::table! {
    secularism (post_id) {
        post_id -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        admin -> Bool,
    }
}

diesel::joinable!(feminism -> posts (post_id));
diesel::joinable!(islamism -> posts (post_id));
diesel::joinable!(modernity -> posts (post_id));
diesel::joinable!(posts -> users (author_id));
diesel::joinable!(secularism -> posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(feminism, islamism, modernity, posts, secularism, users,);
