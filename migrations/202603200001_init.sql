CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS artists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_artists_normalized_name ON artists(normalized_name);

CREATE TABLE IF NOT EXISTS songs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subsonic_id TEXT NOT NULL UNIQUE,
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    normalized_title TEXT NOT NULL,
    album TEXT,
    genre TEXT,
    year INT,
    duration_seconds INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_songs_artist_id ON songs(artist_id);
CREATE INDEX IF NOT EXISTS idx_songs_genre ON songs(genre);
CREATE INDEX IF NOT EXISTS idx_songs_year ON songs(year);
CREATE INDEX IF NOT EXISTS idx_songs_normalized_title ON songs(normalized_title);

CREATE TABLE IF NOT EXISTS play_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    played_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_play_history_song_id ON play_history(song_id);
CREATE INDEX IF NOT EXISTS idx_play_history_played_at ON play_history(played_at);

CREATE TABLE IF NOT EXISTS aggregated_stats (
    song_id UUID PRIMARY KEY REFERENCES songs(id) ON DELETE CASCADE,
    total_play_count BIGINT NOT NULL DEFAULT 0,
    recent_7d_count BIGINT NOT NULL DEFAULT 0,
    recent_30d_count BIGINT NOT NULL DEFAULT 0,
    last_played_at TIMESTAMPTZ,
    score_cache DOUBLE PRECISION,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_aggregated_stats_score ON aggregated_stats(score_cache DESC);
CREATE INDEX IF NOT EXISTS idx_aggregated_stats_recent ON aggregated_stats(recent_7d_count DESC);

CREATE TABLE IF NOT EXISTS generated_playlists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    date DATE NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    song_ids UUID[] NOT NULL,
    navidrome_playlist_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(date, kind)
);

CREATE INDEX IF NOT EXISTS idx_generated_playlists_date ON generated_playlists(date);
