// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        role -> Int4,
        confirmed -> Bool,
        created_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}
