# PatchWork: hackathon demo runbook

Four acts, about four minutes. Presenter at the keyboard, teammate with a phone or second laptop for Act 4. Tool names below match `backend/src/mcp.rs`.

Shell setup for every command below (loads `.env` into the shell without printing it):

```bash
cd /Users/manubaba/patchwork && set -a && . ./.env && set +a
```

Replace `<tunnel host>` with the cloudflared host (no `https://`), and `<thread-id>` with the part after `/messaging/thread/`.

## 1. Pre-flight checklist

Run all of these 10 to 15 minutes before going on stage.

| # | Check | Command | Expected |
| --- | --- | --- | --- |
| 1 | API and agents up, graph loaded | `curl -s http://127.0.0.1:8000/api/health` | 200 with JSON; graph counts are non-zero |
| 2 | UI proxies the API | `curl -s http://localhost:5173/api/health` | Same JSON as check 1 |
| 3 | UI renders the graph | Open http://localhost:5173 | Graph visible, counts in header |
| 4 | MCP HTTP up locally, owner gated | `curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8765/mcp` | `401` (token is set) |
| 5 | Tunnel rejects no token | `curl -s -o /dev/null -w '%{http_code}\n' https://<tunnel host>/mcp` | `401` |
| 6 | Both MCP servers connected in Claude Code | `claude mcp list` | `patchwork` and `linkedin` both show connected |
| 7 | Share links work end to end | In owner chat: "create a share labeled rehearsal with memory_recall, hide note and place". Then `curl -s -o /dev/null -w '%{http_code}\n' <rehearsal url>` | `200`. Afterwards "revoke rehearsal", then the same curl gives `401` |
| 8 | LinkedIn MCP reachable only with token | `curl -s -o /dev/null -w '%{http_code}\n' "$LINKEDIN_MCP_URL"` then the same with `-H "Authorization: Bearer $LINKEDIN_BEARER_TOKEN"` | `401` without, `200` with |
| 9 | Demo fallback file present | `ls data/demo_linkedin_thread.json` | File listed |
| 10 | Clean slate | Owner chat: "list_shares" | Only the shares you expect (none, before Act 4) |

If any check fails, see section 3 before going on stage.

## 2. The four-minute script

### Act 1: talks for my issues (about 1 min)

Type in Claude Code (owner `/mcp`):

> Which talks on day 2 morning help with my open issues?

Says the client: `talks_for_my_issues` with `day` 2026-10-29 and an earliest start of 10:00.

**Audience sees:** a 5-hop path, Me > Repo > Issue > Topic > Talk > Speaker. The first row is U26-09 "Open pull requests, don't merge them" (Nick Taylor, Pomerium, 10:30, Demo Stage), reached from issue #1 through `mcp-auth`. Point at the `hops` and `run_time_ms` fields in the envelope.

### Act 2: LinkedIn thread to memory (about 1 min)

Both servers are on in Claude Code. Type:

> Read my LinkedIn thread `<thread-id>` with get_conversation, summarize it in two sentences, and log_encounter it with place "LinkedIn DM", source_url set to the thread URL, and met_linkedin set to their /in/ slug from the thread.

Then type:

> Run check_alerts.

**Audience sees:** the client reads the thread (reads only, nothing sent), then `log_encounter` returns `encounter_uid` and state `PARSED`. After the agents link it, `check_alerts` shows a bridge from the recruiter, Sudhir Madnawat, to issue #3 (CI/CD), with place "LinkedIn DM".

Say out loud: "Summary only in the graph, no raw transcript."

### Act 3: alert card and send (about 1 min)

Switch to the UI at http://localhost:5173. The alert card for the Sudhir bridge shows an editable draft.

Say: "Copy is the default. Send exists, and it only goes out after a second confirm."

Click **Copy** first. Then, only if you really mean to message that person, click **Send on LinkedIn**, then **Confirm**. Otherwise click **Dismiss**.

> Say this before pressing Confirm: "Only press Confirm if you really mean to message that person. Otherwise Copy."

