-- Your SQL goes here
CREATE TABLE user_settings (
    lock CHAR(1) PRIMARY KEY,
    name_min_length INTEGER NOT NULL,
    name_max_length INTEGER NOT NULL,
    name_special_characters VARCHAR(32) NOT NULL,
    password_min_length INTEGER NOT NULL,
    password_needed_checks INTEGER NOT NULL,
    password_check_numbers BOOLEAN NOT NULL,
    password_check_uppercase BOOLEAN NOT NULL,
    password_check_lowercase BOOLEAN NOT NULL,
    password_check_special_characters BOOLEAN NOT NULL
);

INSERT INTO user_settings (lock, name_min_length, name_max_length, name_special_characters, password_min_length, password_needed_checks, password_check_numbers, password_check_uppercase, password_check_lowercase, password_check_special_characters)
VALUES ('X', 3, 28, '-_.$@!#%^&*', 8, 3, TRUE, TRUE, TRUE, TRUE);
