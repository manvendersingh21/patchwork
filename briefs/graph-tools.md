# Brief: snapshot and smoke tools — agy

Read `CLAUDE.md` and the "Command" table and "Hardening" section of `PLAN.md`.

## You own (create only these)
- `backend/src/bin/snapshot.rs` — export and restore the whole graph (`patchwork` graph on localhost:6379) as JSON
- `backend/src/bin/smoke.rs` — run every template in `backend/src/templates.rs` and report pass/fail
- `data/snapshots/` (directory for output)

## Do not touch
- `backend/src/graph.rs`, `backend/src/templates.rs`, `backend/Cargo.toml`, `backend/src/lib.rs`, `backend/src/main.rs`, anything under `backend/src/agents/`, `backend/src/llm.rs`, `backend/src/github.rs`, `backend/src/events.rs`, `backend/src/config.rs`, `frontend/`, `data/*.json`, `.env`.
- Do NOT add dependencies. Use only what `backend/Cargo.toml` already has (falkordb, tokio, serde, serde_json, anyhow, futures-util). If you need one more, stop and say so.
- Do NOT run `git commit`.

## Facts
- Export: nodes (every label, all properties, uid) and edges (type, src uid, dst uid, properties). Query with `MATCH (n) RETURN labels(n), properties(n)` and `MATCH (a)-[r]->(b) RETURN a.uid, type(r), b.uid, properties(r)`. Skip AgentEvent nodes.
- Restore: delete the graph, then recreate nodes and edges by uid with MERGE. Re-running restore must be safe.
- Smoke: a list of (name, cypher, sample params). Run each, exit non-zero if any errors. Only read `templates.rs` constants; do not copy Cypher into this file.

## Acceptance (paste output in your final message)
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$PWD/target-graph" RUSTFLAGS="-D warnings" cargo build -p patchwork --bins`
- `cargo run -q -p patchwork --bin snapshot -- export data/snapshots/baseline.json` then `... restore data/snapshots/baseline.json`, then export again; the two exports must match in node and edge counts.
- `cargo run -q -p patchwork --bin smoke` exits 0 (T1 at least must pass).