**Audience sees:** draft, copy, and (if sent) a single confirmed message. Guards to mention: the recipient comes from the bridge's contact on the server, not from the page; each bridge sends at most once; a contact with no LinkedIn id shows a clear error.

If you do not intend to send in front of the audience, skip Confirm and say so.

### Act 4: share memory with a teammate, then revoke (about 1.5 min)

Owner chat, type:

> Create a share for Priya with memory_recall and talks_for_issue, hide note and place.

The client calls `create_share` and returns a `url` plus `hidden_tools`.

Teammate, in a second client (Codex or Claude, not the owner's session), adds the URL as an MCP server with no auth. In Codex the connection is:

```bash
codex exec --skip-git-repo-check -c 'mcp_servers.patchwork.enabled=false' -c 'mcp_servers.priya.url="<share url>"' -c 'mcp_servers.priya.default_tools_approval_mode="approve"' "List your tools, then run memory_recall for issue:manvendersingh21/deepcore-n1#1"
```

Teammate prompt: "List your PatchWork tools, then run memory_recall for issue:manvendersingh21/deepcore-n1#1."

**Audience sees:** only `memory_recall` and `talks_for_issue` in `tools/list`. Results have no `note` or `place` fields.

Owner chat:

> list_shares

Then:

> Revoke priya.

Teammate runs the same prompt again. **Audience sees:** `401`, the link is dead at once.

## 3. Fallbacks

**Act 1 fails (no result or error):** check `curl -s http://127.0.0.1:8000/api/health`. If the graph is empty, run `make restore SNAP=synced`, then ask the question again.

**Act 2: LinkedIn down, login or captcha fails:** use the demo persona thread instead:

> Read data/demo_linkedin_thread.json (the demo persona thread), summarize it in two sentences, and log_encounter it with place "LinkedIn DM", using the source_url, met_name and met_linkedin fields from the file.

Say out loud: "This is the labeled demo thread." Copy works regardless; Send needs the LinkedIn server, so it will fail. Use Copy.

**Tunnel URL changed (quick tunnels reset on restart):**

1. Stop `mcp_http` (Ctrl-C in its terminal).
2. Run `make tunnel` and note the new host it prints.
3. Restart: `MCP_ALLOWED_HOSTS=<new host> MCP_PUBLIC_URL=https://<new host> ./target/debug/mcp_http`
4. Re-create the share, since its URL carries the host. Update the teammate's connector, and the claude.ai connector if you use it.
5. Re-run checks 5 and 7.

**UI blank:** open the browser console on :5173 and read the error. Confirm check 1 returns JSON. If the API is down, restart with `./target/debug/patchwork` (or `make api`), then reload. As a last resort, show the Act 1 answer in the terminal, which is the same envelope.

**Act 3 alert does not appear:** ask in chat "check_alerts" and show the bridge in the terminal. Say that the card is the same data.

**Act 4 teammate fails before revoke:** re-check `list_shares`. Confirm the share URL with check 7 curl. If the tunnel host changed, use the tunnel-changed steps above.

## 4. After the demo

1. Rotate `LINKEDIN_BEARER_TOKEN`. It was pasted in chat, so treat it as exposed. Update `.env` and restart the LinkedIn server on the Mac mini.
2. Rotate `MCP_BEARER_TOKEN` in `.env`, restart `mcp_http`, and update any claude.ai or Codex connector headers.
3. Revoke every share: run `list_shares`, then `revoke_share` for each label (for example `priya`, and `rehearsal` if left over).
4. Stop the tunnel: `pkill cloudflared`. Stop `mcp_http`, `patchwork`, and the Vite dev server.
5. Delete `/tmp/cloudflared.log` (it holds the public URL).

## 5. Known limits

- Quick tunnel URLs change on every restart of cloudflared. Every share and connector breaks until updated.
- `send_message` opens a new DM. It does not reply inside the existing thread.
- LinkedIn automation can get an account restricted, and the User Agreement prohibits automated access. Import only threads you chose, and keep the live demo short.
- Only the LinkedIn server's `send_message` sends anything. The "Send on LinkedIn" button is the only path that calls it. Never call `connect_with_person` on stage.
- Share URLs are passwords. Do not show the token in a screenshot.
