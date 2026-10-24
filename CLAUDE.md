# PatchWork: working rules

Full build plan is in `PLAN.md`. These rules apply to every session.

## Decisions changed (2026-10-24)
- Backend is Rust: axum + tokio, `falkordb` crate with the `tokio` feature, reqwest, serde.
- Frontend stays Vite + React + TypeScript (react-force-graph-2d).
- LLM is OpenAI (`OPENAI_API_KEY`, `OPENAI_MODEL`). No local model for now.
- T1 and T3 match `-[:MAINTAINS|TRACKS]->`. `Person-[:TRACKS]->Repo` is for repos the presenter follows without maintaining. `TRACKED_REPOS` in `.env` lists them.

## Rules
1. Use the stack in this file. Ask before adding a dependency or changing a decision.
2. Build the smallest version that passes the acceptance check. No speculative abstractions.
3. Run every Cypher template against live FalkorDB before wiring it into code.
4. The LLM never writes Cypher at runtime. It picks a template and fills its parameters.
5. Only `scripts/seed.py` (or its Rust equivalent) creates Topic nodes. Once reviewed, `data/topics.json` is frozen.
6. Agents never import or call each other. They coordinate only through FalkorDB state.
7. Commit after every passing acceptance check, with a one-line message naming the task.
8. If a task is stuck for 15 minutes, stop and report two or three options.
9. Keep secrets in `.env`. Never commit it or print keys.

## Commands
- Build: `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$PWD/target" RUSTFLAGS="-D warnings" cargo build -p patchwork`
- Live graph tests: `cargo test -p patchwork -- --ignored`
- DB: `docker run -d --name falkordb -p 6379:6379 -p 3000:3000 falkordb/falkordb:latest`
