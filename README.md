# PatchWork

PatchWork is conference-memory on a FalkorDB graph: your open GitHub issues, the conference schedule, and the hallway conversations you log all live as one graph. A crew of agents coordinates **only** through that graph — no agent-to-agent chat — so every answer is an auditable path, not a vibe. Answers to questions like *"which talks should I attend for my issues?"* come back as 5-hop paths (Person→Repo→Issue→Topic→Talk→Speaker) with proof attached: the fixed Cypher template that ran, the hop count, and FalkorDB's own reported execution time. Clients never write Cypher — they call typed MCP tools, and the server runs pre-verified templates.

## Quickstart

```bash
make api                    # backend API on http://127.0.0.1:8000
make ui                     # Vite dev server on http://localhost:5173 (proxies /api → :8000)

make seed                   # load the demo graph into FalkorDB
make smoke                  # run the smoke checks against the seeded graph
make restore SNAP=baseline  # reset the graph to the baseline snapshot
```

## Connect an MCP client

PatchWork ships two MCP server binaries exposing the same tools:

- **stdio** (`backend/src/bin/mcp.rs`) — MCP over stdin/stdout, for Claude Code, Cursor, Codex, and other local clients.
- **Streamable HTTP** (`backend/src/bin/mcp_http.rs`) — listens on `MCP_HTTP_ADDR` (default `127.0.0.1:8765`) at path `/mcp`, for remote clients.

Build the binaries first:

```bash
cargo build
```

Both servers call `load_dotenv()` at startup and read the repo's `.env` (FalkorDB Cloud connection) themselves — your MCP client needs **no** environment configuration. Never copy `.env` values into client configs.

### Claude Code (stdio)

```bash
claude mcp add patchwork -- /Users/manubaba/patchwork/target/debug/mcp
```

### Claude Code (HTTP alternative)

Start the HTTP server in one terminal:

```bash
cargo run --bin mcp_http
# patchwork MCP (streamable HTTP) on http://127.0.0.1:8765/mcp
```

Then register it:

```bash
claude mcp add --transport http patchwork http://127.0.0.1:8765/mcp
```

### Cursor

Create `.cursor/mcp.json` in the project:

```json
{
  "mcpServers": {
    "patchwork": {
      "command": "/Users/manubaba/patchwork/target/debug/mcp",
      "cwd": "/Users/manubaba/patchwork"
    }
  }
}
```

### Codex CLI

Add to `~/.codex/config.toml`:

```toml
[mcp_servers.patchwork]
command = "/Users/manubaba/patchwork/target/debug/mcp"
```

This machine already has PatchWork registered in `~/.codex/config.toml` — nothing to do.

### Security

- The HTTP server accepts an optional `MCP_BEARER_TOKEN` env var; when set, every request must carry `Authorization: Bearer <token>` or it gets a 401. Use it whenever the port is reachable beyond loopback.
- The HTTP server also supports `MCP_ALLOWED_HOSTS` (comma-separated) to allow additional hostnames, e.g. a tunnel domain.
- The stdio server inherits the caller's permissions — register it only for users you trust with the graph.
- Both servers read the repo `.env` for the FalkorDB connection. Keep `.env` out of configs, screenshots, and chat.

## MCP tools

All tools run fixed, pre-verified Cypher templates on FalkorDB — arguments are typed values only, and no client-supplied Cypher is ever accepted. Template-backed tools return an envelope with `template`, `hops`, `run_time_ms` (FalkorDB's own internal execution time, not wall clock), `rows`, and the `path`.

| Tool | What it returns | Notable args |
| --- | --- | --- |
| `talks_for_my_issues` | Talks covering the topics of a person's open issues, as a 5-hop path (Person→Repo→Issue→Topic→Talk→Speaker). Template T1. | `person` (required, e.g. `person:gh:<handle>`); optional `day` (`YYYY-MM-DD`), `after` and `before` (`HH:MM` 24h earliest/latest start; empty = any) |
| `talks_for_issue` | Talks covering the topics of one issue, with the connecting path. Template T2. | `issue` (required, e.g. `issue:<owner>/<repo>#<number>`) |
| `people_to_meet` | Speakers relevant to the person's open issues whom they haven't logged meeting yet (top 5). Template T3. | `person` (required) |
| `memory_recall` | People the user met who discussed the topics of one issue, newest first. Template T4. | `issue` (required) |
| `list_topics` | The closed topic list (slug + name). | none |
| `log_encounter` | Records a hallway conversation: creates the `Encounter`, the met `Person`, and `DISCUSSED` links to topics; returns `encounter_uid` / `contact_uid` with state `PARSED`. PatchWork's graph agents then link it to open issues. No LLM runs on the server — the calling model extracts who was met and which topics. | `person`, `note`, `met_name`, `topics` (required; `topics` must be non-empty slugs from `list_topics`, unknown slugs are rejected); optional `place`, `met_handle` (leave empty if unknown — never guess) |
| `check_alerts` | Bridges PatchWork found between your open issues and people you met (newest first, up to 10), with state and hop count. Draft follow-ups with your own model. | none |
| `talks_by_topic` | Talks about the given topics, with their path. Template T6. | `topics` (required, comma-separated slugs, e.g. `"mcp-auth,agentic-security"`); optional `day`, `after`, `before` (same format as above) |

## Repository

- `backend/` — Rust workspace: API, agents, Cypher templates, and the two MCP server binaries
- `frontend/` — Vite + React UI
- `data/snapshots/` — graph snapshots (`make export SNAP=…` / `make restore SNAP=…`)
- Demo repo synced into the graph: `manvendersingh21/deepcore-n1`
