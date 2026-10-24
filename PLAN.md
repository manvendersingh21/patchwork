PatchWork — Build Plan for Claude Code
Oct 24, 2026 · @your mama
How to use this plan with Claude Code
Download this doc as Markdown, save it as PLAN.md at the root of an empty repo, and give each workstream its own Claude Code session.
Starter prompt for each session (change the workstream letter and name):
Read PLAN.md end to end. You own workstream B (agents and backend).
Do the Phase 0 and Phase 1 tasks for B only, in order.
After each task, run its acceptance check and show me the output.
Stop at every gate and wait for my go-ahead.
Operating rules for the agent:
1. Use the stack and decisions in this plan. Ask before adding a dependency or changing a decision.
2. Build the smallest version that passes the acceptance check, then stop. No speculative abstractions.
3. Run every Cypher template against the live FalkorDB before wiring it into code. FalkorDB's Cypher differs from Neo4j's in places: on failure, try the fallback form given here, then the FalkorDB docs.
4. The LLM never writes Cypher at runtime. It picks a template and fills its parameters.
5. Only scripts/seed.py creates Topic nodes. Once reviewed, data/topics.json is frozen.
6. Agents never import or call each other. They coordinate only by reading and writing state in FalkorDB.
7. Commit after every passing acceptance check, with a one-line message naming the task.
8. If a task is stuck for 15 minutes, stop and report two or three options instead of thrashing.
9. Keep secrets in .env. Never commit it or print keys.
Put these rules in CLAUDE.md at the repo root. Claude Code loads it at the start of every session, including sessions launched inside backend/ or frontend/, and re-reads it after /compact (docs). Refer to the plan as PLAN.md in backticks rather than as an @ import, which would load the whole plan into every session. Run /context to confirm the file loaded.
Mission and how judges score it
PatchWork wins Track 1 by making FalkorDB do three visible jobs on stage: 5-hop traversals, memory that survives a restart, and three agents that coordinate only through the graph.
One-line pitch: "Conference apps die on Monday. PatchWork reads your open issues, maps them to the Universe talks and people that solve them, remembers every hallway conversation, and wakes up weeks later when a matching issue lands."
The three acts (full script in the demo section):
1. Codebase to talks. Sync a real GitHub handle and ask what to attend after lunch. A 5-hop path lights up: Me → Repo → Issue → Topic → Talk → Speaker.
2. Hallway to graph. Log a one-line note about someone you met. Three agents hand off through node and edge state until a bridge to one of your open issues appears.
3. Weeks later. Restart the backend on stage, open a real issue from a phone, and the app surfaces the person you met, the memory path, and a drafted follow-up.
Judging criterion
What proves it on stage
Graph at the core
Parameterized multi-hop Cypher as the agents' tools; memory across a restart; agents share state only through FalkorDB
Explainable answers
Every answer returns its path; the graph pane highlights it; a drawer shows the Cypher, hop count and FalkorDB's own execution time
Real conference problem
Attendees pick talks by their actual open issues and stop losing hallway context after the event
Working demo
Real Universe sessions, a real repo, a real issue created live, and a fallback for every step
Scope: non-negotiables, cuts, stretch goals
Ship a narrow demo that never breaks rather than a broad one that might; nothing under Stretch starts until the Phase 4 gate passes.
Non-negotiables:
• Every answer carries its graph path and the Cypher that produced it.
• Agents coordinate only through state on nodes and edges in FalkorDB.
• One closed topic list. Nothing creates Topic nodes at runtime.
• The LLM picks templates, classifies text and writes prose. It never writes Cypher or invents a connection.
• Every person, talk and issue on screen is real, or labeled on screen as a demo persona.
• Every live step has a fallback (see the demo script).
Do not build:
• Voice input. A text box feeds the graph the same way.
• Login, accounts, cloud deployment.
• Sending email or Slack messages. Draft plus a Copy button is enough.
• Vector search, LinkedIn scraping, or attendee data nobody gave you.
• A 3D graph or a mobile layout.
• Side quests for other tracks (Qodo, Bright Data prizes).
Stretch, in priority order:
1. Dependency path: manifest → Package → its maintainers → their Universe talk. Structured data, no LLM.
2. explain_connection template for "how am I connected to X?"
3. A QR page so people in the room can add their own GitHub handle.
4. Voice notes via speech-to-text.
Stack, repo layout and commands
Everything runs locally: FalkorDB in Docker, a Python FastAPI backend that also hosts the three agents, and a Vite React frontend. Only OpenAI and the GitHub API go over the network.
Layer
Choice
Notes
Graph
FalkorDB in Docker (falkordb/falkordb)
Port 6379 is the database, 3000 is FalkorDB Browser. Pull the image once in the morning; never re-pull on demo day
Backend
Python 3.11+, FastAPI, Uvicorn, falkordb, openai, httpx, pydantic v2, python-dotenv
Sync falkordb client wrapped in asyncio.to_thread
Agents
Three asyncio loops started on FastAPI startup
One module each; they share library code but never import each other
LLM
OpenAI on the event credits
Structured outputs for classification and parsing; tool calling for /api/ask
Frontend
Vite, React, TypeScript, Tailwind, react-force-graph-2d
Polls the API; mock mode for parallel work
patchwork/
  PLAN.md
  Makefile
  .env.example
  backend/
    app/
      main.py            # FastAPI app; starts the three agent loops on startup
      config.py          # env vars
      graph.py           # FalkorDB connection, run_template(), path serializer, claim helpers
      templates.py       # the Cypher templates: the agents' only read tools
      llm.py             # classify(), parse_note(), draft_followup(), answer()
      github.py          # repos, open issues, watcher
      taxonomy.py        # loads data/topics.json, builds the enum
      events.py          # writes AgentEvent nodes
      api.py             # routes
      agents/
        cartographer.py
        pathfinder.py
        concierge.py
    scripts/
      build_taxonomy.py
      seed.py
      snapshot.py        # export / restore the whole graph as JSON
      smoke.py           # runs every template and the golden tests
    data/
      sessions.json      # real Universe sessions
      topics.json        # frozen taxonomy
      people.json        # team and opted-in people
      cache/             # classification cache
      snapshots/
    tests/
      test_taxonomy.py
  frontend/
    src/
      App.tsx
      api.ts             # typed client; VITE_MOCK=1 serves src/mocks/*.json
      mocks/
      components/
        GraphPane.tsx
        AskPane.tsx
        PathBadges.tsx
        GraphProofDrawer.tsx
        AgentTicker.tsx
        AlertCard.tsx
        EncounterBox.tsx
