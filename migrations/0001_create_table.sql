CREATE TABLE IF NOT EXISTS users (
    username varchar PRIMARY KEY NOT NULL,
    password_hash varchar NOT NULL,
    active_token varchar NOT NULL
);