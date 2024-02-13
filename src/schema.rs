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
    user_settings (lock) {
        #[max_length = 1]
        lock -> Bpchar,
        name_min_length -> Int4,
        name_max_length -> Int4,
        #[max_length = 32]
        name_special_characters -> Varchar,
        password_min_length -> Int4,
        password_needed_checks -> Int4,
        password_check_numbers -> Bool,
        password_check_uppercase -> Bool,
        password_check_lowercase -> Bool,
        password_check_special_characters -> Bool,
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

diesel::allow_tables_to_appear_in_same_query!(resources, user_settings, users,);
