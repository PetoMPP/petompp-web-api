use diesel::{query_builder::AsChangeset, Insertable, Queryable};
use petompp_web_models::models::user_settings_dto::UserSettingsDto;

#[derive(Default, Queryable, Insertable, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::user_settings)]
#[diesel(primary_key(lock))]
pub struct UserSettings {
    #[diesel(deserialize_as = String)]
    lock: Option<String>,
    #[diesel(deserialize_as = i32)]
    name_min_length: Option<i32>,
    #[diesel(deserialize_as = i32)]
    name_max_length: Option<i32>,
    #[diesel(deserialize_as = String)]
    name_special_characters: Option<String>,
    #[diesel(deserialize_as = i32)]
    password_min_length: Option<i32>,
    #[diesel(deserialize_as = i32)]
    password_needed_checks: Option<i32>,
    #[diesel(deserialize_as = bool)]
    password_check_numbers: Option<bool>,
    #[diesel(deserialize_as = bool)]
    password_check_uppercase: Option<bool>,
    #[diesel(deserialize_as = bool)]
    password_check_lowercase: Option<bool>,
    #[diesel(deserialize_as = bool)]
    password_check_special_characters: Option<bool>,
}

impl From<UserSettingsDto> for UserSettings {
    fn from(value: UserSettingsDto) -> Self {
        Self {
            name_min_length: value.name_min_length,
            name_max_length: value.name_max_length,
            name_special_characters: value.name_special_characters,
            password_min_length: value.password_min_length,
            password_needed_checks: value.password_needed_checks,
            password_check_numbers: value.password_check_numbers,
            password_check_uppercase: value.password_check_uppercase,
            password_check_lowercase: value.password_check_lowercase,
            password_check_special_characters: value.password_check_special_characters,
            ..Default::default()
        }
    }
}

impl From<UserSettings> for UserSettingsDto {
    fn from(val: UserSettings) -> Self {
        UserSettingsDto {
            name_min_length: val.name_min_length,
            name_max_length: val.name_max_length,
            name_special_characters: val.name_special_characters,
            password_min_length: val.password_min_length,
            password_needed_checks: val.password_needed_checks,
            password_check_numbers: val.password_check_numbers,
            password_check_uppercase: val.password_check_uppercase,
            password_check_lowercase: val.password_check_lowercase,
            password_check_special_characters: val.password_check_special_characters,
        }
    }
}