# .env.example
FALKORDB_HOST=localhost
FALKORDB_PORT=6379
GRAPH_NAME=patchwork
OPENAI_API_KEY=
OPENAI_MODEL=              # fast model covered by the event credits
GITHUB_TOKEN=              # fine-grained, read-only; avoids the 60/hour anonymous limit
PRESENTER_HANDLE=          # GitHub handle of the person on stage
DEMO_REPO=owner/name       # real repo with real open issues; the watcher polls it
WATCH_INTERVAL_SEC=5
# make db
docker run -d --name falkordb -p 6379:6379 -p 3000:3000 \
  -v falkordb_data:/var/lib/falkordb/data falkordb/falkordb:latest
Confirm the data path against the FalkorDB README. make snapshot is the real safety net either way.
Command
What it does
make db
Starts the FalkorDB container
make taxonomy
Drafts data/topics.json from the sessions; a human reviews it next
make seed
Idempotent: indexes, topics, talks, speakers, COVERS and SPEAKS_AT edges
make api
FastAPI plus the three agents on port 8000
make web
Frontend on port 5173
make smoke
Runs every template and the golden taxonomy tests; non-zero exit on failure
make snapshot / make restore
Exports or imports the whole graph as JSON (SNAP=baseline by default, or SNAP=synced) in data/snapshots/
make demo-reset
Restores the baseline snapshot and moves the repo watcher's cursor to now
Graph schema and state machines
Every node and every BRIDGE edge carries a unique uid with a type prefix, and every write uses MERGE on it, so seeding and syncing are always safe to re-run.
Label
uid format
Properties
Written by
Person
person:gh:<handle> or person:name:<slug>
name, handle, company, kind (self, speaker, guest, contact), demo (bool), state
seed, API, Cartographer
Repo
repo:<owner>/<name>
full_name, url, description, stars, watch_since
Cartographer
Issue
issue:<owner>/<name>#<number>
number, title, body (first 2,000 chars), url, gh_state, created_at, state
Cartographer
Topic
topic:<slug>
slug, name, description
seed only
Talk
talk:<session id>
title, abstract, day (YYYY-MM-DD), start_time and end_time (HH:MM, 24h), room, url
seed
Encounter
enc:<uuid>
note, place, met_at (ISO 8601), state
API writes it raw; Cartographer parses it
AgentEvent
evt:<uuid>
ts (ms), agent, action, target, detail
all agents; hidden from the graph pane
Edge
From → To
Properties
Written by
MAINTAINS
Person → Repo

