CREATE TABLE tracks (
    id BIGSERIAL PRIMARY KEY,
    author_username varchar NOT NULL,
    name varchar NOT NULL,
    cnt_rates bigint NOT NULL,
    sum_rates bigint NOT NULL,
    UNIQUE (author_username, name)
);

-- TODO для комментов отдельную таблицу, но пока без комментов.
-- CREATE TABLE comments (
--   id SERIAL PRIMARY KEY,
--   track_id INTEGER NOT NULL,
--   author_username VARCHAR NOT NULL,
--   body TEXT NOT NULL,
--   FOREIGN KEY (track_id) REFERENCES tracks (id) ON DELETE CASCADE
-- );
