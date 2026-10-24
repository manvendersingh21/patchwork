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

### ChatGPT (anyone at the conference)

The HTTP server makes PatchWork a connector anyone can add — no install, no token:

1. Presenter: `make api-mcp` in one terminal, `make tunnel` in another — it prints a public URL like `https://…trycloudflare.com/mcp` (restart `api-mcp` with `MCP_ALLOWED_HOSTS=<that-hostname>`).
2. Attendee, in ChatGPT: **Settings → Connectors → Add new connector**, name `PatchWork`, paste the URL, no authentication.
3. Then just talk (or share photos):
   - "I just learned that hosted MCP servers should scope OAuth per identity — log it" → `log_learning`, which also tells you which talks cover it
   - Share a **photo of a slide or whiteboard** and say "remember what this teaches" → ChatGPT reads the photo and logs the lesson with topics (images never touch the server)
   - "Our agent keeps hallucinating repo structure — that's an open problem for me" → `log_issue`, and PatchWork's agents bridge it to the talks that address it within seconds
   - "What did I learn this week?" → `recall_learnings`

Each person's memory is keyed by the handle they give; memories are not shared between users.

### Security

- The HTTP server accepts an optional `MCP_BEARER_TOKEN` env var; when set, every request must carry `Authorization: Bearer <token>` or it gets a 401. Use it whenever the port is reachable beyond loopback.
- The HTTP server also supports `MCP_ALLOWED_HOSTS` (comma-separated) to allow additional hostnames, e.g. a tunnel domain.
- The stdio server inherits the caller's permissions — register it only for users you trust with the graph.
- Both servers read the repo `.env` for the FalkorDB connection. Keep `.env` out of configs, screenshots, and chat.

## LinkedIn follow-ups