Cartographer
HAS_ISSUE
Repo → Issue

Cartographer
ABOUT
Issue → Topic, Repo → Topic
confidence, evidence
Cartographer
COVERS
Talk → Topic
confidence
seed
SPEAKS_AT
Person → Talk

seed
LOGGED
Person (self) → Encounter

API
MET
Encounter → Person

Cartographer
DISCUSSED
Encounter → Topic
confidence, evidence
Cartographer
MENTIONED_REPO
Encounter → Repo

Cartographer
BRIDGE
Issue → Talk or Person
uid, kind (talk, contact), via_topic, hops, path_uids, score, state, draft, seen, created_at
Pathfinder, then Concierge
Naming rules that prevent parser trouble: state is the pipeline state and GitHub's open/closed lives in gh_state. Avoid Cypher keywords as property names, which is why the schema says place, met_at, start_time and end_time instead of where, when, start and end.
Entity
State flow
Who moves it
Person (sync)
SYNC_REQUESTED → SYNCING → SYNCED
API, then Cartographer
Issue
NEW → CLASSIFYING → CLASSIFIED → LINKING → LINKED
Cartographer, then Pathfinder
Encounter
NEW → PARSING → PARSED → LINKING → LINKED
API, Cartographer, then Pathfinder
BRIDGE
DISCOVERED → DRAFTING → ACTION_READY
Pathfinder, then Concierge; the UI sets seen
Indexes, created by seed.py (verify the syntax in FalkorDB Browser first):
CREATE INDEX FOR (n:Person) ON (n.uid)
CREATE INDEX FOR (n:Repo) ON (n.uid)
CREATE INDEX FOR (n:Issue) ON (n.uid)
CREATE INDEX FOR (n:Issue) ON (n.state)
CREATE INDEX FOR (n:Topic) ON (n.uid)
CREATE INDEX FOR (n:Talk) ON (n.uid)
CREATE INDEX FOR (n:Encounter) ON (n.uid)
CREATE INDEX FOR (n:Encounter) ON (n.state)
CREATE INDEX FOR (n:AgentEvent) ON (n.ts)
Topic taxonomy (make-or-break)
Every multi-hop path runs through Topic nodes, so talks, issues, repo descriptions, hallway notes and user questions must all be classified into one closed list of 30 to 40 topics. If an issue lands on "MCP authorization" and a talk on "hosted MCP auth" as two separate nodes, the path breaks and the demo returns nothing.
Pipeline:
1. make taxonomy runs scripts/build_taxonomy.py. For each session in data/sessions.json, the LLM returns 2 to 4 short topic phrases (structured output).
2. One more LLM call clusters all phrases into 30 to 40 canonical topics shaped {slug, name, description, aliases}. Specific beats broad: "MCP authorization", not "AI".
3. A human reviews data/topics.json for 10 minutes, merging near-duplicates, renaming and deleting. Then it is frozen.
4. classify(items) is the only classifier, used for every kind of text. Its output schema is a JSON-schema enum of the frozen slugs plus none. It returns 0 to 3 topics per item as {slug, confidence, evidence}, drops anything under 0.5 confidence, and limits evidence to 12 words copied from the input. The UI shows evidence on hover.
5. Classify up to 10 items per call. Cache results by sha256 of (taxonomy version + text) in data/cache/classify.json, so reruns are instant and identical.
Acceptance checks:
[ ] tests/test_taxonomy.py passes: 10 hand-written issue texts, written after the review, map to their expected slugs.
[ ] Every topic has at least one talk through COVERS. Merge or delete topics with none.
[ ] At least 3 open issues in DEMO_REPO reach a talk through T1.
[ ] The Act 2 note and the Act 3 issue text are pre-classified (cache warm) and both land on the storyline topic.
Cypher tool templates
Seven parameterized templates are the only way the agents and /api/ask read the graph. Every run returns one envelope: rows, the path, the Cypher text, the hop count and FalkorDB's execution time.
If FalkorDB rejects LIMIT $limit, inline a validated integer instead. Expose T1, T3, T4 and T6 to /api/ask as OpenAI function tools.
T1 talks_for_my_issues (Act 1, 5 hops):
MATCH p = (me:Person {uid: $person})-[:MAINTAINS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})
          -[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person)
WHERE ($day IS NULL OR k.day = $day) AND ($after IS NULL OR k.start_time >= $after)
RETURN p, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day,
       k.start_time AS start_time, k.room AS room, s.name AS speaker
