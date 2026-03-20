# Daily Playlist Generator

A Rust service that builds 5 daily playlists from Subsonic/Navidrome metadata and Last.fm listening signals.

## Modules

- `ingestion`: fetch and store metadata/signals
- `normalization`: robust matching and text cleanup
- `scoring`: deterministic weighted ranking with light randomness
- `generator`: build 5 playlists with soft de-dup constraints
- `exporter`: idempotent playlist upsert to Navidrome
- `scheduler`: daily orchestration loop

## Local run

1. Copy `.env.example` to `.env` and fill credentials.
2. Start infra: `docker compose up -d postgres`.
3. Run app: `cargo run`.
4. Health check: `curl http://localhost:8080/health`.
