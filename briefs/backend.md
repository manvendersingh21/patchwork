# Brief: Rust backend (agents + HTTP API) — codex

Read `CLAUDE.md` first, then `PLAN.md`. CLAUDE.md overrides PLAN.md where they differ
(Rust instead of Python, OpenAI via `OPENAI_API_KEY`/`OPENAI_MODEL`, `MAINTAINS|TRACKS`).

## You own (create/edit only these)
- `backend/src/templates.rs` (add T2–T6; keep T1 as is)
- `backend/src/llm.rs`, `backend/src/github.rs`, `backend/src/events.rs`, `backend/src/api.rs`, `backend/src/config.rs`
- `backend/src/agents/{mod.rs,cartographer.rs,pathfinder.rs,concierge.rs}`
- `backend/src/main.rs` (axum server on :8000 that also starts the three agent loops)
- `backend/src/lib.rs` (add module lines only)
- `backend/Cargo.toml` (allowed new crates: axum, tower-http (cors), reqwest (json, rustls), uuid, sha2, chrono, tracing, tracing-subscriber — nothing else without asking)
- `backend/tests/*.rs`

## Do not touch
- `backend/src/graph.rs` envelope types and `run_template()` signature (frozen). You may add new helper fns at the bottom (claim helpers).
- `backend/src/bin/seed.rs`, `data/*`, `frontend/*`, `.env`, `PLAN.md`, `CLAUDE.md`.
- Do NOT run `git commit`. Do not print `.env` values.

## Hard facts already verified
- falkordb 0.10.5 cannot decode Path values. Every template returns the path as
  `[n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes`
  and `[r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges`, plus `size(relationships(p)) AS hops`. See T1.
- Params are passed as strings; optional filters use `$x = ''` for "no filter". LIMIT must be an inlined literal.
- `WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s)` works in FalkorDB. `shortestPath` works.
- Live graph is seeded (`cargo run -p patchwork --bin seed`). Presenter uid: `person:gh:manvendersingh21`. DEMO_REPO `manvendersingh21/deepcore-n1` has issues #1–#4.

## Build, in this order; after each step run the check and stop if red
1. T2–T6 in templates.rs, each verified with a `#[ignore]` live test in `backend/tests/templates.rs`.
2. llm.rs: `classify(items) -> Vec<Vec<TopicHit{slug,confidence,evidence}>>` with JSON-schema enum of slugs from `data/topics.json` + "none", drop <0.5, ≤10 items per call, cache in `data/cache/llm.json` keyed by sha256(fn+model+input). Also `parse_note`, `draft_followup`, `answer`. OpenAI chat completions via reqwest.
3. github.rs: list top repos, open issues (skip PRs), and the DEMO_REPO watcher (cursor on Repo.watch_since). Use `GITHUB_TOKEN` if set, else `gh auth token` output read at startup (never logged).
4. Agents per PLAN.md "Three agents and the shared-state protocol": claim with one write query, AgentEvent per transition, startup recovery of stuck claims, 0.5 s idle sleep. Agents never `use` each other.
5. api.rs: every route in PLAN.md "Backend API contract". JSON shapes must match `frontend/src/mocks/*.json` exactly.
6. `/api/ask`: OpenAI tool calling over T1, T3, T4, T6; second call writes 1–3 sentences from rows only; refusal text when no tool fits.

## Acceptance (run all; paste output in your final message)
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$PWD/target-backend" RUSTFLAGS="-D warnings" cargo build -p patchwork`
- `... cargo test -p patchwork -- --ignored` (FalkorDB on localhost:6379 is running)
- `grep -rn "use crate::agents::\(cartographer\|pathfinder\|concierge\)" backend/src/agents` returns nothing
- With `cargo run -p patchwork` running: `curl -s localhost:8000/api/health` shows falkordb/openai/github all true;
  `POST /api/encounters` with the Act 2 note produces an ACTION_READY contact BRIDGE within 15 s after
  `POST /api/demo/inject-issue` with the Act 3 issue text.
