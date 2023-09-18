// @generated automatically by Diesel CLI.

diesel::table! {
    resources (key) {
        #[max_length = 64]
        key -> Varchar,
        en -> Text,
        pl -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        normalized_name -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        role -> Int4,
        confirmed -> Bool,
        created_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(resources, users,);