PatchWork can remember LinkedIn DM threads the same way it remembers hallway chats. Your MCP client runs two servers side by side: PatchWork, and the third-party [linkedin-mcp-server](https://github.com/stickerdaniel/linkedin-mcp-server), pinned in `.mcp.json` to `mcp-server-linkedin==4.28.0`. The two servers never call each other.

1. Log in once (opens a browser window, and the session is saved to `~/.linkedin-mcp/profile/`): `uvx mcp-server-linkedin==4.28.0 --login`
2. Ask your client: *"Read my LinkedIn thread `2-ZjFi…==` with get_conversation, summarize it, and log_encounter it with place 'LinkedIn DM', the thread URL as source_url, and their profile slug as met_linkedin."* Pass the thread ID, the part after `/messaging/thread/`, not the whole URL.
3. The contact is stored as `person:li:<slug>`, so re-importing the same person adds a new encounter to the same node. PatchWork's agents bridge the thread's topics to your open issues.
4. When a matching issue lands (`log_issue`, or a real GitHub issue), `memory_recall` and `check_alerts` surface that person with place `LinkedIn DM`. Your client drafts the follow-up for you to copy.

Rules: import only threads you chose. Store a summary in `note`, never the raw transcript. **Never auto-send:** don't call the LinkedIn server's `send_message` or `connect_with_person` without explicit approval. Keep the LinkedIn server on stdio and local. Its HTTP mode has no auth and acts as your account, so never put it behind `make tunnel`. LinkedIn's User Agreement prohibits automated access, so use it sparingly. Demo fallback: `data/demo_linkedin_thread.json` (labeled demo persona).

## Share with teammates

The owner's `/mcp` can create read-only child MCP links from chat:

1. Ask your MCP client something like "share the talk tools with team-berlin, hide my notes". It calls `create_share`, which asks which tools to share (`talks_for_my_issues`, `talks_for_issue`, `people_to_meet`, `memory_recall`, `talks_by_topic`, `list_topics`) and whether to hide `note` / `place`. The reply includes a pasteable `url` and the `hidden_tools` the teammate won't see.
2. The teammate pastes that `url` (`<MCP_PUBLIC_URL>/share/<token>/mcp`) into their MCP client as a Streamable HTTP server. They see only the shared tools; write and owner tools (`log_*`, `create_share`, `list_shares`, `revoke_share`, ...) are rejected.
3. "Which shares are live?" → `list_shares` (shows only the first 8 characters of each token). "Revoke team-berlin" → `revoke_share`, and the link returns 401 right away.

Set `MCP_PUBLIC_URL` (e.g. your tunnel's `https://….trycloudflare.com`) so `url` uses the public host; otherwise it uses `http://$MCP_HTTP_ADDR`. When tunneled, `/mcp` must be protected with `MCP_BEARER_TOKEN`. Share URLs do not use that bearer: the token in the path is their credential, so treat a share URL like a password.

## MCP tools

All tools run fixed, pre-verified Cypher templates on FalkorDB — arguments are typed values only, and no client-supplied Cypher is ever accepted. Template-backed tools return an envelope with `template`, `hops`, `run_time_ms` (FalkorDB's own internal execution time, not wall clock), `rows`, and the `path`.

| Tool | What it returns | Notable args |
| --- | --- | --- |
| `talks_for_my_issues` | Talks covering the topics of a person's open issues, as a 5-hop path (Person→Repo→Issue→Topic→Talk→Speaker). Template T1. | `person` (required, e.g. `person:gh:<handle>`); optional `day` (`YYYY-MM-DD`), `after` and `before` (`HH:MM` 24h earliest/latest start; empty = any) |
| `talks_for_issue` | Talks covering the topics of one issue, with the connecting path. Template T2. | `issue` (required, e.g. `issue:<owner>/<repo>#<number>`) |
| `people_to_meet` | Speakers relevant to the person's open issues whom they haven't logged meeting yet (top 5). Template T3. | `person` (required) |
| `memory_recall` | People the user met who discussed the topics of one issue, newest first. Template T4. | `issue` (required) |
| `list_topics` | The closed topic list (slug + name). | none |
| `log_encounter` | Records a hallway conversation: creates the `Encounter`, the met `Person`, and `DISCUSSED` links to topics; returns `encounter_uid` / `contact_uid` with state `PARSED`. PatchWork's graph agents then link it to open issues. No LLM runs on the server — the calling model extracts who was met and which topics. | `person`, `note`, `met_name`, `topics` (required; `topics` must be non-empty slugs from `list_topics`, unknown slugs are rejected); optional `place`, `met_handle` / `met_linkedin` (leave empty if unknown — never guess), `source_url` (e.g. LinkedIn thread URL) |
| `check_alerts` | Bridges PatchWork found between your open issues and people you met (newest first, up to 10), with state and hop count. Draft follow-ups with your own model. | none |
| `log_learning` | Logs something learned — from a sentence or a photo the calling model reads — as a `Learning` node linked to the person and topics, and returns the talks that cover those topics. | `person`, `learned`, `topics` (required); `source` (optional) |
| `log_issue` | Logs an open problem the user describes as an issue on their personal notes repo, auto-classified under topics; the agents bridge it to talks and contacts within seconds. | `person`, `title`, `body`, `topics` (required) |
| `recall_learnings` | Everything the person logged as learned, newest first, optionally filtered to one topic. | `person` (required); `topic` (optional) |
| `talks_by_topic` | Talks about the given topics, with their path. Template T6. | `topics` (required, comma-separated slugs, e.g. `"mcp-auth,agentic-security"`); optional `day`, `after`, `before` (same format as above) |
| `create_share` | Owner only. Creates a scoped, read-only child MCP link; returns `url`, `mcp_path`, `tools`, `redact`, and `hidden_tools`. | `label`, `tools` (required, from the sharable read tools); `redact` (optional: `note`, `place`) |
| `list_shares` | Owner only. Live shares, newest first: label, tools, redact, created_at, 8-char `token_preview`. | none |
| `revoke_share` | Owner only. Deletes the shares with that label; returns `removed`. Their URLs return 401 at once. | `label` (required) |

## Repository

- `backend/` — Rust workspace: API, agents, Cypher templates, and the two MCP server binaries
- `frontend/` — Vite + React UI
- `data/snapshots/` — graph snapshots (`make export SNAP=…` / `make restore SNAP=…`)
- Demo repo synced into the graph: `manvendersingh21/deepcore-n1`