ORDER BY day, start_time
LIMIT $limit
For a person with no open issues, run the repo-level variant: replace the HAS_ISSUE/Issue/ABOUT hops with (r:Repo)-[:ABOUT]->(t:Topic).
T2 talks_for_issue (Pathfinder, one new issue):
MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person)
RETURN p, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker
LIMIT 2
T3 people_to_meet (relevant speakers you haven't met):
MATCH p = (me:Person {uid: $person})-[:MAINTAINS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})
          -[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person)
WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s)
RETURN p, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk
LIMIT $limit
Fallback if FalkorDB rejects the path filter in WHERE:
MATCH p = (me:Person {uid: $person})-[:MAINTAINS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})
          -[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person)
OPTIONAL MATCH (me)-[:LOGGED]->(e:Encounter)-[:MET]->(s)
WITH p, s, t, k, e
WHERE e IS NULL
RETURN p, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk
LIMIT $limit
T4 memory_recall (Act 3: issue → topic → encounter → person you met):
MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person)
RETURN p, c.uid AS contact_uid, c.name AS contact, e.note AS note,
       e.place AS place, e.met_at AS met_at, t.name AS topic
ORDER BY met_at DESC
LIMIT 3
T5 issues_for_encounter (Act 2: encounter → topic → your open issue → repo → you):
MATCH p = (e:Encounter {uid: $encounter})-[:DISCUSSED]->(t:Topic)<-[:ABOUT]-(i:Issue {gh_state: 'open'})
          <-[:HAS_ISSUE]-(:Repo)<-[:MAINTAINS]-(:Person {kind: 'self'})
MATCH (e)-[:MET]->(c:Person)
RETURN p, i.uid AS issue_uid, i.title AS issue, c.uid AS contact_uid, c.name AS contact, t.name AS topic
T6 talks_by_topic (the track's sample question; $topics comes from classifying the question):
MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person)
WHERE t.slug IN $topics AND ($day IS NULL OR k.day = $day) AND ($after IS NULL OR k.start_time >= $after)
RETURN p, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room,
       s.name AS speaker, t.name AS topic
