# Daily Playlist Generator

A Rust service that builds 5 daily playlists from Subsonic/Navidrome metadata and Last.fm listening signals.

Generated playlists:
- Favorites Mix
- Rediscovery Mix
- Genre Mix
- Artist Mix
- Smart Shuffle

## Modules

- `ingestion`: fetch and store metadata/signals
- `normalization`: robust matching and text cleanup
- `scoring`: deterministic weighted ranking with light randomness
- `generator`: build 5 playlists with soft de-dup constraints
- `exporter`: idempotent playlist upsert to Navidrome
- `scheduler`: daily orchestration loop

Detailed architecture is in `docs/ARCHITECTURE.md`.

## Scoring formula

For each candidate song:

```
score = 0.30*metadata_similarity
	+ 0.28*ln(total_play_count + 1)
	+ 0.22*ln(recent_7d_count + 1)
	+ 0.12*(1/sqrt(days_since_last_play))
	- 0.08*(repetition_14d_count^1.15)
	+ deterministic_jitter
```

## Local run

1. Copy `.env.example` to `.env` and fill credentials.
2. Start stack: `docker compose up -d`.
3. Check logs: `docker compose logs -f playlist-generator`.
4. Health check: `curl http://localhost:8080/health`.

## Bare-metal run

1. Start postgres: `docker compose up -d postgres`.
2. Export environment from `.env`.
3. Run `cargo run`.

## Daily scheduling

The service runs one job immediately on startup, then once daily at 00:05 UTC.
