# PatchWork — stage demo script

Three minutes, three acts. Presenter at the keyboard, one teammate holding the phone.
Demo repo: `manvendersingh21/deepcore-n1` (4 real open issues). Storyline topic for all three acts: `mcp-auth`.

## Storyline texts

Write these once, pre-classify them so the LLM cache is warm, and never retype them on stage.

**Act 1 issue** — already open as issue #1 in the demo repo (real text, verbatim):

> Title: Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)
>
> Body: Hosted MCP tool calls currently run with one shared token. Scope permissions per calling identity so an agent can open PRs but cannot merge them.

<https://github.com/manvendersingh21/deepcore-n1/issues/1> · must land on `mcp-auth` (also `agentic-security`, `oauth`).

**Act 2 hallway note** (teammate: Vasisht):

> Met Vasisht at the snack table. They scope agent tokens so agents can open PRs but never merge them.

Place: `Snack table`. Must land on `mcp-auth`.
The repo has no `data/people.json`, so there is no real teammate on file yet. Before Act 2: pick a real person
who actually discussed MCP authorization with you that day, get their consent, sync them as a guest, and replace
the placeholder. Do not run with the angle brackets still in the note — the parser would create a Person named
Vasisht.

**Act 3 issue** — opened live from the phone during the demo:

> Title: Agents can merge PRs through our MCP server; we need per-identity scopes
>
> Body: Our hosted MCP server lets an agent merge its own pull request. We need scopes per calling identity so agents can open PRs but never merge them.

Must land on `mcp-auth`. Keep this as a prefilled new-issue link open on the phone:
`https://github.com/manvendersingh21/deepcore-n1/issues/new?title=...&body=...`

## The three acts

**0:00–0:20 — Hook.** "Conference apps die on Monday. Ours wakes up on Monday." Point at the graph: real Universe
sessions, speakers and topics, with counts in the header.
*Fallback:* nothing is live yet, so there is nothing to fail.

**0:20–1:10 — Act 1: codebase to talks.** Type the presenter's handle, click Sync. The ticker shows Cartographer
pulling repos and classifying issues. Then click the Act 1 chip. A 5-hop path lights up
(Me › Repo › Issue › Topic › Talk › Speaker). Open Graph proof: the Cypher, 5 hops, FalkorDB's own time.
*Fallback:* if the sync stalls, run `make restore SNAP=synced`, then click the chip.

**1:10–1:50 — Act 2: hallway to graph.** "Earlier today I met \<teammate\>." Paste the hallway note and submit.
The Encounter node appears, Cartographer parses it, Pathfinder bridges it to open issue #1 with no LLM call,
Concierge drafts the follow-up.
*Fallback:* the note's LLM calls are cached from rehearsal, so it replays even if OpenAI is unreachable.

**1:50–2:50 — Act 3: weeks later.** "Three weeks later." Ctrl-C the backend in the visible terminal and run
`make api`; the chat is empty. The teammate taps Submit on the prefilled issue. Within about 10 s the ticker runs
all three agents and the alert slides in: the person you met, the memory path, and a ready follow-up.
*Fallback:* no pickup after 20 s, press Alt+I to inject the same issue locally — and say out loud that you did.

**2:50–3:00 — Close.** "Five-hop traversals, memory that survived a restart, three agents that only talk through
the graph, and every answer shows its path." Leave the memory path highlighted.
*Fallback:* none needed; the highlight is already on screen.

## Act 1 chip question

> Which talks on day 2 morning help with my open issues?

Resolves to T1 `talks_for_my_issues` with `day = "2026-10-29"`, `after = "10:00"`, `person = person:gh:<presenter>`.

Expected first row from the current `data/sessions.json`: **U26-09 "Open pull requests, don't merge them:
Fine-grained authorization for hosted MCP servers"**, Nick Taylor (Pomerium), 2026-10-29 10:30, Demo Stage
(Festival Pavilion) — reached from issue #1 through `mcp-auth`.

Caveat to know before you click: "morning" is now enforced as an upper bound — the server derives
`before = "12:00"` from the word "morning" in the question, so afternoon talks no longer sneak in. Asking
about a day that doesn't exist (e.g. "day 3" or "friday") returns the conference's actual days
(2026-10-28 and 2026-10-29) instead of silently clamping to the closest real day.

## Real vs staged

Say this out loud during the demo.

**Real:** the Universe sessions, speakers, days, times and rooms in `data/sessions.json`; the repo
`manvendersingh21/deepcore-n1`; its four open issues (#1–#4), filed on 2026-10-24 and all genuinely ours to work
on; the Act 3 issue, created live on GitHub from the phone; the presenter, the catalog speakers, and the Act 2
teammate, who is a real person who gave consent.

**Staged:** the three-week jump. It is a backend restart, not a clock change — FalkorDB keeps the memory across
the restart and the agents resume their claims. Nothing else about Act 3 is simulated.

## Data check: data/sessions.json

`python3 -c "import json;json.load(open('data/sessions.json'))"` exits 0, and all 22 topics in `topics.json` are
covered by at least one session. Against the PLAN rules, these problems remain — `data/` is read-only for this
brief, so none of them were changed:

- **Every one of the 10 sessions is missing `abstract` and `url`.** The PLAN session shape and the Talk node both
  require them. No `abstract` means `classify()` sees only the title, and no `url` means the answer cards have
  nothing to link to.
- **Only 10 sessions, against the Phase 0 bar of 30 to 40.** Five PLAN anchor sessions are absent: "How GitHub
  taught Copilot to remember and when to forget" (the agent-memory pitch tie-in), "Build once, run on any agent",
  "Your benchmark is lying", "One CLI to replace your entire JavaScript toolchain", and "Lessons from building
  tech in a low-connectivity community".
- **Day, time and room are present on all 10 sessions but unverified.** The file's own `source` field says
  "user-supplied […] not verified against RainFocus". The rule is never to invent a day, time or room, so each of
  these needs checking against the official catalog before seeding, or the session stays out of T1 and T6.
- **U26-01 has no real speaker.** `{"name": "GitHub Leadership", "company": "GitHub", "link": false}` would create
  a Person node that is not a person, and `link` is not a field in the schema (the schema expects `github`, null
  when unknown). Either name the real keynote speakers or leave the session without a SPEAKS_AT edge — which also
  keeps it out of T1, T3 and T6, since all three traverse SPEAKS_AT.

No session is missing `sid`, `title`, `day`, `start_time`, `end_time`, `room`, `speakers` or `topics`.
