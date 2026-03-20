# Architecture and Data Flow

## 1. Internal Context

### Available data
- Navidrome/Subsonic metadata: song id, artist, album, genre, year, duration.
- Last.fm behavior: recent tracks, top tracks, play count signals.
- Local persistence: song catalog, play history, aggregated stats, generated playlists.

### Assumptions
- Subsonic password auth is accepted by target Navidrome instance.
- Last.fm API key has permissions for user stats.
- Playlist updates are performed by deleting same-name playlist first, then recreating.
- Song matching threshold of 0.78 is sufficient for noisy metadata.

### Scoring context
- The ranking score uses metadata similarity, popularity, recency, freshness, and repeat-penalty.
- A deterministic jitter is injected per day+playlist kind to avoid fixed ordering ties.

## 2. Components

### Ingestion
Input:
- Subsonic API (`getArtists`, `getArtist`, `getAlbum`)
- Last.fm API (`user.getrecenttracks`, `user.gettoptracks`)

Output:
- Rows in `artists`, `songs`, `play_history`, `aggregated_stats`

Transformations:
- API payload flattening
- Text normalization for matching keys
- Upsert semantics for idempotent re-runs

### Normalization
Input:
- Raw artist/title strings from both providers

Output:
- Canonical lowercase, punctuation-stripped names
- Similarity score from fuzzy matching

Transformations:
- Normalize text to compare artist/title pairs
- Compute weighted similarity with edit distance

### Scoring
Input:
- Aggregated listening features and metadata completeness

Output:
- One scalar score per candidate song

Transformations:
- Log-scale play count and recency
- Apply freshness and repeat penalties
- Add deterministic jitter

### Playlist Generation
Input:
- Scored song candidates
- Date and playlist type

Output:
- Five playlist drafts (~50 songs each)

Transformations:
- Kind-specific scoring bias
- Soft cross-playlist dedup constraint
- Artist clustering and genre balancing checks

### Export
Input:
- Playlist drafts with internal song UUIDs

Output:
- Navidrome playlists and persisted mapping in `generated_playlists`

Transformations:
- Convert UUID songs to Subsonic song IDs
- Delete same-name playlist to ensure idempotent replacement
- Create playlist and store resulting remote ID

## 3. Pipeline Data Flow

1. Scheduler starts daily job.
2. Subsonic ingestion updates local catalog.
3. Last.fm ingestion updates play signals.
4. Aggregated stats are recomputed.
5. Scoring computes ranking vectors per song.
6. Generator builds 5 playlist drafts.
7. Exporter upserts corresponding Navidrome playlists.
8. Final playlist snapshots are stored in DB.
