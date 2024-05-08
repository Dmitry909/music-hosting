CREATE TABLE playlists (
    id BIGSERIAL PRIMARY KEY,
    owner_username varchar NOT NULL,
    name varchar NOT NULL,
    UNIQUE (owner_username, name)
);

CREATE TABLE playlists_tracks (
    id BIGSERIAL PRIMARY KEY,
    playlist_id BIGINT,
    track_id BIGINT,
    UNIQUE (playlist_id, track_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists (id) ON DELETE CASCADE 
);