ORDER BY day, start_time
LIMIT $limit
T7 explain_connection (stretch; check that FalkorDB supports this shortestPath form and that it stays fast):
MATCH (a {uid: $from}), (b {uid: $to})
WITH shortestPath((a)-[*..6]-(b)) AS p
RETURN p
Serialization rules:
• Walk each returned p (falkordb-py Path objects; check attribute names in the installed version) into {nodes: [{uid, label, name, state}], edges: [{type, src_uid, dst_uid}]}. Merge rows and dedupe by uid.
• hops = edge count of the longest returned path.
• run_time_ms = FalkorDB's internal execution time from the result statistics. If it isn't exposed, use wall-clock time and label it "round trip".
{
  "template": "talks_for_my_issues",
  "params": {"person": "person:gh:octocat", "day": "2026-10-28", "after": "13:00", "limit": 5},
  "cypher": "MATCH p = ...",
  "hops": 5,
  "run_time_ms": 0.0,
  "rows": [{"issue": "...", "topic": "...", "talk": "...", "speaker": "..."}],
  "path": {"nodes": [], "edges": []}
}
Three agents and the shared-state protocol
Cartographer, Pathfinder and Concierge never call each other: each claims work by flipping state in FalkorDB, writes its result, and hands off by setting the next state.
Only Cartographer calls GitHub, Pathfinder reasons over the graph with no LLM, and the UI reads the same graph the agents write.
Each claim is one write query, so two loops can never grab the same item:
MATCH (n:Issue {state: 'NEW'})
WITH n LIMIT 1
SET n.state = 'CLASSIFYING', n.claimed_by = $agent, n.claimed_at = timestamp()
RETURN n.uid AS uid, n.title AS title, n.body AS body
Agent
Claims
Does
Hands off as
Cartographer
Person SYNC_REQUESTED
Pulls the top 3 non-fork repos by last push (always including DEMO_REPO for the presenter) and up to 10 open issues each, skipping pull requests. MERGEs Repo, Issue (NEW), MAINTAINS, HAS_ISSUE. A repo with no open issues gets its description classified into Repo ABOUT edges
Person SYNCED
Cartographer
Timer: the DEMO_REPO watcher
Every WATCH_INTERVAL_SEC, lists issues created after Repo.watch_since, MERGEs them as NEW (the cursor starts at the repo's first sync), advances watch_since
Issue NEW
Cartographer
Issue NEW
Batches classify() and MERGEs ABOUT edges
Issue CLASSIFIED
Cartographer
Encounter NEW
One structured LLM call returns {people: [{name, handle, company}], topics: [slug], repo_urls: []} using the same topic enum. MERGEs Person (kind contact; uid from the handle if given, else the name), MET, DISCUSSED, MENTIONED_REPO
Encounter PARSED
Pathfinder
Issue CLASSIFIED
Runs T4 (people you met) and T2 (talks); writes one BRIDGE per hit
Issue LINKED, BRIDGE DISCOVERED
Pathfinder
Encounter PARSED
Runs T5; writes a BRIDGE from each matching issue to the contact
Encounter LINKED, BRIDGE DISCOVERED
Concierge
BRIDGE DISCOVERED
Contact bridge: one LLM call drafts a 2 to 3 sentence follow-up citing the note and the issue. Talk bridge: of any two talks that overlap in time, drops the lower score
BRIDGE ACTION_READY, seen = false
Rules:
• Pathfinder makes no LLM calls. It is pure graph reasoning, and the pitch says so.
• BRIDGE score = topic confidence × (1.0 if hops ≤ 3, else 0.8). The newest encounter wins ties. Don't tune further.
• Every transition writes an AgentEvent node for the UI ticker. Events are a log, not a channel: no agent reads them.
• On startup, each agent returns its own stuck claims to the previous state: SYNCING → SYNC_REQUESTED, CLASSIFYING → NEW, PARSING → NEW, LINKING → CLASSIFIED or PARSED, DRAFTING → DISCOVERED. This is what makes the on-stage restart clean.
• The watcher's cursor lives on the Repo node, so a restarted backend resumes where it stopped.
• Idle loops sleep 0.5 s and claim one item at a time. No queues, and no threads beyond asyncio.to_thread for database calls.
Acceptance checks:
[ ] With the API running, open a real issue in DEMO_REPO on a topic a logged encounter discussed. Within 15 s an ACTION_READY contact BRIDGE exists, and the ticker shows all three agents in order.
[ ] Kill the API mid-pipeline and restart it. The same item finishes with no manual fix.
Backend API contract
The frontend builds against this contract from minute one. With VITE_MOCK=1 it serves fixtures from frontend/src/mocks/, and workstream B swaps each fixture for a real response as soon as that endpoint works.
Method and path
Body or query
Returns
Notes
GET /api/health
none
{falkordb, openai, github, counts}
Pre-demo check; counts per label
POST /api/sync
{handle, kind} with kind self or guest
{person_uid}
Writes Person SYNC_REQUESTED; the agents do the rest
POST /api/encounters
{note, place}
{encounter_uid}
Writes Encounter NEW plus LOGGED from the self person; met_at = now
POST /api/ask
{question, person_uid}
Template envelope plus {answer}
One tool call picks a template; the answer may use only the returned rows
GET /api/graph
scope=overview, or scope=around&uid=...&depth=2
{nodes, edges}
Excludes AgentEvent; at most 400 nodes; includes state
GET /api/events
since=<ms>
AgentEvents, oldest first
UI polls every 1 s
GET /api/alerts
none
[{bridge_uid, issue, contact, note, place, met_at, draft, path, cypher, run_time_ms}]
ACTION_READY contact bridges with seen = false
POST /api/alerts/{bridge_uid}/seen
none
{ok}

GET /api/topics
none
[{slug, name}]
Legend and badges
POST /api/demo/inject-issue
{title, body}
{issue_uid}
Wi-Fi fallback only. Writes Issue NEW into DEMO_REPO directly; everything after that is the normal pipeline
/api/ask behavior:
1. Send the question with the T1, T3, T4 and T6 tool schemas. person_uid defaults to the presenter.
2. For T6, classify the question first to fill topics. Resolve relative times against the days in sessions.json; "after lunch" means 13:00.
3. Run the template, then make a second call that writes 1 to 3 sentences from the returned rows only.
4. If no tool fits, return "I can only answer what the graph can trace" plus the quick-question chips. Never answer from model knowledge.
Frontend spec
One split screen designed for a 1280×720 projector: the left 40% holds inputs, answers and alerts, the right 60% holds the live graph, in a dark theme with text no smaller than 16 px.
Left pane, top to bottom:
1. Header: "PatchWork", the presenter's handle, and three health dots (FalkorDB, OpenAI, GitHub).
2. Sync box: GitHub handle plus "Sync my codebase".
3. Ask box plus 2 or 3 quick-question chips holding the exact demo questions.
4. "Log a hallway conversation": note textarea plus a place field.
5. Answer cards. Each shows the answer, path badges (Me › Repo › Issue › Topic › Talk › Speaker, colored by label, with evidence on hover) and a "Graph proof" drawer: Cypher, params, hop count, FalkorDB time in ms, row count.
6. Agent ticker pinned to the bottom: the last 8 AgentEvents, colored by agent, each showing its state change ("Pathfinder · Issue #89 · CLASSIFIED → LINKING").
7. Alert card that slides in when /api/alerts returns something new: "Someone you met can help with this", the contact, note, place and time, the highlighted path, and the draft with Copy and Dismiss buttons.
Right pane, react-force-graph-2d:
• Color nodes by label and size them by degree. Labels always show on highlighted nodes, otherwise on hover. Add a small legend.
• On a new answer or alert, dim everything else to 15% opacity, widen the highlighted edges, add linkDirectionalParticles (4), and zoom to fit the highlighted nodes.
• Nodes in an in-progress state pulse amber. BRIDGE edges are dashed while DISCOVERED and solid green at ACTION_READY.
• Refresh from /api/graph every 2 s and update node objects in place by uid, so the layout never jumps.
• Fallback hotkeys, active only when no text field has focus: Alt+I injects the Act 3 issue through /api/demo/inject-issue; Alt+R replays the last answer's highlight.
Acceptance checks:
[ ] In mock mode, every component renders from fixtures, and an answer fixture highlights its path.
[ ] At 1280×720 the whole demo fits on one screen with no scrolling except the chat.
Seed data plan
Seed only real data: the official Universe 2026 catalog, a real repo with real open issues, and people who agreed to be in the graph. Universe runs October 28–29, 2026 (GitHub blog).
Sessions (time-box 30 minutes; do it before the event if the rules allow data prep):
1. Open the official session catalog in Chrome with DevTools on the Network tab. Find the request that returns the session list as JSON, save the response as backend/data/raw_catalog.json, and have Claude Code write scripts/import_catalog.py to normalize it.
2. If the catalog needs a login or the JSON is unusable, hand-enter 30 to 40 sessions, starting with the anchors below.
3. Never invent a day, time or room. A session missing them stays out of T1 and T6 results.
{
  "sid": "1777489102123001ThSM",
  "title": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
  "abstract": "First two sentences of the official description",
  "speakers": [{"name": "Nick Taylor", "company": "Pomerium", "github": null}],
  "day": "2026-10-28 or 2026-10-29, from the catalog",
  "start_time": "HH:MM from the catalog",
  "end_time": "HH:MM from the catalog",
  "room": "from the catalog",
  "url": "https://reg.githubuniverse.com/flow/github/universe26/attendee-portal/page/sessioncatalog/session/1777489102123001ThSM"
}
Anchor sessions confirmed on GitHub's blog; seed these even if the import fails (source):
Session
Speakers
Use in the demo
Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers
Nick Taylor (Pomerium)
Storyline topic for all three acts
How GitHub taught Copilot to remember and when to forget
Cooper Nederhood, Alejandro Carderera (GitHub)
Agent-memory topic; pitch tie-in
The 2 a.m. RCA Agent: Architecting AI that investigates, not hallucinates
Achin Gupta (Intuit), Divya Mahajan (Amazon)
Pitch tie-in: deterministic code finds the evidence, the LLM narrates it
What happens behind the scenes when you run npm install
Karen Li, Leo Balter (GitHub)
Dependency-path stretch goal
Disrupting supply chain attacks: A threat framework for GitHub Actions
Steve Glass, Greg Ose (GitHub)
Security topic
Treat your AI context as infrastructure
Christopher Harrison (GitHub)
MCP and skills topics
Build once, run on any agent: a practical guide to agent skills
Shishir Tewari (Procore), Andrea Griffiths (GitHub)
Skills topic
Your benchmark is lying: What evals actually look like
Walker Chabbott (GitHub), Julia Kasper (Microsoft)
Evals topic
Beyond pass or fail: How agents verify AI-generated code
Jeff An (Momentic)
Testing topic
One CLI to replace your entire JavaScript toolchain
Alexander Lichter (VoidZero)
JS tooling topic
Lessons from building tech in a low-connectivity community
Alex Junior Antwi (Braveon AI)
Offline topic
People and repo:
• Presenter and repo. PRESENTER_HANDLE (kind self) plus DEMO_REPO: a real repo with at least 5 real open issues that land on Universe topics. HIVE is a strong candidate if its issues are public. Run T1 on it before committing, and only file new issues you would genuinely work on.
• Teammates join as guests through /api/sync, with their consent.
• The Act 2 contact is a real person at the hackathon (a teammate works) who actually discussed the storyline topic with you that day.
• Speakers' GitHub handles go in only when the catalog or the speaker's own page states them. Otherwise null; never guess.
• Judges. Note their GitHub handles when they're introduced. Around 3 PM, sync each as a guest and confirm T1 or its repo-level variant returns a path. Then restore the baseline so the live sync runs for real; the classification cache keeps it fast.
Storyline texts. Write them once, pre-classify them, and keep them in demo/script.md:
Text
Draft wording (edit to match your real repo)
Must land on
Act 1 issue, already open in DEMO_REPO
"Scope agent permissions per identity"
The MCP authorization topic
Act 2 hallway note
"Met <teammate> at the snack table. They scope agent tokens so agents can open PRs but never merge them."
The MCP authorization topic
Act 3 issue, opened live
"Agents can merge PRs through our MCP server; we need per-identity scopes"
The MCP authorization topic
Build schedule and gates
The build runs 9:00 to 5:00 in six phases, each ending at a gate; code freezes at 5:00 and the project is submitted before 5:30.
Gates G0 to G4 close Phases 0 to 4; Phase 5 hardens the demo, code freezes at 5:00, and the project is submitted by 5:30.
Workstream
Owns
Claude Code session works in
A. Graph and data
Schema, seed, taxonomy, templates, smoke tests
backend/scripts, graph.py, templates.py
B. Agents and API
FastAPI, the three agents, GitHub, OpenAI, events, alerts
backend/app
C. Frontend
Vite app, graph pane, cards, ticker
frontend/
D. Demo and data
Sessions, demo repo, storyline texts, judge checks, backup video, pitch
backend/data, demo/
With three people, split D's tasks between A and C.
Phase 0: Setup (9:00–9:45)
[ ] All: repo created, .env filled, make db running, FalkorDB Browser open at localhost:3000.
[ ] A: graph.py connects and runs RETURN 1; seed.py creates the indexes.
[ ] B: FastAPI skeleton; /api/health reports real FalkorDB, OpenAI and GitHub status.
[ ] C: Vite app with mock mode and the two-pane layout.
[ ] D: sessions.json holds at least 30 real sessions with day, time and room; DEMO_REPO chosen.
• Gate G0: /api/health is all green and make seed loads the sessions.
Phase 1: Graph core (9:45–11:45)
[ ] A: make taxonomy, human review, topics.json frozen by 10:30.
[ ] A: seed talks, speakers, COVERS (through classify()) and SPEAKS_AT.
[ ] A: T1 to T6 pass in FalkorDB Browser, then through run_template() with the full envelope; make smoke green.
[ ] B: github.py (repos, issues, watcher) and llm.py (classify, parse_note, draft_followup) with the cache.
[ ] C: graph pane renders the fixture graph; highlight and zoom-to-fit work; answer card and proof drawer done.
[ ] D: storyline texts written and pre-classified; golden tests written.
[ ] D: one person attends the FalkorDB workshop (10:00–11:00) with this template list and asks about path filters in WHERE, LIMIT $param and shortestPath.
• Gate G1: a scripted sync of the presenter, with no agents yet, makes T1 return at least 3 five-hop paths.
Phase 2: Agents (11:45–1:45)
[ ] B: Cartographer, Pathfinder and Concierge loops with claims, AgentEvents and startup recovery.
[ ] B: /api/sync, /api/encounters, /api/events, /api/alerts, /api/graph.
[ ] A: snapshot.py export and restore; make demo-reset.
[ ] C: ticker, alert card and state coloring against fixtures.
[ ] D: replace hand-written fixtures with real responses from B.
• Gate G2: both agent acceptance checks pass.
Phase 3: Integration (1:45–3:15, pizza included)
[ ] B: /api/ask with tool calling and the no-tool refusal.
[ ] C: mock mode off; everything wired to the real API.
• Gate G3: Acts 1 to 3 run end to end once, unscripted.
Phase 4: Demo polish (3:15–4:30)
[ ] B: /api/demo/inject-issue.
[ ] C: projector theme, quick-question chips, fallback hotkeys.
[ ] D: judge pre-checks, prefilled new-issue link on the phone, storyline cache warm.
• Gate G4: two timed runs in a row, each under 3:00.
Phase 5: Hardening and submission (4:30–5:30)
[ ] Record a backup video of one clean run.
[ ] Failure drills: Wi-Fi off (switch to hotspot), backend killed mid-act (restart), inject-issue path.
[ ] README: one-paragraph pitch, architecture, run steps, video link.
[ ] 5:00 code freeze, make snapshot of the baseline, submit before 5:30.
Stage demo script
Three minutes, three acts, one presenter at the keyboard and one teammate holding the phone. Every step has a fallback that keeps the story intact.
Keep two snapshots: baseline (catalog, taxonomy, teammates) and synced (baseline plus the presenter's repos and issues). make demo-reset restores baseline.
Checklist at 5:25:
[ ] make demo-reset done; /api/health all green; node counts match the baseline.
[ ] Browser zoom set for the projector; screen mirrored; notifications off.
[ ] A terminal running make api is visible next to the browser.
[ ] Phone logged in to GitHub, with the prefilled new-issue link open: https://github.com/<owner>/<repo>/issues/new?title=...&body=....
[ ] Storyline texts in demo/script.md; LLM cache warm from rehearsal.
[ ] Phone hotspot ready.
Time
Do
Judges see
If it fails
0:00–0:20
Hook: "Conference apps die on Monday. Ours wakes up on Monday." Point at the graph
Real Universe sessions, speakers and topics, with counts in the header
Nothing live yet
0:20–1:10
Act 1. Type the presenter's handle and click Sync. Then click the chip "Which talks after lunch on day 1 help with my open issues?"
The ticker shows Cartographer pulling repos and classifying. A 5-hop path lights up. Open Graph proof: the Cypher, 5 hops, FalkorDB's time
Sync stalls: make restore SNAP=synced, then click the chip
1:10–1:50
Act 2. "Earlier today I met <teammate>." Paste the hallway note and submit
The Encounter node appears. Cartographer parses it, Pathfinder bridges it to your open issue with no LLM, Concierge drafts a follow-up
The note's LLM calls are cached; if OpenAI is down the cache still answers
1:50–2:50
Act 3. "Three weeks later." Ctrl-C the backend in the visible terminal and run make api; the chat is empty. The teammate taps Submit on the prefilled issue
Within about 10 s the ticker runs all three agents and the alert slides in: the person you met, the memory path, a ready follow-up
No pickup after 20 s: press Alt+I to inject the same issue locally, and say so out loud
2:50–3:00
Close: "Five-hop traversals, memory that survived a restart, three agents that only talk through the graph, and every answer shows its path."
The memory path stays highlighted

Q&A answers to have ready:
• Why a graph and not SQL? "You could write this as recursive SQL. In Cypher the path is both the query and the answer, so provenance comes free, and FalkorDB returns it in the time on screen."
• Does the LLM make things up? "It never writes Cypher and never creates a connection. It picks a template, and its answer may only use the rows that came back."
• Is this useful after the conference? "That's Act 3. The memory is the product; the conference is how it fills up."
• Privacy? "Only your repos, people who gave you their handle, and notes you wrote yourself."
• What's next? "One graph per team, so a teammate's hallway conversation can answer your issue."
• Bonus: offer to sync a judge's GitHub handle live. Only do it in Q&A, never inside the three minutes.
Hardening and pitch rules
The demo has to survive bad Wi-Fi, a crashed process and a judge's curveball. These rules cost little and remove the usual ways hackathon demos die.
Hardening:
• Route every LLM call (classify, parse_note, draft_followup and the /api/ask answer) through one cache keyed by function, model and input hash. The rehearsed storyline then replays even if OpenAI is unreachable.
• snapshot.py exports every node and edge with its properties to JSON. Restore deletes the graph, then recreates nodes and edges by uid.
• Take the baseline and synced snapshots after the 4:30 rehearsal and commit them.
• After 4:30, don't re-pull the FalkorDB image, update packages or touch the taxonomy.
• Close rehearsal issues on GitHub. make demo-reset moves the watcher cursor so they never come back.
• OpenAI and GitHub are the only network dependencies. If Wi-Fi drops, switch to the hotspot first; then use make restore SNAP=synced for Act 1 and Alt+I for Act 3.
• Keep the backup video open in a browser tab.
Pitch rules:
• Never say "impossible in SQL." Say what the graph makes natural: multi-hop paths with provenance, in milliseconds.
• Show only timings FalkorDB actually returned. No hard-coded numbers anywhere in the UI.
• Say what's real (sessions, repo, issues, people) and what's staged (the three-week jump is a restart, not a clock change).
• Optional line: Pathfinder follows the same split as the 2 a.m. RCA Agent talk, a real Universe session already in your graph: deterministic code finds the evidence and the model narrates it.
• No architecture slides. The graph pane is the slide.
Sources
• 10 technical talks I'm excited about at GitHub Universe 2026, GitHub blog, October 1, 2026: session titles, speakers, catalog links and event dates.
• GitHub Universe 2026 session catalog: days, times and rooms for sessions.json.
