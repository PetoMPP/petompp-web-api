use crate::{impl_query_config, schema::users};

impl_query_config!(
    users::dsl::users,
    users::table,
    users::BoxedQuery<'static, Pg>,
    UsersQuery,
    [
        (users::id, "id"),
        (users::name, "name"),
        (users::normalized_name, "normalized_name"),
        (users::role, "role"),
        (users::confirmed, "confirmed"),
        (users::created_at, "created_at"),
        (users::deleted_at, "deleted_at"),
    ]
);
