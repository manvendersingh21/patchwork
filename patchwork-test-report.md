# Patchwork adversarial test report

Test date: 2026-10-24 (America/Los_Angeles). Executed against the connected Patchwork MCP plugin.

143 tool calls across all 11 tools: **107 pass, 35 fail, 1 inconclusive**. These are case verdicts, not 35 distinct bugs. Five main findings follow. There were **19 successful write calls** (7 learnings, 7 issues, 5 encounters), all using mcp-test- identities. No graph cleanup or snapshot restore was attempted.

## Scope and limits

The two supplied screenshots were used as the test contract. The connector's exposed tool inventory and typed declarations were inspected; raw MCP tools/list, HTTP initialization, SSE parsing, and the user's local Mac binary were not accessed. A fresh baseline restore was not performed because graph mutation outside the MCP tools was forbidden. The starting topic count was 22 and test-alpha learning recall was empty.

All graph writes used official Patchwork tools, with mcp-test- logger handles and met-person names/handles. The presenter was queried read-only. However, background agents created cross-owner CONTACT bridges on presenter issues: this is a finding, not a requested direct write. No messages were sent. No .env, credentials, or secret files were read or printed.

Every result fed a narrower query, round-trip, boundary case, or final invariant check. Follow-up relations are recorded in the table. The final self-generated rounds (#128–135 and #136–143) produced no new defect class; repeated known failures remain failures. This is the stopping condition, not a claim that all possible inputs were exhausted.

## Five worst findings

### 1. High — Encounter memory crosses user boundaries

Alpha receives Beta's private encounter note and place. Beta receives Alpha's encounter. The presenter also sees synthetic test encounters. The returned T4 Cypher joins Issue→Topic←Encounter→Person without requiring the encounter logger to own/track the issue. This proves logical ownership leakage in this shared graph; transport-level multi-account authentication was not tested.

Evidence: calls #55, #56, #61, #63, #113.

Minimal reproducer (reuse the created issue UID where shown; calls are official tool names and JSON arguments):

```json
[
  {
    "tool": "log_issue",
    "args": {
      "person": "mcp-test-beta",
      "title": "mcp-test: beta-only auth issue",
      "body": "mcp-test: beta should only see its own issues",
      "topics": [
        "mcp-auth"
      ]
    }
  },
  {
    "tool": "log_encounter",
    "args": {
      "person": "person:gh:mcp-test-beta",
      "met_name": "mcp-test-beta-private",
      "met_handle": "mcp-test-beta-private",
      "note": "mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024",
      "place": "mcp-test-beta-place",
      "topics": [
        "mcp-auth"
      ]
    }
  },
  {
    "tool": "memory_recall",
    "args": {
      "issue": "issue:notes/mcp-test-alpha#7c136aa7"
    }
  }
]
```

Expected: Alpha's issue must exclude Beta's encounter.

Actual: The result contains BETA_ONLY_PRIVATE_NOTE_20261024 and mcp-test-beta-place.

Suggested fix: Constrain the issue owner and Encounter LOGGED relationship to the same Person in T4; enforce ownership consistently for recall.

### 2. High — CONTACT bridge creation crosses user boundaries

A test-only encounter caused ACTION_READY CONTACT bridges on the presenter's real issues. No call wrote as the presenter; the graph agents created the cross-owner links. The unscoped check_alerts API also exposes these cross-owner records.

Evidence: calls #29, #31, #36, #56, #112, #143.

Minimal reproducer (reuse the created issue UID where shown; calls are official tool names and JSON arguments):

```json
[
  {
    "tool": "log_encounter",
    "args": {
      "person": "person:gh:mcp-test-alpha",
      "met_handle": "mcp-test-contact",
      "met_name": "mcp-test-contact",
      "note": "mcp-test: shared MCP auth experience.",
      "place": "mcp-test-hallway",
      "topics": [
        "mcp-auth"
      ]
    }
  },
  {
    "tool": "check_alerts",
    "args": {}
  }
]
```

Expected: Only issues belonging to the encounter logger should gain personal follow-up bridges.

Actual: bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd points mcp-test-contact at issue:manvendersingh21/deepcore-n1#1. Four observed bridges target presenter issues (#1 and #4).

Suggested fix: Scope background bridge matching by the logger's maintained/tracked repos, and scope alert reads by the authenticated/explicit person.

### 3. Medium — Learning topic filter does not filter records

A topic filter changes the returned tags but leaves all learning records present. This gives false summaries. It also drops other valid tags from matching multi-topic lessons.

Evidence: calls #101, #108, #109, #111, #122.

Minimal reproducer (reuse the created issue UID where shown; calls are official tool names and JSON arguments):

```json
[
  {
    "tool": "recall_learnings",
    "args": {
      "person": "mcp-test-alpha",
      "topic": "agent-memory"
    }
  },
  {
    "tool": "recall_learnings",
    "args": {
      "person": "mcp-test-alpha",
      "topic": "oauth"
    }
  }
]
```

Expected: agent-memory returns zero records; oauth returns exactly the two OAuth lessons.

Actual: Both calls return all five alpha records. For agent-memory all five have topics: []; for oauth three nonmatching records have topics: []. The overlapping mcp-auth/oauth lesson loses its mcp-auth tag in the filtered response.

Suggested fix: Apply topic membership as a required record selection condition before aggregating complete topic tags; reject unknown topic slugs.

### 4. Medium — Unsupported dates and malformed times silently remove filters

The API accepts a restrictive or malformed filter and silently broadens the query. Known conference dates 2026-10-28 and 2026-10-29 work; other YYYY-MM-DD dates such as 2030-01-01 are silently echoed as an empty day rather than yielding no rows/error. Invalid times behave similarly.

Evidence: calls #16, #17, #21, #26, #32, #45, #125, #126.

Minimal reproducer (reuse the created issue UID where shown; calls are official tool names and JSON arguments):

```json
[
  {
    "tool": "talks_by_topic",
    "args": {
      "topics": "mcp-auth",
      "day": "2030-01-01"
    }
  },
  {
    "tool": "talks_for_my_issues",
    "args": {
      "person": "person:gh:manvendersingh21",
      "after": "99:99"
    }
  }
]
```

Expected: A valid date outside the schedule yields no rows; malformed times produce a validation error.

Actual: 2030-01-01 becomes params.day="" and returns the 2026-10-29 talk. 99:99 becomes params.after="" and returns the baseline rows.

Suggested fix: Validate actual date/time syntax; preserve well-formed dates and reject unsupported values explicitly instead of clearing supplied filters.

### 5. Medium — Read validation violates the closed-topic and bad-UID contract

Unknown/noncanonical/empty topic values are successful empty queries (or partial successes when mixed with a valid slug). Missing/malformed/empty person and issue UID values also produce empty success when present as strings. Required-key omissions do fail at the connector/schema boundary. Writes correctly reject unknown/empty topics.

Evidence: calls #8, #10, #11, #12, #13, #14, #15, #20, #23, #25, #35, #109, #127, #135, #137, #140.

Minimal reproducer (reuse the created issue UID where shown; calls are official tool names and JSON arguments):

```json
[
  {
    "tool": "talks_by_topic",
    "args": {
      "topics": "mcp-test-unknown"
    }
  },
  {
    "tool": "talks_by_topic",
    "args": {
      "topics": "mcp-auth,mcp-test-unknown"
    }
  },
  {
    "tool": "talks_for_issue",
    "args": {
      "issue": "bad"
    }
  },
  {
    "tool": "people_to_meet",
    "args": {
      "person": ""
    }
  }
]
```

Expected: INVALID_ARGUMENT for unknown slugs and malformed/missing entities, as requested by the supplied test contract.

Actual: isError=false, empty rows for unknown/bad inputs; mixed valid/unknown topics return the valid talk. Unknown topics are not created.

Suggested fix: Share a canonical topic validator across all tools and validate UID format and entity existence before executing templates.

## Confirmed working behavior

- Exactly 22 unique topics remained unchanged before and after the test; every topic has at least one talk. Consequently the requested valid-topic/no-talk bridge scenario cannot be constructed with this dataset. A time-filtered no-result query passed (#107).
- Presenter T1 returned nonempty five-hop rows; baseline query runtime was 2.059 ms, within the supplied ~0.5–5 ms target. Rows were preserved after all inert Cypher payloads.
- All observed template responses contained template, hops, run_time_ms, rows and path. All path edge endpoints referenced returned nodes. Empty envelopes use hops=0. Populated T1/T3 use 5; T2/T4 use 3; T6 uses 2.
- All three write tools reject unknown/empty topics and empty required text, with clean INVALID_ARGUMENT errors. Failed validation left no learning records for mcp-test-validation and did not alter the topic set; lack of an all-node API prevents proving absence of every possible partial side effect.
- Learning recall preserves ownership, newest-first order, exact Unicode/composing marks, quotes, newlines, omitted source default, and all text/topic fields when no topic filter is supplied. Two identical Beta writes produced two distinct UIDs and two read-back records: append-only behavior, not idempotent deduplication.
- Unicode/Cypher-looking title, learning, encounter note and place were returned exactly. The 2000-character learning and encounter note were returned exactly. Single-word cases worked. The 2000-character issue body was accepted, but no exposed tool reads bodies back; byte-for-byte issue-body verification is **inconclusive**.
- Half-open time filtering works for canonical times: after=10:30 includes, before=10:30 excludes, before=10:31 includes. A zero-width or inverted interval returns empty rows. Known conference-day filters work.
- check_alerts returned CONTACT-shaped records only; no talk-only bridges appeared in any observed alert response. Current schemas return hop values in alerts as strings; template hops are numeric. This is an observation, not a ranked contract failure.

One Unicode learning call (#49) failed before normal execution with “catalog changed after ... was prepared.” The same arguments succeeded at #57 after refreshing availability. Classified as a connector interruption, not a Patchwork server bug.

Automated result comparisons:

| Assertion | Result |
| --- | --- |
| All alpha learning text/source/topics byte-for-byte | pass |
| Alpha newest first | pass |
| Presenter baseline rows preserved | pass |
| Topic set preserved | pass |
| All template envelopes and edge endpoints valid | pass |

## State-machine timing

All three sampled issues were CLASSIFIED in log_issue responses and LINKED with nonempty three-hop talk paths on the first read. The table measures observation bounds, **not exact agent processing time**. Connector round-trip latency was longer than the requested 2-second polling interval, so the next poll was issued immediately after each write response and no artificial wait was added. No sample required a second poll.

| Write → poll | Issue UID | Write response → first read result (ms) | Write start → first read result (ms) | State | Rows |
| --- | --- | --- | --- | --- | --- |
| #29 → #30 | issue:notes/mcp-test-alpha#7c136aa7 | 3062 | 6659 | LINKED | 1 |
| #118 → #119 | issue:notes/mcp-test-timing-1#c30e2013 | 2968 | 6111 | LINKED | 1 |
| #120 → #121 | issue:notes/mcp-test-timing-2#bc4fd6eb | 2719 | 5730 | LINKED | 1 |

Response-to-first-read observation bounds: min 2719 ms, median 2968 ms, max 3062 ms, n=3. All observed readiness bounds are below ~10 seconds. This does not isolate the timing of persisted TALK bridge creation: T2 follows ABOUT/COVERS directly; LINKED state provides additional lifecycle evidence, and no raw graph bridge enumeration is exposed.

Template query runtime across 85 responses: min 0.132 ms, median 0.467 ms, max 2.059 ms. Per-call wall times in the evidence include connector queuing when reads were batched and should not be interpreted as database runtimes.

## Tool coverage

| Tool | Calls |
| --- | --- |
| check_alerts | 4 |
| list_topics | 3 |
| log_encounter | 8 |
| log_issue | 10 |
| log_learning | 11 |
| memory_recall | 12 |
| people_to_meet | 6 |
| recall_learnings | 17 |
| talks_by_topic | 49 |
| talks_for_issue | 8 |
| talks_for_my_issues | 15 |

## Complete call table

Long inputs are abbreviated here. Exact inputs, complete responses and timings are in the JSON evidence appendix. Pass/fail uses the screenshot's contract, which deliberately requires bad IDs and unknown topics to fail cleanly. Cases that reproduce an already known bug remain fail even when the observed defect is the follow-up's anticipated outcome.

| # | tool | input summary | expected | actual | verdict | follow-up spawned |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | list_topics | {} | Exactly 22 unique topic slugs | 22 topics | pass | Query each slug (same-tool follow-up #103) |
| 2 | people_to_meet | {"person":"person:gh:manvendersingh21"} | T3 envelope, 5-hop relevant speakers | T3; 5 rows; hops=5; query=1.485ms; params={"limit":5,"person":"person:gh:manvendersingh21"} | pass | Compare synthetic person speakers (same-tool follow-up #11) |
| 3 | check_alerts | {} | CONTACT alerts baseline | 0 CONTACT alerts | pass | Create test encounter then recheck (same-tool follow-up #36) |
| 4 | talks_for_my_issues | {"person":"person:gh:manvendersingh21"} | Nonempty T1 rows, 5 hops, graph runtime ~0.5–5ms | T1; 5 rows; hops=5; query=2.059ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | pass | Test day/time and unknown identities (same-tool follow-up #8) |
| 5 | recall_learnings | {"person":"mcp-test-alpha"} | No preexisting test learnings | 0 learnings; topics=[] | pass | Write then recall (same-tool follow-up #13) |
| 6 | talks_by_topic | {"topics":"mcp-auth"} | T5 envelope with known talks | T6; 1 rows; hops=2; query=1.203ms; params={"after":"","before":"","day":"","topics":"mcp-auth"} | pass | Use observed date/time bounds (same-tool follow-up #14) |
| 7 | talks_for_issue | {"issue":"issue:manvendersingh21/deepcore-n1#1"} | 3-hop envelope and rows | T2; 2 rows; hops=3; query=1.056ms; params={"issue":"issue:manvendersingh21/deepcore-n1#1"} | pass | Test missing issue (same-tool follow-up #10) |
| 8 | talks_for_my_issues | {"person":"person:gh:mcp-test-missing"} | Clean error for nonexistent person per brief | T1; 0 rows; hops=0; query=0.917ms; params={"after":"","before":"","day":"","person":"person:gh:mcp-test-missing"} | fail | Compare empty person (same-tool follow-up #16) |
| 9 | memory_recall | {"issue":"issue:manvendersingh21/deepcore-n1#1"} | T4 envelope | T4; 0 rows; hops=0; query=0.935ms; params={"issue":"issue:manvendersingh21/deepcore-n1#1"} | pass | Create matching test encounter (same-tool follow-up #12) |
| 10 | talks_for_issue | {"issue":"issue:mcp-test-missing/notes#999999"} | Clean error per brief | T2; 0 rows; hops=0; query=0.132ms; params={"issue":"issue:mcp-test-missing/notes#999999"} | fail | Compare malformed uid (same-tool follow-up #30) |
| 11 | people_to_meet | {"person":""} | Clean validation error | T3; 0 rows; hops=0; query=0.203ms; params={"limit":5,"person":""} | fail | Compare nonexistent person (same-tool follow-up #35) |
| 12 | memory_recall | {"issue":"bad"} | Clean error per brief | T4; 0 rows; hops=0; query=0.170ms; params={"issue":"bad"} | fail | Compare empty uid (same-tool follow-up #25) |
| 13 | recall_learnings | {"person":"mcp-test-alpha","topic":"mcp-test-unknown"} | Unknown slug error | 0 learnings; topics=[] | fail | Compare valid topic (same-tool follow-up #27) |
| 14 | talks_by_topic | {"topics":"mcp-test-unknown"} | Unknown slug error | T6; 0 rows; hops=0; query=0.340ms; params={"after":"","before":"","day":"","topics":"mcp-test-unknown"} | fail | Mixed valid and unknown slug (same-tool follow-up #15) |
| 15 | talks_by_topic | {"topics":""} | Empty topics error | T6; 0 rows; hops=0; query=0.339ms; params={"after":"","before":"","day":"","topics":""} | fail | Whitespace topics (same-tool follow-up #18) |
| 16 | talks_for_my_issues | {"person":"person:gh:manvendersingh21","day":"2026-13-99"} | Invalid date error | T1; 5 rows; hops=5; query=1.271ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | fail | Valid no-result date (same-tool follow-up #17) |
| 17 | talks_for_my_issues | {"person":"person:gh:manvendersingh21","after":"99:99"} | Invalid time error | T1; 5 rows; hops=5; query=1.314ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | fail | Compare valid exact bounds (same-tool follow-up #24) |
| 18 | talks_by_topic | {"topics":"mcp-auth","day":"2026-10-29"} | One MCP Auth talk | T6; 1 rows; hops=2; query=0.441ms; params={"after":"","before":"","day":"2026-10-29","topics":"mcp-auth"} | pass | Exclude via other date (same-tool follow-up #19) |
| 19 | talks_by_topic | {"topics":"mcp-auth","after":"23:59","before":"00:01"} | Zero rows or explicit inverted range error | T6; 0 rows; hops=0; query=0.217ms; params={"after":"23:59","before":"00:01","day":"","topics":"mcp-auth"} | pass | Exact interval (same-tool follow-up #20) |
| 20 | talks_by_topic | {"topics":"mcp-auth,mcp-test-unknown"} | Unknown slug error | T6; 1 rows; hops=2; query=0.447ms; params={"after":"","before":"","day":"","topics":"mcp-auth,mcp-test-unknown"} | fail | Check topics still 22 (same-tool follow-up #21) |
| 21 | talks_by_topic | {"topics":"mcp-auth","day":"2030-01-01"} | Zero rows | T6; 1 rows; hops=2; query=0.459ms; params={"after":"","before":"","day":"","topics":"mcp-auth"} | fail | Inspect echoed params (same-tool follow-up #22) |
| 22 | talks_by_topic | {"topics":"   "} | Validation error | T6; 0 rows; hops=0; query=0.334ms; params={"after":"","before":"","day":"","topics":"   "} | fail | Uppercase slug (same-tool follow-up #23) |
| 23 | talks_by_topic | {"topics":"MCP-AUTH"} | Reject noncanonical slug | T6; 0 rows; hops=0; query=0.321ms; params={"after":"","before":"","day":"","topics":"MCP-AUTH"} | fail | Known canonical topic (same-tool follow-up #32) |
| 24 | talks_for_my_issues | {"person":"person:gh:manvendersingh21","day":"","after":"","before":""} | Same baseline | T1; 5 rows; hops=5; query=1.370ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | pass | Valid date excludes (same-tool follow-up #26) |
| 25 | memory_recall | {"issue":""} | Validation error | T4; 0 rows; hops=0; query=0.137ms; params={"issue":""} | fail | Missing issue (same-tool follow-up #38) |
| 26 | talks_for_my_issues | {"person":"person:gh:manvendersingh21","day":"2030-01-01"} | Zero rows | T1; 5 rows; hops=5; query=1.316ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | fail | Check params (same-tool follow-up #34) |
| 27 | recall_learnings | {"person":"mcp-test-alpha","topic":"mcp-auth"} | Empty before writes | 0 learnings; topics=[] | pass | Learning round-trip (same-tool follow-up #33) |
| 28 | log_learning | {"person":"mcp-test-alpha","learned":"mcp-test: MCP auth scopes require explicit checks.","source":"mcp-test-roundtrip","topics":["mcp-auth"]} | New learning with related MCP Auth talk | {"learning_uid":"learning:9d307e75-e63d-42d8-985b-f2471f6a6303","person_uid":"person:gh:mcp-test-alpha","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiv… | pass | Recall byte-for-byte and second overlap (same-tool follow-up #46) |
| 29 | log_issue | {"person":"mcp-test-alpha","title":"mcp-test: MCP auth bridge timing","body":"mcp-test: investigate MCP authorization scopes.","topics":["mcp-auth"]} | CLASSIFIED issue then bridged talks within ~10s | {"issue_uid":"issue:notes/mcp-test-alpha#7c136aa7","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); … | pass | Poll issue immediately (same-tool follow-up #53) |
| 30 | talks_for_issue | {"issue":"issue:notes/mcp-test-alpha#7c136aa7"} | 3-hop talks | T2; 1 rows; hops=3; query=0.392ms; params={"issue":"issue:notes/mcp-test-alpha#7c136aa7"} | pass | Delayed poll and alerts (same-tool follow-up #99) |
| 31 | log_encounter | {"person":"person:gh:mcp-test-alpha","met_handle":"mcp-test-contact","met_name":"mcp-test-contact","note":"mcp-test: shared MCP auth experience.","place":"mcp-test-hal… | Recorded encounter under test identities | {"contact_uid":"person:gh:mcp-test-contact","encounter_uid":"enc:308e4ce9-9740-456d-85b4-dc315561230f","state":"PARSED"} | pass | Recall via issue and CONTACT alert (same-tool follow-up #54) |
| 32 | talks_by_topic | {"topics":"mcp-auth","day":"2030-01-01","after":"10:30","before":"10:31"} | No rows due date | T6; 1 rows; hops=2; query=1.135ms; params={"after":"10:30","before":"10:31","day":"","topics":"mcp-auth"} | pass | Check exact time boundaries (same-tool follow-up #41) |
| 33 | recall_learnings | {"person":"mcp-test-alpha"} | Exact first learning | 1 learnings; topics=[["mcp-auth"]] | pass | Second overlapping learning (same-tool follow-up #62) |
| 34 | talks_for_my_issues | {"person":"person:gh:manvendersingh21","after":"23:59"} | Zero rows | T1; 0 rows; hops=0; query=1.004ms; params={"after":"23:59","before":"","day":"","person":"person:gh:manvendersingh21"} | pass | Inspect echoed after (same-tool follow-up #37) |
| 35 | people_to_meet | {"person":"person:gh:mcp-test-missing"} | Clean error per brief | T3; 0 rows; hops=0; query=0.994ms; params={"limit":5,"person":"person:gh:mcp-test-missing"} | fail | Test alpha isolation (same-tool follow-up #39) |
| 36 | check_alerts | {} | CONTACT bridge for test issue and contact only | 1 CONTACT alerts | fail | Verify no TALK alerts (same-tool follow-up #112) |
| 37 | talks_for_my_issues | {"person":"person:gh:mcp-test-alpha"} | Only test issue, five hops | T1; 1 rows; hops=5; query=0.914ms; params={"after":"","before":"","day":"","person":"person:gh:mcp-test-alpha"} | pass | Beta has no issues (same-tool follow-up #40) |
| 38 | memory_recall | {"issue":"issue:notes/mcp-test-alpha#7c136aa7"} | Own contact plus exact note and place | T4; 1 rows; hops=3; query=1.046ms; params={"issue":"issue:notes/mcp-test-alpha#7c136aa7"} | pass | Repeat no duplicate contacts (same-tool follow-up #61) |
| 39 | people_to_meet | {"person":"person:gh:mcp-test-alpha"} | Nick Taylor relevant, not synthetic contact | T3; 1 rows; hops=5; query=0.647ms; params={"limit":5,"person":"person:gh:mcp-test-alpha"} | pass | Compare beta (same-tool follow-up #133) |
| 40 | talks_for_my_issues | {"person":"person:gh:mcp-test-beta"} | Zero rows and no presenter leak | T1; 0 rows; hops=0; query=0.879ms; params={"after":"","before":"","day":"","person":"person:gh:mcp-test-beta"} | pass | Create beta issue (same-tool follow-up #96) |
| 41 | talks_by_topic | {"topics":"mcp-auth","after":"10:30"} | Includes 10:30 talk | T6; 1 rows; hops=2; query=0.471ms; params={"after":"10:30","before":"","day":"","topics":"mcp-auth"} | pass | After 10:31 excludes (same-tool follow-up #42) |
| 42 | talks_by_topic | {"topics":"mcp-auth","before":"10:30"} | Excludes 10:30 talk | T6; 0 rows; hops=0; query=0.340ms; params={"after":"","before":"10:30","day":"","topics":"mcp-auth"} | pass | Before 10:31 includes (same-tool follow-up #43) |
| 43 | talks_by_topic | {"topics":"mcp-auth","after":"10:31"} | Zero rows | T6; 0 rows; hops=0; query=0.304ms; params={"after":"10:31","before":"","day":"","topics":"mcp-auth"} | pass | Invalid after validation (same-tool follow-up #44) |
| 44 | talks_by_topic | {"topics":"mcp-auth","before":"10:31"} | One row | T6; 1 rows; hops=2; query=0.446ms; params={"after":"","before":"10:31","day":"","topics":"mcp-auth"} | pass | Compare omitted bounds (same-tool follow-up #45) |
| 45 | talks_by_topic | {"topics":"mcp-auth","day":"2026-13-99","after":"99:99"} | Validation error | T6; 1 rows; hops=2; query=0.529ms; params={"after":"","before":"","day":"","topics":"mcp-auth"} | fail | Inspect params handling (same-tool follow-up #65) |
| 46 | log_learning | {"person":"mcp-test-alpha","learned":"mcp-test: second overlapping lesson.","source":"mcp-test-overlap","topics":["mcp-auth","oauth"]} | Distinct newer uid; related talks match known topic | {"learning_uid":"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9","person_uid":"person:gh:mcp-test-alpha","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiv… | pass | Recall ordering (same-tool follow-up #47) |
| 47 | log_learning | {"person":"mcp-test-beta","learned":"mcp-test: identical lesson.","source":"mcp-test-duplicate","topics":["mcp-auth"]} | Beta learning independent of alpha | {"learning_uid":"learning:05362c1e-7200-494e-9185-f8eabf1bcd25","person_uid":"person:gh:mcp-test-beta","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiva… | pass | Repeat identical write (same-tool follow-up #48) |
| 48 | log_learning | {"person":"mcp-test-beta","learned":"mcp-test: identical lesson.","source":"mcp-test-duplicate","topics":["mcp-auth"]} | Document duplicate behavior | {"learning_uid":"learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2","person_uid":"person:gh:mcp-test-beta","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiva… | pass | Recall beta two uids (same-tool follow-up #49) |
| 49 | log_learning | {"person":"mcp-test-alpha","learned":"mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' 'MATCH (n) DETACH DELETE n'","source":"mcp-test: 雪 ❄️ ' \" \\ newline\nend","topi… | Store exact inert payload | tool call error: tool call rejected because the catalog changed after 'codex_apps/patchwork.log_learning' was prepared | inconclusive | Recall byte-for-byte; baseline intact (same-tool follow-up #50) |
| 50 | log_learning | {"person":"mcp-test-alpha","learned":"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx… | Accept 2000 characters intact | {"learning_uid":"learning:90d70c0a-1147-414c-b29e-c659500bcb48","person_uid":"person:gh:mcp-test-alpha","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiv… | pass | Recall length (same-tool follow-up #51) |
| 51 | log_learning | {"person":"mcp-test-alpha","learned":"mcp-test-singleword","topics":["mcp-auth"]} | Accept single word with omitted source | {"learning_uid":"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9","person_uid":"person:gh:mcp-test-alpha","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiv… | pass | Recall source default (same-tool follow-up #52) |
| 52 | log_learning | {"person":"mcp-test-validation","learned":"mcp-test: invalid topic must reject","topics":["mcp-test-unknown"]} | Reject before mutation | {"error":"McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_… | pass | Check 22 topics and no saved learning (same-tool follow-up #57) |
| 53 | log_issue | {"person":"mcp-test-validation","title":"mcp-test: unknown slug","body":"mcp-test: reject unknown topic","topics":["mcp-test-unknown"]} | Reject before mutation | {"error":"McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_… | pass | Check topic count (same-tool follow-up #55) |
| 54 | log_encounter | {"person":"person:gh:mcp-test-validation","met_handle":"mcp-test-invalid-contact","met_name":"mcp-test-invalid-contact","note":"mcp-test: reject unknown topic","topics… | Reject before mutation | {"error":"McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_… | pass | Check graph baseline (same-tool follow-up #56) |
| 55 | log_issue | {"person":"mcp-test-beta","title":"mcp-test: beta-only auth issue","body":"mcp-test: beta should only see its own issues","topics":["mcp-auth"]} | New beta issue only | {"issue_uid":"issue:notes/mcp-test-beta#f6cc6fcd","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); w… | pass | Alpha memory must not show beta contact (same-tool follow-up #59) |
| 56 | log_encounter | {"person":"person:gh:mcp-test-beta","met_name":"mcp-test-beta-private","met_handle":"mcp-test-beta-private","note":"mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024","place":… | Beta private encounter | {"contact_uid":"person:gh:mcp-test-beta-private","encounter_uid":"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a","state":"PARSED"} | pass | Alpha memory must exclude beta note (same-tool follow-up #60) |
| 57 | log_learning | {"person":"mcp-test-alpha","learned":"mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' 'MATCH (n) DETACH DELETE n'","source":"mcp-test: 雪 ❄️ ' \" \\ newline\nend","topi… | Inert exact string | {"learning_uid":"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f","person_uid":"person:gh:mcp-test-alpha","related_talks":[{"day":"2026-10-29","room":"Demo Stage (Festiv… | pass | Recall equality and presenter health (same-tool follow-up #58) |
| 58 | log_learning | {"person":"mcp-test-validation","learned":"mcp-test-empty-topic","topics":[]} | Clean error before mutation | {"error":"McpServerError: topics must be non-empty slugs from list_topics; unknown: []","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_rpc_error","code":-3… | pass | Validation recall empty (same-tool follow-up #93) |
| 59 | log_issue | {"person":"mcp-test-validation","title":"mcp-test-empty-topic","body":"mcp-test-empty-topic","topics":[]} | Clean error before mutation | {"error":"McpServerError: topics must be non-empty slugs from list_topics; unknown: []","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_rpc_error","code":-3… | pass | Validation recall empty (same-tool follow-up #87) |
| 60 | log_encounter | {"person":"person:gh:mcp-test-validation","met_name":"mcp-test-validation","note":"mcp-test-empty-topic","topics":[]} | Clean error before mutation | {"error":"McpServerError: topics must be non-empty slugs from list_topics; unknown: []","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_rpc_error","code":-3… | pass | Topic count stays 22 (same-tool follow-up #88) |
| 61 | memory_recall | {"issue":"issue:notes/mcp-test-alpha#7c136aa7"} | Must exclude BETA_ONLY private note | T4; 2 rows; hops=3; query=0.478ms; params={"issue":"issue:notes/mcp-test-alpha#7c136aa7"} | fail | Reverse beta recall (same-tool follow-up #63) |
| 62 | recall_learnings | {"person":"mcp-test-beta"} | Only two beta duplicate records, newest first | 2 learnings; topics=[["mcp-auth"],["mcp-auth"]] | pass | Alpha excludes beta uids (same-tool follow-up #64) |
| 63 | memory_recall | {"issue":"issue:notes/mcp-test-beta#f6cc6fcd"} | Must exclude alpha encounter | T4; 2 rows; hops=3; query=0.445ms; params={"issue":"issue:notes/mcp-test-beta#f6cc6fcd"} | fail | Compare path LOGGED constraints (same-tool follow-up #97) |
| 64 | recall_learnings | {"person":"mcp-test-alpha"} | All alpha uids exact and newest first | 5 learnings; topics=[["mcp-auth"],["mcp-auth"],["oauth"],["mcp-auth","oauth"],["mcp-auth"]] | pass | Filter by oauth (same-tool follow-up #101) |
| 65 | talks_by_topic | {"topics":"agentic-security"} | Valid topic; record whether talks exist | T6; 2 rows; hops=2; query=1.193ms; params={"after":"","before":"","day":"","topics":"agentic-security"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #66) |
| 66 | talks_by_topic | {"topics":"ai-context"} | Valid topic; record whether talks exist | T6; 4 rows; hops=2; query=0.430ms; params={"after":"","before":"","day":"","topics":"ai-context"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #67) |
| 67 | talks_by_topic | {"topics":"agent-memory"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.499ms; params={"after":"","before":"","day":"","topics":"agent-memory"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #68) |
| 68 | talks_by_topic | {"topics":"ai-assisted-testing"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.463ms; params={"after":"","before":"","day":"","topics":"ai-assisted-testing"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #69) |
| 69 | talks_by_topic | {"topics":"agentic-workflows"} | Valid topic; record whether talks exist | T6; 3 rows; hops=2; query=0.468ms; params={"after":"","before":"","day":"","topics":"agentic-workflows"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #70) |
| 70 | talks_by_topic | {"topics":"ai-operations"} | Valid topic; record whether talks exist | T6; 2 rows; hops=2; query=0.416ms; params={"after":"","before":"","day":"","topics":"ai-operations"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #71) |
| 71 | talks_by_topic | {"topics":"cicd"} | Valid topic; record whether talks exist | T6; 3 rows; hops=2; query=0.534ms; params={"after":"","before":"","day":"","topics":"cicd"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #72) |
| 72 | talks_by_topic | {"topics":"community-governance"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.469ms; params={"after":"","before":"","day":"","topics":"community-governance"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #73) |
| 73 | talks_by_topic | {"topics":"copilot"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.462ms; params={"after":"","before":"","day":"","topics":"copilot"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #74) |
| 74 | talks_by_topic | {"topics":"dependencies"} | Valid topic; record whether talks exist | T6; 2 rows; hops=2; query=0.454ms; params={"after":"","before":"","day":"","topics":"dependencies"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #75) |
| 75 | talks_by_topic | {"topics":"deterministic-verification"} | Valid topic; record whether talks exist | T6; 3 rows; hops=2; query=0.495ms; params={"after":"","before":"","day":"","topics":"deterministic-verification"} | pass | Compare rarest topic / learning related talks (same-tool follow-up #76) |
| 76 | talks_by_topic | {"topics":"enterprise-ai-adoption"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.510ms; params={"after":"","before":"","day":"","topics":"enterprise-ai-adoption"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #77) |
| 77 | talks_by_topic | {"topics":"frontier-models"} | Valid topic; record whether talks exist | T6; 3 rows; hops=2; query=0.409ms; params={"after":"","before":"","day":"","topics":"frontier-models"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #78) |
| 78 | talks_by_topic | {"topics":"github-actions"} | Valid topic; record whether talks exist | T6; 2 rows; hops=2; query=0.411ms; params={"after":"","before":"","day":"","topics":"github-actions"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #79) |
| 79 | talks_by_topic | {"topics":"developer-experience"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.483ms; params={"after":"","before":"","day":"","topics":"developer-experience"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #80) |
| 80 | talks_by_topic | {"topics":"oauth"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.581ms; params={"after":"","before":"","day":"","topics":"oauth"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #81) |
| 81 | talks_by_topic | {"topics":"mcp-auth"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.457ms; params={"after":"","before":"","day":"","topics":"mcp-auth"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #82) |
| 82 | talks_by_topic | {"topics":"open-source"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.388ms; params={"after":"","before":"","day":"","topics":"open-source"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #83) |
| 83 | talks_by_topic | {"topics":"package-management"} | Valid topic; record whether talks exist | T6; 2 rows; hops=2; query=0.406ms; params={"after":"","before":"","day":"","topics":"package-management"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #84) |
| 84 | talks_by_topic | {"topics":"platform-engineering"} | Valid topic; record whether talks exist | T6; 1 rows; hops=2; query=0.414ms; params={"after":"","before":"","day":"","topics":"platform-engineering"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #85) |
| 85 | talks_by_topic | {"topics":"rca-agent"} | Valid topic; record whether talks exist | T6; 2 rows; hops=2; query=0.472ms; params={"after":"","before":"","day":"","topics":"rca-agent"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #86) |
| 86 | talks_by_topic | {"topics":"supply-chain-security"} | Valid topic; record whether talks exist | T6; 4 rows; hops=2; query=0.431ms; params={"after":"","before":"","day":"","topics":"supply-chain-security"} | pass | Compare rarest topic / related talk sets (same-tool follow-up #105) |
| 87 | log_issue | {"person":"mcp-test-payload","title":"mcp-test: नमस्ते 🧵 café é \"quote\" 'single' 'MATCH (n) DETACH DELETE n'","body":"mcp-test: नमस्ते 🧵 café é \"quote\" 'single… | CLASSIFIED; inert title | {"issue_uid":"issue:notes/mcp-test-payload#2c4df3e7","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4)… | pass | Read exact title via own issue query; body not exposed (same-tool follow-up #89) |
| 88 | log_encounter | {"person":"person:gh:mcp-test-payload","met_handle":"mcp-test-payload-contact","met_name":"mcp-test-payload-contact","note":"mcp-test: नमस्ते 🧵 café é \"quote\" 'sin… | PARSED; inert exact note/place | {"contact_uid":"person:gh:mcp-test-payload-contact","encounter_uid":"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e","state":"PARSED"} | pass | Read through payload issue (same-tool follow-up #90) |
| 89 | log_issue | {"person":"mcp-test-long","title":"mcp-test-long","body":"mcp-test: zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz… | Accept 2000-char body | {"issue_uid":"issue:notes/mcp-test-long#b34588d5","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); w… | pass | Read issue existence; body not exposed (same-tool follow-up #91) |
| 90 | log_encounter | {"person":"person:gh:mcp-test-long","met_handle":"mcp-test-long-contact","met_name":"mcp-test-long-contact","note":"mcp-test: zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz… | Accept full 2000-char note | {"contact_uid":"person:gh:mcp-test-long-contact","encounter_uid":"enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57","state":"PARSED"} | pass | Readback exact note (same-tool follow-up #92) |
| 91 | log_issue | {"person":"mcp-test-single","title":"mcp-test-singleword","body":"mcp-test-singleword","topics":["community-governance"]} | Accept single word | {"issue_uid":"issue:notes/mcp-test-single#21c47f94","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4);… | pass | Read via own issues (same-tool follow-up #94) |
| 92 | log_encounter | {"person":"person:gh:mcp-test-single","met_name":"mcp-test-single-contact","note":"mcp-test-singleword","topics":["community-governance"]} | Accept omitted handle/place | {"contact_uid":"person:name:mcp-test-single-contact","encounter_uid":"enc:8b96002b-328a-48cc-9ae7-6777d69b00e5","state":"PARSED"} | pass | Read default place and uid (same-tool follow-up #95) |
| 93 | log_learning | {"person":"mcp-test-validation","learned":"","topics":["mcp-auth"]} | Reject empty content | {"error":"McpServerError: person and learned are required","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_rpc_error","code":-32602,"message":"person and le… | pass | Recall whether blank learning persisted (final closure: invariants and evidence reviewed) |
| 94 | log_issue | {"person":"mcp-test-validation","title":"","body":"","topics":["mcp-auth"]} | Reject empty content | {"error":"McpServerError: person, title and body are required","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_rpc_error","code":-32602,"message":"person, t… | pass | Read blank issue if accepted (same-tool follow-up #118) |
| 95 | log_encounter | {"person":"person:gh:mcp-test-validation","met_handle":"mcp-test-blank-contact","met_name":"mcp-test-blank-contact","note":"","topics":["mcp-auth"]} | Reject empty note or document allowance | {"error":"McpServerError: note and met_name are required","error_code":"INVALID_ARGUMENT","error_data":{"type":"json_rpc_error","code":-32602,"message":"note and met_n… | pass | Read blank note if accepted (final closure: invariants and evidence reviewed) |
| 96 | talks_for_my_issues | {"person":"person:gh:mcp-test-payload"} | Exact Unicode title and own issue only | T1; 1 rows; hops=5; query=1.818ms; params={"after":"","before":"","day":"","person":"person:gh:mcp-test-payload"} | pass | Graph integrity check (same-tool follow-up #104) |
| 97 | memory_recall | {"issue":"issue:notes/mcp-test-payload#2c4df3e7"} | Exact inert Unicode note/place | T4; 3 rows; hops=3; query=0.475ms; params={"issue":"issue:notes/mcp-test-payload#2c4df3e7"} | pass | Compare equality automatically (same-tool follow-up #98) |
| 98 | memory_recall | {"issue":"issue:notes/mcp-test-long#b34588d5"} | Exact 2000-char note; default place | T4; 1 rows; hops=3; query=0.423ms; params={"issue":"issue:notes/mcp-test-long#b34588d5"} | pass | Compare length (same-tool follow-up #100) |
| 99 | talks_for_issue | {"issue":"issue:notes/mcp-test-single#21c47f94"} | 3-hop result | T2; 1 rows; hops=3; query=0.430ms; params={"issue":"issue:notes/mcp-test-single#21c47f94"} | pass | Confirm LINKED state (same-tool follow-up #116) |
| 100 | memory_recall | {"issue":"issue:notes/mcp-test-single#21c47f94"} | Exact note; no-handle identity | T4; 1 rows; hops=3; query=0.419ms; params={"issue":"issue:notes/mcp-test-single#21c47f94"} | pass | Repeat no duplicate rows (same-tool follow-up #113) |
| 101 | recall_learnings | {"person":"mcp-test-alpha","topic":"oauth"} | Only OAuth tagged alpha learnings | 5 learnings; topics=[[],[],["oauth"],["oauth"],[]] | fail | Compare unknown topic (same-tool follow-up #102) |
| 102 | recall_learnings | {"person":"mcp-test-validation"} | No writes from rejected attempts | 0 learnings; topics=[] | pass | Repeat unknown read topic (same-tool follow-up #108) |
| 103 | list_topics | {} | Exactly original 22 topics | 22 topics | pass | Repeat final topics (same-tool follow-up #142) |
| 104 | talks_for_my_issues | {"person":"person:gh:manvendersingh21"} | Same original issue rows, no lost graph | T1; 5 rows; hops=5; query=1.350ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | pass | Compare full baseline row projection (same-tool follow-up #106) |
| 105 | talks_by_topic | {"topics":"mcp-auth","day":"2026-10-28"} | Zero rows | T6; 0 rows; hops=0; query=1.041ms; params={"after":"","before":"","day":"2026-10-28","topics":"mcp-auth"} | pass | Allowed 10-29 date includes (same-tool follow-up #107) |
| 106 | talks_for_my_issues | {"person":"person:gh:manvendersingh21","day":"2026-10-29"} | Only 10-29 rows | T1; 5 rows; hops=5; query=1.035ms; params={"after":"","before":"","day":"2026-10-29","person":"person:gh:manvendersingh21"} | pass | Compare baseline (same-tool follow-up #115) |
| 107 | talks_by_topic | {"topics":"community-governance","after":"23:59"} | Graceful empty rows and envelope | T6; 0 rows; hops=0; query=0.236ms; params={"after":"23:59","before":"","day":"","topics":"community-governance"} | pass | Final empty-envelope structural check (same-tool follow-up #117) |
| 108 | recall_learnings | {"person":"mcp-test-alpha","topic":"agent-memory"} | Zero records; alpha has no agent-memory learning | 5 learnings; topics=[[],[],[],[],[]] | fail | Unknown topic comparison (same-tool follow-up #109) |
| 109 | recall_learnings | {"person":"mcp-test-alpha","topic":"mcp-test-unknown"} | Unknown slug error | 5 learnings; topics=[[],[],[],[],[]] | fail | Empty topic comparison (same-tool follow-up #110) |
| 110 | recall_learnings | {"person":"mcp-test-alpha","topic":""} | Same all records and full topic arrays | 5 learnings; topics=[["mcp-auth"],["mcp-auth"],["oauth"],["mcp-auth","oauth"],["mcp-auth"]] | pass | Multi-topic preservation (same-tool follow-up #111) |
| 111 | recall_learnings | {"person":"mcp-test-alpha","topic":"mcp-auth"} | Only mcp-auth records; full tags preserved | 5 learnings; topics=[["mcp-auth"],["mcp-auth"],[],["mcp-auth"],["mcp-auth"]] | fail | Compare oauth record (same-tool follow-up #114) |
| 112 | check_alerts | {} | CONTACT only and stable bridge identities | 7 CONTACT alerts | fail | Record all observed bridge ids (same-tool follow-up #143) |
| 113 | memory_recall | {"issue":"issue:manvendersingh21/deepcore-n1#1"} | No test encounters owned by other people | T4; 3 rows; hops=3; query=0.531ms; params={"issue":"issue:manvendersingh21/deepcore-n1#1"} | fail | Confirm synthetic leakage (same-tool follow-up #131) |
| 114 | recall_learnings | {"person":"mcp-test-missing"} | Empty with canonical uid; no leak | 0 learnings; topics=[] | pass | Repeat no mutation (same-tool follow-up #122) |
| 115 | talks_for_my_issues | {} | Schema rejects missing required person | {"error_code":"INVALID_ARGUMENT"} | pass | Blank uid comparison (same-tool follow-up #123) |
| 116 | talks_for_issue | {} | Schema rejects missing issue | {"error_code":"INVALID_ARGUMENT"} | pass | Malformed uid comparison (same-tool follow-up #119) |
| 117 | talks_by_topic | {"topics":"mcp-auth,mcp-auth"} | One row, no multiplication | T6; 1 rows; hops=2; query=0.508ms; params={"after":"","before":"","day":"","topics":"mcp-auth,mcp-auth"} | pass | Whitespace trimming (same-tool follow-up #125) |
| 118 | log_issue | {"person":"mcp-test-timing-1","title":"mcp-test: timing sample 1","body":"mcp-test: measure bridge readiness","topics":["copilot"]} | CLASSIFIED then LINKED within ~10sec | {"issue_uid":"issue:notes/mcp-test-timing-1#c30e2013","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4… | pass | Immediate poll then repeat (same-tool follow-up #120) |
| 119 | talks_for_issue | {"issue":"issue:notes/mcp-test-timing-1#c30e2013"} | 3 hops with LINKED issue | T2; 1 rows; hops=3; query=0.467ms; params={"issue":"issue:notes/mcp-test-timing-1#c30e2013"} | pass | Repeat final state read (same-tool follow-up #121) |
| 120 | log_issue | {"person":"mcp-test-timing-2","title":"mcp-test: timing sample 2","body":"mcp-test: measure bridge readiness","topics":["copilot"]} | CLASSIFIED then LINKED within ~10sec | {"issue_uid":"issue:notes/mcp-test-timing-2#bc4fd6eb","next":"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4… | pass | Immediate poll then repeat (final closure: invariants and evidence reviewed) |
| 121 | talks_for_issue | {"issue":"issue:notes/mcp-test-timing-2#bc4fd6eb"} | 3 hops with LINKED issue | T2; 1 rows; hops=3; query=0.438ms; params={"issue":"issue:notes/mcp-test-timing-2#bc4fd6eb"} | pass | Repeat final state read (same-tool follow-up #135) |
| 122 | recall_learnings | {"person":"mcp-test-beta","topic":"agent-memory"} | Empty since beta only mcp-auth | 2 learnings; topics=[[],[]] | fail | Confirm same filter defect on beta (same-tool follow-up #124) |
| 123 | talks_for_my_issues | {"person":"mcp-test-alpha"} | Canonicalize or reject incorrect UID; document behavior | T1; 0 rows; hops=0; query=0.969ms; params={"after":"","before":"","day":"","person":"mcp-test-alpha"} | fail | Compare full canonical UID (same-tool follow-up #141) |
| 124 | recall_learnings | {"person":"person:gh:mcp-test-alpha"} | Same records as raw handle | 5 learnings; topics=[["mcp-auth"],["mcp-auth"],["oauth"],["mcp-auth","oauth"],["mcp-auth"]] | pass | Compare canonical uid (same-tool follow-up #132) |
| 125 | talks_by_topic | {"topics":"mcp-auth","after":"9:00"} | Reject invalid HH:MM | T6; 1 rows; hops=2; query=0.488ms; params={"after":"09:00","before":"","day":"","topics":"mcp-auth"} | fail | Canonical 09:00 comparison (same-tool follow-up #126) |
| 126 | talks_by_topic | {"topics":"mcp-auth","before":"99:99"} | Reject invalid time | T6; 1 rows; hops=2; query=0.475ms; params={"after":"","before":"","day":"","topics":"mcp-auth"} | fail | Echoed params (same-tool follow-up #127) |
| 127 | talks_by_topic | {"topics":" mcp-auth , oauth "} | Trim slug boundaries or reject cleanly | T6; 0 rows; hops=0; query=0.348ms; params={"after":"","before":"","day":"","topics":" mcp-auth , oauth "} | fail | No topic mutation (same-tool follow-up #128) |
| 128 | talks_by_topic | {"topics":"mcp-auth","day":"2026-10-28","after":"10:30","before":"10:31"} | Zero rows; all filters applied | T6; 0 rows; hops=0; query=0.185ms; params={"after":"10:30","before":"10:31","day":"2026-10-28","topics":"mcp-auth"} | pass | R2 matching interval (same-tool follow-up #129) |
| 129 | talks_by_topic | {"topics":"mcp-auth","after":"10:30","before":"10:30"} | Empty half-open interval | T6; 0 rows; hops=0; query=0.225ms; params={"after":"10:30","before":"10:30","day":"","topics":"mcp-auth"} | pass | R2 one-minute interval (same-tool follow-up #130) |
| 130 | talks_by_topic | {"topics":"mcp-auth,oauth"} | Two topic rows, same one talk | T6; 2 rows; hops=2; query=0.419ms; params={"after":"","before":"","day":"","topics":"mcp-auth,oauth"} | pass | R2 compare related talks unique set (same-tool follow-up #134) |
| 131 | memory_recall | {} | INVALID_ARGUMENT | {"error_code":"INVALID_ARGUMENT"} | pass | R2 malformed nonempty uid (same-tool follow-up #137) |
| 132 | recall_learnings | {} | INVALID_ARGUMENT | {"error_code":"INVALID_ARGUMENT"} | pass | R2 unknown prefixed person (same-tool follow-up #138) |
| 133 | people_to_meet | {} | INVALID_ARGUMENT | {"error_code":"INVALID_ARGUMENT"} | pass | R2 correct alpha uid (same-tool follow-up #139) |
| 134 | talks_by_topic | {} | INVALID_ARGUMENT | {"error_code":"INVALID_ARGUMENT"} | pass | R2 invalid topic (same-tool follow-up #136) |
| 135 | talks_for_issue | {"issue":"mcp-test: ' MATCH (n) RETURN n //"} | No rows; must not execute Cypher; existing missing-UID validation defect | T2; 0 rows; hops=0; query=0.216ms; params={"issue":"mcp-test: ' MATCH (n) RETURN n //"} | fail | R2 presenter graph intact (final closure: invariants and evidence reviewed) |
| 136 | talks_by_topic | {"topics":"mcp-auth","day":"2026-10-29","after":"10:30","before":"10:31"} | One talk at 10:30 | T6; 1 rows; hops=2; query=0.445ms; params={"after":"10:30","before":"10:31","day":"2026-10-29","topics":"mcp-auth"} | pass | Closure: exact bounds checked (same-tool follow-up #140) |
| 137 | memory_recall | {"issue":"issue:notes/mcp-test-missing#none"} | Known defect: empty success instead of nonexistent uid error | T4; 0 rows; hops=0; query=0.160ms; params={"issue":"issue:notes/mcp-test-missing#none"} | fail | Closure: missing-uid defect reproduced (final closure: invariants and evidence reviewed) |
| 138 | recall_learnings | {"person":"mcp-test-beta"} | Exactly own 2 duplicate records | 2 learnings; topics=[["mcp-auth"],["mcp-auth"]] | pass | Closure: ownership holds (final closure: invariants and evidence reviewed) |
| 139 | people_to_meet | {"person":"person:gh:mcp-test-alpha"} | Own issue path; no test contact listed as speaker | T3; 1 rows; hops=5; query=1.414ms; params={"limit":5,"person":"person:gh:mcp-test-alpha"} | pass | Closure: speaker query unchanged (final closure: invariants and evidence reviewed) |
| 140 | talks_by_topic | {"topics":"mcp-test-unknown"} | Known defect: empty success | T6; 0 rows; hops=0; query=0.330ms; params={"after":"","before":"","day":"","topics":"mcp-test-unknown"} | fail | Closure: unknown-slug defect reproduced (final closure: invariants and evidence reviewed) |
| 141 | talks_for_my_issues | {"person":"person:gh:manvendersingh21"} | Same baseline rows and 5-hop structure | T1; 5 rows; hops=5; query=1.326ms; params={"after":"","before":"","day":"","person":"person:gh:manvendersingh21"} | pass | Closure: no observed graph loss (final closure: invariants and evidence reviewed) |
| 142 | list_topics | {} | Exactly original 22 topics | 22 topics | pass | Closure: no new topics created (final closure: invariants and evidence reviewed) |
| 143 | check_alerts | {} | Only CONTACT records; inventory auto-created bridges | 7 CONTACT alerts | fail | Closure: record observed bridge ids (final closure: invariants and evidence reviewed) |

## Everything created: direct writes and observed nodes

The following inventory accounts for every successful write. Rejected calls are not counted as creations. Support nodes are returned identities associated with these writes; no claim is made that a support node was absent before the run unless the baseline established that. Auto-created bridges listed below were observed after an empty initial alert response. The server does not expose an unrestricted node/bridge listing, so additional internal background-agent nodes cannot be enumerated through this interface. Nothing was deleted.

### Successful write records (19)

| Call | Kind | Record UID | Logger | Related returned UID |
| --- | --- | --- | --- | --- |
| #28 | log_learning | learning:9d307e75-e63d-42d8-985b-f2471f6a6303 | mcp-test-alpha | person:gh:mcp-test-alpha |
| #29 | log_issue | issue:notes/mcp-test-alpha#7c136aa7 | mcp-test-alpha | repo:notes/mcp-test-alpha |
| #31 | log_encounter | enc:308e4ce9-9740-456d-85b4-dc315561230f | person:gh:mcp-test-alpha | person:gh:mcp-test-contact |
| #46 | log_learning | learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9 | mcp-test-alpha | person:gh:mcp-test-alpha |
| #47 | log_learning | learning:05362c1e-7200-494e-9185-f8eabf1bcd25 | mcp-test-beta | person:gh:mcp-test-beta |
| #48 | log_learning | learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2 | mcp-test-beta | person:gh:mcp-test-beta |
| #50 | log_learning | learning:90d70c0a-1147-414c-b29e-c659500bcb48 | mcp-test-alpha | person:gh:mcp-test-alpha |
| #51 | log_learning | learning:58efc5ad-c647-4c10-8137-d0f25a004ae9 | mcp-test-alpha | person:gh:mcp-test-alpha |
| #55 | log_issue | issue:notes/mcp-test-beta#f6cc6fcd | mcp-test-beta | repo:notes/mcp-test-beta |
| #56 | log_encounter | enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a | person:gh:mcp-test-beta | person:gh:mcp-test-beta-private |
| #57 | log_learning | learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f | mcp-test-alpha | person:gh:mcp-test-alpha |
| #87 | log_issue | issue:notes/mcp-test-payload#2c4df3e7 | mcp-test-payload | repo:notes/mcp-test-payload |
| #88 | log_encounter | enc:80da1b5b-f799-4ad6-82b2-435c2adc194e | person:gh:mcp-test-payload | person:gh:mcp-test-payload-contact |
| #89 | log_issue | issue:notes/mcp-test-long#b34588d5 | mcp-test-long | repo:notes/mcp-test-long |
| #90 | log_encounter | enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57 | person:gh:mcp-test-long | person:gh:mcp-test-long-contact |
| #91 | log_issue | issue:notes/mcp-test-single#21c47f94 | mcp-test-single | repo:notes/mcp-test-single |
| #92 | log_encounter | enc:8b96002b-328a-48cc-9ae7-6777d69b00e5 | person:gh:mcp-test-single | person:name:mcp-test-single-contact |
| #118 | log_issue | issue:notes/mcp-test-timing-1#c30e2013 | mcp-test-timing-1 | repo:notes/mcp-test-timing-1 |
| #120 | log_issue | issue:notes/mcp-test-timing-2#bc4fd6eb | mcp-test-timing-2 | repo:notes/mcp-test-timing-2 |

### Person identities (12)

- person:gh:mcp-test-alpha
- person:gh:mcp-test-contact
- person:gh:mcp-test-beta
- person:gh:mcp-test-beta-private
- person:gh:mcp-test-payload
- person:gh:mcp-test-payload-contact
- person:gh:mcp-test-long
- person:gh:mcp-test-long-contact
- person:gh:mcp-test-single
- person:name:mcp-test-single-contact
- person:gh:mcp-test-timing-1
- person:gh:mcp-test-timing-2

### Notes repos (7)

- repo:notes/mcp-test-alpha
- repo:notes/mcp-test-beta
- repo:notes/mcp-test-payload
- repo:notes/mcp-test-long
- repo:notes/mcp-test-single
- repo:notes/mcp-test-timing-1
- repo:notes/mcp-test-timing-2

### Observed new CONTACT bridges (7)

| Bridge UID | Issue UID | Contact | State | Hops |
| --- | --- | --- | --- | --- |
| bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd | issue:manvendersingh21/deepcore-n1#1 | mcp-test-contact | ACTION_READY | 4 |
| bridge:d20f4fac-77b0-4482-8757-ed74dbf4208c | issue:manvendersingh21/deepcore-n1#4 | mcp-test-long-contact | ACTION_READY | 4 |
| bridge:483f0dfa-7642-445f-a35e-538166eceef8 | issue:manvendersingh21/deepcore-n1#1 | mcp-test-payload-contact | ACTION_READY | 4 |
| bridge:e5ad65fd-72b5-46b0-abc5-80f3fa12f158 | issue:notes/mcp-test-payload#2c4df3e7 | mcp-test-contact | ACTION_READY | 3 |
| bridge:34be0887-cf6b-4dfa-9c3f-c0f60f756d83 | issue:notes/mcp-test-payload#2c4df3e7 | mcp-test-beta-private | ACTION_READY | 3 |
| bridge:9cf26a79-4ad7-470a-8cec-b5f2454dd6f5 | issue:manvendersingh21/deepcore-n1#1 | mcp-test-beta-private | ACTION_READY | 4 |
| bridge:1a81cd1e-a24c-4723-8c2d-6b7bbacff29b | issue:notes/mcp-test-beta#f6cc6fcd | mcp-test-contact | ACTION_READY | 3 |

Four of these bridge records reference existing presenter issues. Keep this list for owner-controlled restoration from data/snapshots/baseline.json; this run did not invoke restoration.

## Full evidence appendix

Each call records the exact arguments, complete tool result, parsed data, expected outcome, follow-up, verdict and measured wall time. Initial five baseline calls did not retain ISO timestamps, but their complete results and wall times are retained. Tool inventory declarations follow the calls.

```json
{
  "tests": [
    {
      "id": 1,
      "label": "topics baseline",
      "tool": "list_topics",
      "args": {},
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[\n  {\n    \"name\": \"Agent Memory\",\n    \"slug\": \"agent-memory\"\n  },\n  {\n    \"name\": \"Agentic Security\",\n    \"slug\": \"agentic-security\"\n  },\n  {\n    \"name\": \"Agentic Workflows\",\n    \"slug\": \"agentic-workflows\"\n  },\n  {\n    \"name\": \"AI-Assisted Testing\",\n    \"slug\": \"ai-assisted-testing\"\n  },\n  {\n    \"name\": \"AI Context\",\n    \"slug\": \"ai-context\"\n  },\n  {\n    \"name\": \"AI Operations\",\n    \"slug\": \"ai-operations\"\n  },\n  {\n    \"name\": \"CI/CD\",\n    \"slug\": \"cicd\"\n  },\n  {\n    \"name\": \"Community Governance\",\n    \"slug\": \"community-governance\"\n  },\n  {\n    \"name\": \"Copilot\",\n    \"slug\": \"copilot\"\n  },\n  {\n    \"name\": \"Dependencies\",\n    \"slug\": \"dependencies\"\n  },\n  {\n    \"name\": \"Deterministic Verification\",\n    \"slug\": \"deterministic-verification\"\n  },\n  {\n    \"name\": \"Developer Experience\",\n    \"slug\": \"developer-experience\"\n  },\n  {\n    \"name\": \"Enterprise AI Adoption\",\n    \"slug\": \"enterprise-ai-adoption\"\n  },\n  {\n    \"name\": \"Frontier Models\",\n    \"slug\": \"frontier-models\"\n  },\n  {\n    \"name\": \"GitHub Actions\",\n    \"slug\": \"github-actions\"\n  },\n  {\n    \"name\": \"MCP Auth\",\n    \"slug\": \"mcp-auth\"\n  },\n  {\n    \"name\": \"OAuth\",\n    \"slug\": \"oauth\"\n  },\n  {\n    \"name\": \"Open Source\",\n    \"slug\": \"open-source\"\n  },\n  {\n    \"name\": \"Package Management\",\n    \"slug\": \"package-management\"\n  },\n  {\n    \"name\": \"Platform Engineering\",\n    \"slug\": \"platform-engineering\"\n  },\n  {\n    \"name\": \"RCA Agent\",\n    \"slug\": \"rca-agent\"\n  },\n  {\n    \"name\": \"Supply Chain Security\",\n    \"slug\": \"supply-chain-security\"\n  }\n]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "wall_ms": 3642,
      "data": [
        {
          "name": "Agent Memory",
          "slug": "agent-memory"
        },
        {
          "name": "Agentic Security",
          "slug": "agentic-security"
        },
        {
          "name": "Agentic Workflows",
          "slug": "agentic-workflows"
        },
        {
          "name": "AI-Assisted Testing",
          "slug": "ai-assisted-testing"
        },
        {
          "name": "AI Context",
          "slug": "ai-context"
        },
        {
          "name": "AI Operations",
          "slug": "ai-operations"
        },
        {
          "name": "CI/CD",
          "slug": "cicd"
        },
        {
          "name": "Community Governance",
          "slug": "community-governance"
        },
        {
          "name": "Copilot",
          "slug": "copilot"
        },
        {
          "name": "Dependencies",
          "slug": "dependencies"
        },
        {
          "name": "Deterministic Verification",
          "slug": "deterministic-verification"
        },
        {
          "name": "Developer Experience",
          "slug": "developer-experience"
        },
        {
          "name": "Enterprise AI Adoption",
          "slug": "enterprise-ai-adoption"
        },
        {
          "name": "Frontier Models",
          "slug": "frontier-models"
        },
        {
          "name": "GitHub Actions",
          "slug": "github-actions"
        },
        {
          "name": "MCP Auth",
          "slug": "mcp-auth"
        },
        {
          "name": "OAuth",
          "slug": "oauth"
        },
        {
          "name": "Open Source",
          "slug": "open-source"
        },
        {
          "name": "Package Management",
          "slug": "package-management"
        },
        {
          "name": "Platform Engineering",
          "slug": "platform-engineering"
        },
        {
          "name": "RCA Agent",
          "slug": "rca-agent"
        },
        {
          "name": "Supply Chain Security",
          "slug": "supply-chain-security"
        }
      ],
      "expected": "Exactly 22 unique topic slugs",
      "follow": "Query each slug",
      "verdict": "pass"
    },
    {
      "id": 2,
      "label": "presenter speakers",
      "tool": "people_to_meet",
      "args": {
        "person": "person:gh:manvendersingh21"
      },
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T3\",\n  \"params\": {\n    \"limit\": 5,\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.485205,\n  \"rows\": [\n    {\n      \"company\": \"Momentic\",\n      \"hops\": 5,\n      \"person\": \"Jeff An\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"Agentic Security\"\n    },\n    {\n      \"company\": \"Pomerium\",\n      \"hops\": 5,\n      \"person\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"Agentic Security\"\n    },\n    {\n      \"company\": \"Pomerium\",\n      \"hops\": 5,\n      \"person\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"company\": \"Pomerium\",\n      \"hops\": 5,\n      \"person\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"OAuth\"\n    },\n    {\n      \"company\": \"Momentic\",\n      \"hops\": 5,\n      \"person\": \"Jeff An\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"Deterministic Verification\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"label\": \"Issue\",\n        \"name\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:agentic-security\",\n        \"label\": \"Topic\",\n        \"name\": \"Agentic Security\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-07\",\n        \"label\": \"Talk\",\n        \"name\": \"Beyond pass or fail: How agents verify AI-generated code\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jeff-an\",\n        \"label\": \"Person\",\n        \"name\": \"Jeff An\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:oauth\",\n        \"label\": \"Topic\",\n        \"name\": \"OAuth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#2\",\n        \"label\": \"Issue\",\n        \"name\": \"Race condition & hallucinated root-cause in multi-agent state handoff during 2am incident triage\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:deterministic-verification\",\n        \"label\": \"Topic\",\n        \"name\": \"Deterministic Verification\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#1\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jeff-an\",\n        \"dst_uid\": \"talk:U26-07\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"dst_uid\": \"topic:oauth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:oauth\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#2\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#2\",\n        \"dst_uid\": \"topic:deterministic-verification\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:deterministic-verification\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "wall_ms": 6859,
      "data": {
        "template": "T3",
        "params": {
          "limit": 5,
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.485205,
        "rows": [
          {
            "company": "Momentic",
            "hops": 5,
            "person": "Jeff An",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "Agentic Security"
          },
          {
            "company": "Pomerium",
            "hops": 5,
            "person": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "Agentic Security"
          },
          {
            "company": "Pomerium",
            "hops": 5,
            "person": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          },
          {
            "company": "Pomerium",
            "hops": 5,
            "person": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "OAuth"
          },
          {
            "company": "Momentic",
            "hops": 5,
            "person": "Jeff An",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "Deterministic Verification"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#1",
              "label": "Issue",
              "name": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
              "state": "LINKED"
            },
            {
              "uid": "topic:agentic-security",
              "label": "Topic",
              "name": "Agentic Security",
              "state": ""
            },
            {
              "uid": "talk:U26-07",
              "label": "Talk",
              "name": "Beyond pass or fail: How agents verify AI-generated code",
              "state": ""
            },
            {
              "uid": "person:name:jeff-an",
              "label": "Person",
              "name": "Jeff An",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "topic:oauth",
              "label": "Topic",
              "name": "OAuth",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#2",
              "label": "Issue",
              "name": "Race condition & hallucinated root-cause in multi-agent state handoff during 2am incident triage",
              "state": "LINKED"
            },
            {
              "uid": "topic:deterministic-verification",
              "label": "Topic",
              "name": "Deterministic Verification",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#1"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#1",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jeff-an",
              "dst_uid": "talk:U26-07"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#1",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#1",
              "dst_uid": "topic:oauth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:oauth"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#2"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#2",
              "dst_uid": "topic:deterministic-verification"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:deterministic-verification"
            }
          ]
        }
      },
      "expected": "T3 envelope, 5-hop relevant speakers",
      "follow": "Compare synthetic person speakers",
      "verdict": "pass"
    },
    {
      "id": 3,
      "label": "alerts baseline",
      "tool": "check_alerts",
      "args": {},
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "wall_ms": 9639,
      "data": [],
      "expected": "CONTACT alerts baseline",
      "follow": "Create test encounter then recheck",
      "verdict": "pass"
    },
    {
      "id": 4,
      "label": "presenter baseline",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21"
      },
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 2.058729,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "wall_ms": 11955,
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 2.058729,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "expected": "Nonempty T1 rows, 5 hops, graph runtime ~0.5–5ms",
      "follow": "Test day/time and unknown identities",
      "verdict": "pass"
    },
    {
      "id": 5,
      "label": "learnings baseline",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha"
      },
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "wall_ms": 15112,
      "data": {
        "learnings": [],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "expected": "No preexisting test learnings",
      "follow": "Write then recall",
      "verdict": "pass"
    },
    {
      "id": 6,
      "label": "known topic",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth"
      },
      "expected": "T5 envelope with known talks",
      "follow": "Use observed date/time bounds",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 1.202544,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 1.202544,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 2640,
      "at": "2026-10-24T21:35:27.900Z",
      "verdict": "pass"
    },
    {
      "id": 7,
      "label": "existing issue",
      "tool": "talks_for_issue",
      "args": {
        "issue": "issue:manvendersingh21/deepcore-n1#1"
      },
      "expected": "3-hop envelope and rows",
      "follow": "Test missing issue",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"issue:manvendersingh21/deepcore-n1#1\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 3,\n  \"run_time_ms\": 1.055508,\n  \"rows\": [\n    {\n      \"hops\": 3,\n      \"speaker\": \"Jeff An\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"talk_uid\": \"talk:U26-07\",\n      \"topic\": \"agentic-security\"\n    },\n    {\n      \"hops\": 3,\n      \"speaker\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"talk_uid\": \"talk:U26-09\",\n      \"topic\": \"agentic-security\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"label\": \"Issue\",\n        \"name\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:agentic-security\",\n        \"label\": \"Topic\",\n        \"name\": \"Agentic Security\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-07\",\n        \"label\": \"Talk\",\n        \"name\": \"Beyond pass or fail: How agents verify AI-generated code\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jeff-an\",\n        \"label\": \"Person\",\n        \"name\": \"Jeff An\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jeff-an\",\n        \"dst_uid\": \"talk:U26-07\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "issue:manvendersingh21/deepcore-n1#1"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 3,
        "run_time_ms": 1.055508,
        "rows": [
          {
            "hops": 3,
            "speaker": "Jeff An",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "talk_uid": "talk:U26-07",
            "topic": "agentic-security"
          },
          {
            "hops": 3,
            "speaker": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "talk_uid": "talk:U26-09",
            "topic": "agentic-security"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:manvendersingh21/deepcore-n1#1",
              "label": "Issue",
              "name": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
              "state": "LINKED"
            },
            {
              "uid": "topic:agentic-security",
              "label": "Topic",
              "name": "Agentic Security",
              "state": ""
            },
            {
              "uid": "talk:U26-07",
              "label": "Talk",
              "name": "Beyond pass or fail: How agents verify AI-generated code",
              "state": ""
            },
            {
              "uid": "person:name:jeff-an",
              "label": "Person",
              "name": "Jeff An",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#1",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jeff-an",
              "dst_uid": "talk:U26-07"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 4965,
      "at": "2026-10-24T21:35:30.225Z",
      "verdict": "pass"
    },
    {
      "id": 8,
      "label": "unknown person",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:mcp-test-missing"
      },
      "expected": "Clean error for nonexistent person per brief",
      "follow": "Compare empty person",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:mcp-test-missing\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.917223,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:mcp-test-missing"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.917223,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 7353,
      "at": "2026-10-24T21:35:32.613Z",
      "verdict": "fail"
    },
    {
      "id": 9,
      "label": "existing issue memory",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:manvendersingh21/deepcore-n1#1"
      },
      "expected": "T4 envelope",
      "follow": "Create matching test encounter",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:manvendersingh21/deepcore-n1#1\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.935113,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:manvendersingh21/deepcore-n1#1"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 0,
        "run_time_ms": 0.935113,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 9743,
      "at": "2026-10-24T21:35:35.003Z",
      "verdict": "pass"
    },
    {
      "id": 10,
      "label": "missing issue",
      "tool": "talks_for_issue",
      "args": {
        "issue": "issue:mcp-test-missing/notes#999999"
      },
      "expected": "Clean error per brief",
      "follow": "Compare malformed uid",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"issue:mcp-test-missing/notes#999999\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.131664,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "issue:mcp-test-missing/notes#999999"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 0,
        "run_time_ms": 0.131664,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 12240,
      "at": "2026-10-24T21:35:37.500Z",
      "verdict": "fail"
    },
    {
      "id": 11,
      "label": "empty person",
      "tool": "people_to_meet",
      "args": {
        "person": ""
      },
      "expected": "Clean validation error",
      "follow": "Compare nonexistent person",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T3\",\n  \"params\": {\n    \"limit\": 5,\n    \"person\": \"\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.203452,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T3",
        "params": {
          "limit": 5,
          "person": ""
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.203452,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 14589,
      "at": "2026-10-24T21:35:39.849Z",
      "verdict": "fail"
    },
    {
      "id": 12,
      "label": "malformed issue",
      "tool": "memory_recall",
      "args": {
        "issue": "bad"
      },
      "expected": "Clean error per brief",
      "follow": "Compare empty uid",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"bad\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.169514,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "bad"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 0,
        "run_time_ms": 0.169514,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 17562,
      "at": "2026-10-24T21:35:42.822Z",
      "verdict": "fail"
    },
    {
      "id": 13,
      "label": "unknown recall topic",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": "mcp-test-unknown"
      },
      "expected": "Unknown slug error",
      "follow": "Compare valid topic",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 21035,
      "at": "2026-10-24T21:35:46.295Z",
      "verdict": "fail"
    },
    {
      "id": 14,
      "label": "unknown topics",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-test-unknown"
      },
      "expected": "Unknown slug error",
      "follow": "Mixed valid and unknown slug",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-test-unknown\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.340094,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-test-unknown"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.340094,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 23372,
      "at": "2026-10-24T21:35:48.632Z",
      "verdict": "fail"
    },
    {
      "id": 15,
      "label": "empty topics",
      "tool": "talks_by_topic",
      "args": {
        "topics": ""
      },
      "expected": "Empty topics error",
      "follow": "Whitespace topics",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.338628,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": ""
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.338628,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 25728,
      "at": "2026-10-24T21:35:50.988Z",
      "verdict": "fail"
    },
    {
      "id": 16,
      "label": "invalid day",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21",
        "day": "2026-13-99"
      },
      "expected": "Invalid date error",
      "follow": "Valid no-result date",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.270544,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.270544,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 28237,
      "at": "2026-10-24T21:35:53.497Z",
      "verdict": "fail"
    },
    {
      "id": 17,
      "label": "invalid time",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21",
        "after": "99:99"
      },
      "expected": "Invalid time error",
      "follow": "Compare valid exact bounds",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.313804,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.313804,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 31106,
      "at": "2026-10-24T21:35:56.366Z",
      "verdict": "fail"
    },
    {
      "id": 18,
      "label": "day match",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2026-10-29"
      },
      "expected": "One MCP Auth talk",
      "follow": "Exclude via other date",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"2026-10-29\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.440554,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "2026-10-29",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.440554,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 2604,
      "at": "2026-10-24T21:36:09.507Z",
      "verdict": "pass"
    },
    {
      "id": 19,
      "label": "time excludes",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "after": "23:59",
        "before": "00:01"
      },
      "expected": "Zero rows or explicit inverted range error",
      "follow": "Exact interval",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"23:59\",\n    \"before\": \"00:01\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.216849,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "23:59",
          "before": "00:01",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.216849,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 5181,
      "at": "2026-10-24T21:36:12.084Z",
      "verdict": "pass"
    },
    {
      "id": 20,
      "label": "mixed unknown slug",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth,mcp-test-unknown"
      },
      "expected": "Unknown slug error",
      "follow": "Check topics still 22",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth,mcp-test-unknown\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.44686,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth,mcp-test-unknown"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.44686,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 7909,
      "at": "2026-10-24T21:36:14.812Z",
      "verdict": "fail"
    },
    {
      "id": 21,
      "label": "day excludes",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2030-01-01"
      },
      "expected": "Zero rows",
      "follow": "Inspect echoed params",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.459191,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.459191,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 10371,
      "at": "2026-10-24T21:36:17.274Z",
      "verdict": "fail"
    },
    {
      "id": 22,
      "label": "whitespace topic",
      "tool": "talks_by_topic",
      "args": {
        "topics": "   "
      },
      "expected": "Validation error",
      "follow": "Uppercase slug",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"   \"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.334441,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "   "
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.334441,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 13035,
      "at": "2026-10-24T21:36:19.938Z",
      "verdict": "fail"
    },
    {
      "id": 23,
      "label": "uppercase topic",
      "tool": "talks_by_topic",
      "args": {
        "topics": "MCP-AUTH"
      },
      "expected": "Reject noncanonical slug",
      "follow": "Known canonical topic",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"MCP-AUTH\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.321324,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "MCP-AUTH"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.321324,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 17243,
      "at": "2026-10-24T21:36:24.146Z",
      "verdict": "fail"
    },
    {
      "id": 24,
      "label": "empty optional params",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21",
        "day": "",
        "after": "",
        "before": ""
      },
      "expected": "Same baseline",
      "follow": "Valid date excludes",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.36972,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.36972,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 20298,
      "at": "2026-10-24T21:36:27.201Z",
      "verdict": "pass"
    },
    {
      "id": 25,
      "label": "empty recall uid",
      "tool": "memory_recall",
      "args": {
        "issue": ""
      },
      "expected": "Validation error",
      "follow": "Missing issue",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.137395,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": ""
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 0,
        "run_time_ms": 0.137395,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 23509,
      "at": "2026-10-24T21:36:30.412Z",
      "verdict": "fail"
    },
    {
      "id": 26,
      "label": "day excludes presenter",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21",
        "day": "2030-01-01"
      },
      "expected": "Zero rows",
      "follow": "Check params",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.316068,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.316068,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 26222,
      "at": "2026-10-24T21:36:33.125Z",
      "verdict": "fail"
    },
    {
      "id": 27,
      "label": "valid recall filter",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": "mcp-auth"
      },
      "expected": "Empty before writes",
      "follow": "Learning round-trip",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 28721,
      "at": "2026-10-24T21:36:35.624Z",
      "verdict": "pass"
    },
    {
      "id": 28,
      "label": "learning first",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-alpha",
        "learned": "mcp-test: MCP auth scopes require explicit checks.",
        "source": "mcp-test-roundtrip",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "New learning with related MCP Auth talk",
      "follow": "Recall byte-for-byte and second overlap",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\",\n  \"person_uid\": \"person:gh:mcp-test-alpha\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"mcp-auth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303",
        "person_uid": "person:gh:mcp-test-alpha",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "mcp-auth"
        ]
      },
      "wall_ms": 3853,
      "at": "2026-10-24T21:36:53.396Z",
      "verdict": "pass"
    },
    {
      "id": 29,
      "label": "issue timing seed",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-alpha",
        "title": "mcp-test: MCP auth bridge timing",
        "body": "mcp-test: investigate MCP authorization scopes.",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "CLASSIFIED issue then bridged talks within ~10s",
      "follow": "Poll issue immediately",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-alpha\",\n  \"repo_uid\": \"repo:notes/mcp-test-alpha\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-alpha#7c136aa7",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-alpha",
        "repo_uid": "repo:notes/mcp-test-alpha",
        "state": "CLASSIFIED"
      },
      "wall_ms": 3597,
      "at": "2026-10-24T21:36:56.994Z",
      "verdict": "pass"
    },
    {
      "id": 30,
      "label": "issue immediate poll",
      "tool": "talks_for_issue",
      "args": {
        "issue": "issue:notes/mcp-test-alpha#7c136aa7"
      },
      "expected": "3-hop talks",
      "follow": "Delayed poll and alerts",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-alpha#7c136aa7\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.392288,\n  \"rows\": [\n    {\n      \"hops\": 3,\n      \"speaker\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"talk_uid\": \"talk:U26-09\",\n      \"topic\": \"mcp-auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: MCP auth bridge timing\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "issue:notes/mcp-test-alpha#7c136aa7"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 3,
        "run_time_ms": 0.392288,
        "rows": [
          {
            "hops": 3,
            "speaker": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "talk_uid": "talk:U26-09",
            "topic": "mcp-auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "label": "Issue",
              "name": "mcp-test: MCP auth bridge timing",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 3061,
      "at": "2026-10-24T21:37:00.056Z",
      "verdict": "pass"
    },
    {
      "id": 31,
      "label": "encounter seed",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-alpha",
        "met_handle": "mcp-test-contact",
        "met_name": "mcp-test-contact",
        "note": "mcp-test: shared MCP auth experience.",
        "place": "mcp-test-hallway",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Recorded encounter under test identities",
      "follow": "Recall via issue and CONTACT alert",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"contact_uid\": \"person:gh:mcp-test-contact\",\n  \"encounter_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n  \"state\": \"PARSED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "contact_uid": "person:gh:mcp-test-contact",
        "encounter_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
        "state": "PARSED"
      },
      "wall_ms": 3255,
      "at": "2026-10-24T21:37:03.312Z",
      "verdict": "pass"
    },
    {
      "id": 32,
      "label": "date param isolation",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2030-01-01",
        "after": "10:30",
        "before": "10:31"
      },
      "expected": "No rows due date",
      "follow": "Check exact time boundaries",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"10:30\",\n    \"before\": \"10:31\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 1.134626,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "10:30",
          "before": "10:31",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 1.134626,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 3223,
      "at": "2026-10-24T21:37:06.536Z",
      "verdict": "pass"
    },
    {
      "id": 33,
      "label": "learning readback first",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha"
      },
      "expected": "Exact first learning",
      "follow": "Second overlapping learning",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 5871,
      "at": "2026-10-24T21:37:09.184Z",
      "verdict": "pass"
    },
    {
      "id": 34,
      "label": "presenter time excludes",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21",
        "after": "23:59"
      },
      "expected": "Zero rows",
      "follow": "Inspect echoed after",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"23:59\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 1.00355,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "23:59",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 1.00355,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 8492,
      "at": "2026-10-24T21:37:11.805Z",
      "verdict": "pass"
    },
    {
      "id": 35,
      "label": "unknown speaker identity",
      "tool": "people_to_meet",
      "args": {
        "person": "person:gh:mcp-test-missing"
      },
      "expected": "Clean error per brief",
      "follow": "Test alpha isolation",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T3\",\n  \"params\": {\n    \"limit\": 5,\n    \"person\": \"person:gh:mcp-test-missing\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.99381,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T3",
        "params": {
          "limit": 5,
          "person": "person:gh:mcp-test-missing"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.99381,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 11330,
      "at": "2026-10-24T21:37:14.643Z",
      "verdict": "fail"
    },
    {
      "id": 36,
      "label": "contact alert after encounter",
      "tool": "check_alerts",
      "args": {},
      "expected": "CONTACT bridge for test issue and contact only",
      "follow": "Verify no TALK alerts",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[\n  {\n    \"bridge_uid\": \"bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  }\n]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": [
        {
          "bridge_uid": "bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd",
          "contact": "mcp-test-contact",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        }
      ],
      "wall_ms": 2894,
      "at": "2026-10-24T21:37:28.497Z",
      "verdict": "fail"
    },
    {
      "id": 37,
      "label": "test person own issues",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:mcp-test-alpha"
      },
      "expected": "Only test issue, five hops",
      "follow": "Beta has no issues",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:mcp-test-alpha\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 0.913611,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"mcp-test: MCP auth bridge timing\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:mcp-test-alpha\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-alpha\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"repo:notes/mcp-test-alpha\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: MCP auth bridge timing\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:mcp-test-alpha\",\n        \"dst_uid\": \"repo:notes/mcp-test-alpha\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:notes/mcp-test-alpha\",\n        \"dst_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:mcp-test-alpha"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 0.913611,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "mcp-test: MCP auth bridge timing",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:mcp-test-alpha",
              "label": "Person",
              "name": "mcp-test-alpha",
              "state": ""
            },
            {
              "uid": "repo:notes/mcp-test-alpha",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "label": "Issue",
              "name": "mcp-test: MCP auth bridge timing",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:mcp-test-alpha",
              "dst_uid": "repo:notes/mcp-test-alpha"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:notes/mcp-test-alpha",
              "dst_uid": "issue:notes/mcp-test-alpha#7c136aa7"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 5403,
      "at": "2026-10-24T21:37:31.006Z",
      "verdict": "pass"
    },
    {
      "id": 38,
      "label": "test issue memory",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-alpha#7c136aa7"
      },
      "expected": "Own contact plus exact note and place",
      "follow": "Repeat no duplicate contacts",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-alpha#7c136aa7\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 1.04619,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:37:03.099287+00:00\",\n      \"note\": \"mcp-test: shared MCP auth experience.\",\n      \"place\": \"mcp-test-hallway\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: MCP auth bridge timing\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"person:gh:mcp-test-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-alpha#7c136aa7"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 1.04619,
        "rows": [
          {
            "contact": "mcp-test-contact",
            "contact_uid": "person:gh:mcp-test-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:37:03.099287+00:00",
            "note": "mcp-test: shared MCP auth experience.",
            "place": "mcp-test-hallway",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "label": "Issue",
              "name": "mcp-test: MCP auth bridge timing",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-contact",
              "label": "Person",
              "name": "mcp-test-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "person:gh:mcp-test-contact"
            }
          ]
        }
      },
      "wall_ms": 7683,
      "at": "2026-10-24T21:37:33.286Z",
      "verdict": "pass"
    },
    {
      "id": 39,
      "label": "test person speakers",
      "tool": "people_to_meet",
      "args": {
        "person": "person:gh:mcp-test-alpha"
      },
      "expected": "Nick Taylor relevant, not synthetic contact",
      "follow": "Compare beta",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T3\",\n  \"params\": {\n    \"limit\": 5,\n    \"person\": \"person:gh:mcp-test-alpha\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 0.64675,\n  \"rows\": [\n    {\n      \"company\": \"Pomerium\",\n      \"hops\": 5,\n      \"person\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:mcp-test-alpha\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-alpha\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"repo:notes/mcp-test-alpha\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: MCP auth bridge timing\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:mcp-test-alpha\",\n        \"dst_uid\": \"repo:notes/mcp-test-alpha\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:notes/mcp-test-alpha\",\n        \"dst_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T3",
        "params": {
          "limit": 5,
          "person": "person:gh:mcp-test-alpha"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5",
        "hops": 5,
        "run_time_ms": 0.64675,
        "rows": [
          {
            "company": "Pomerium",
            "hops": 5,
            "person": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:mcp-test-alpha",
              "label": "Person",
              "name": "mcp-test-alpha",
              "state": ""
            },
            {
              "uid": "repo:notes/mcp-test-alpha",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "label": "Issue",
              "name": "mcp-test: MCP auth bridge timing",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:mcp-test-alpha",
              "dst_uid": "repo:notes/mcp-test-alpha"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:notes/mcp-test-alpha",
              "dst_uid": "issue:notes/mcp-test-alpha#7c136aa7"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 10308,
      "at": "2026-10-24T21:37:35.911Z",
      "verdict": "pass"
    },
    {
      "id": 40,
      "label": "beta own issues empty",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:mcp-test-beta"
      },
      "expected": "Zero rows and no presenter leak",
      "follow": "Create beta issue",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:mcp-test-beta\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.879371,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:mcp-test-beta"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.879371,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 12934,
      "at": "2026-10-24T21:37:38.537Z",
      "verdict": "pass"
    },
    {
      "id": 41,
      "label": "after exact inclusive",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "after": "10:30"
      },
      "expected": "Includes 10:30 talk",
      "follow": "After 10:31 excludes",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"10:30\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.470653,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "10:30",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.470653,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 15922,
      "at": "2026-10-24T21:37:41.525Z",
      "verdict": "pass"
    },
    {
      "id": 42,
      "label": "before exact exclusive",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "before": "10:30"
      },
      "expected": "Excludes 10:30 talk",
      "follow": "Before 10:31 includes",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"10:30\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.340009,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "10:30",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.340009,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 18279,
      "at": "2026-10-24T21:37:43.882Z",
      "verdict": "pass"
    },
    {
      "id": 43,
      "label": "after excludes",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "after": "10:31"
      },
      "expected": "Zero rows",
      "follow": "Invalid after validation",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"10:31\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.304403,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "10:31",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.304403,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 21258,
      "at": "2026-10-24T21:37:46.861Z",
      "verdict": "pass"
    },
    {
      "id": 44,
      "label": "before includes",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "before": "10:31"
      },
      "expected": "One row",
      "follow": "Compare omitted bounds",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"10:31\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.44605,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "10:31",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.44605,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 23563,
      "at": "2026-10-24T21:37:49.166Z",
      "verdict": "pass"
    },
    {
      "id": 45,
      "label": "invalid topic time and day",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2026-13-99",
        "after": "99:99"
      },
      "expected": "Validation error",
      "follow": "Inspect params handling",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.529489,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.529489,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 26114,
      "at": "2026-10-24T21:37:51.717Z",
      "verdict": "fail"
    },
    {
      "id": 46,
      "label": "learning second overlap",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-alpha",
        "learned": "mcp-test: second overlapping lesson.",
        "source": "mcp-test-overlap",
        "topics": [
          "mcp-auth",
          "oauth"
        ]
      },
      "expected": "Distinct newer uid; related talks match known topic",
      "follow": "Recall ordering",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\",\n  \"person_uid\": \"person:gh:mcp-test-alpha\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"mcp-auth\",\n    \"oauth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9",
        "person_uid": "person:gh:mcp-test-alpha",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "mcp-auth",
          "oauth"
        ]
      },
      "wall_ms": 3546,
      "at": "2026-10-24T21:38:10.409Z",
      "verdict": "pass"
    },
    {
      "id": 47,
      "label": "learning duplicate one",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-beta",
        "learned": "mcp-test: identical lesson.",
        "source": "mcp-test-duplicate",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Beta learning independent of alpha",
      "follow": "Repeat identical write",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:05362c1e-7200-494e-9185-f8eabf1bcd25\",\n  \"person_uid\": \"person:gh:mcp-test-beta\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"mcp-auth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:05362c1e-7200-494e-9185-f8eabf1bcd25",
        "person_uid": "person:gh:mcp-test-beta",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "mcp-auth"
        ]
      },
      "wall_ms": 2975,
      "at": "2026-10-24T21:38:13.386Z",
      "verdict": "pass"
    },
    {
      "id": 48,
      "label": "learning duplicate two",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-beta",
        "learned": "mcp-test: identical lesson.",
        "source": "mcp-test-duplicate",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Document duplicate behavior",
      "follow": "Recall beta two uids",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2\",\n  \"person_uid\": \"person:gh:mcp-test-beta\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"mcp-auth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2",
        "person_uid": "person:gh:mcp-test-beta",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "mcp-auth"
        ]
      },
      "wall_ms": 3488,
      "at": "2026-10-24T21:38:16.877Z",
      "verdict": "pass"
    },
    {
      "id": 49,
      "label": "learning unicode injection",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-alpha",
        "learned": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
        "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Store exact inert payload",
      "follow": "Recall byte-for-byte; baseline intact",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "tool call error: tool call rejected because the catalog changed after `codex_apps/patchwork.log_learning` was prepared"
          }
        ],
        "isError": true
      },
      "data": "tool call error: tool call rejected because the catalog changed after `codex_apps/patchwork.log_learning` was prepared",
      "wall_ms": 2458,
      "at": "2026-10-24T21:38:19.337Z",
      "verdict": "inconclusive"
    },
    {
      "id": 50,
      "label": "learning long",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-alpha",
        "learned": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "source": "mcp-test-long",
        "topics": [
          "oauth"
        ]
      },
      "expected": "Accept 2000 characters intact",
      "follow": "Recall length",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\",\n  \"person_uid\": \"person:gh:mcp-test-alpha\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"oauth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48",
        "person_uid": "person:gh:mcp-test-alpha",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "oauth"
        ]
      },
      "wall_ms": 4343,
      "at": "2026-10-24T21:38:23.683Z",
      "verdict": "pass"
    },
    {
      "id": 51,
      "label": "learning single word",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-alpha",
        "learned": "mcp-test-singleword",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Accept single word with omitted source",
      "follow": "Recall source default",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\",\n  \"person_uid\": \"person:gh:mcp-test-alpha\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"mcp-auth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9",
        "person_uid": "person:gh:mcp-test-alpha",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "mcp-auth"
        ]
      },
      "wall_ms": 4381,
      "at": "2026-10-24T21:38:28.067Z",
      "verdict": "pass"
    },
    {
      "id": 52,
      "label": "learning unknown slug write",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-validation",
        "learned": "mcp-test: invalid topic must reject",
        "topics": [
          "mcp-test-unknown"
        ]
      },
      "expected": "Reject before mutation",
      "follow": "Check 22 topics and no saved learning",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
        }
      },
      "wall_ms": 3710,
      "at": "2026-10-24T21:38:31.780Z",
      "verdict": "pass"
    },
    {
      "id": 53,
      "label": "issue unknown slug write",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-validation",
        "title": "mcp-test: unknown slug",
        "body": "mcp-test: reject unknown topic",
        "topics": [
          "mcp-test-unknown"
        ]
      },
      "expected": "Reject before mutation",
      "follow": "Check topic count",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
        }
      },
      "wall_ms": 3887,
      "at": "2026-10-24T21:38:35.669Z",
      "verdict": "pass"
    },
    {
      "id": 54,
      "label": "encounter unknown slug write",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-validation",
        "met_handle": "mcp-test-invalid-contact",
        "met_name": "mcp-test-invalid-contact",
        "note": "mcp-test: reject unknown topic",
        "topics": [
          "mcp-test-unknown"
        ]
      },
      "expected": "Reject before mutation",
      "follow": "Check graph baseline",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "topics must be non-empty slugs from list_topics; unknown: [\"mcp-test-unknown\"]"
        }
      },
      "wall_ms": 8282,
      "at": "2026-10-24T21:38:43.953Z",
      "verdict": "pass"
    },
    {
      "id": 55,
      "label": "beta issue isolation seed",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-beta",
        "title": "mcp-test: beta-only auth issue",
        "body": "mcp-test: beta should only see its own issues",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "New beta issue only",
      "follow": "Alpha memory must not show beta contact",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-beta#f6cc6fcd\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-beta\",\n  \"repo_uid\": \"repo:notes/mcp-test-beta\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-beta#f6cc6fcd",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-beta",
        "repo_uid": "repo:notes/mcp-test-beta",
        "state": "CLASSIFIED"
      },
      "wall_ms": 3010,
      "at": "2026-10-24T21:39:06.012Z",
      "verdict": "pass"
    },
    {
      "id": 56,
      "label": "beta encounter isolation seed",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-beta",
        "met_name": "mcp-test-beta-private",
        "met_handle": "mcp-test-beta-private",
        "note": "mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024",
        "place": "mcp-test-beta-place",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Beta private encounter",
      "follow": "Alpha memory must exclude beta note",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"contact_uid\": \"person:gh:mcp-test-beta-private\",\n  \"encounter_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n  \"state\": \"PARSED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "contact_uid": "person:gh:mcp-test-beta-private",
        "encounter_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
        "state": "PARSED"
      },
      "wall_ms": 2955,
      "at": "2026-10-24T21:39:08.970Z",
      "verdict": "pass"
    },
    {
      "id": 57,
      "label": "learning unicode injection retry",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-alpha",
        "learned": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
        "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Inert exact string",
      "follow": "Recall equality and presenter health",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learning_uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\",\n  \"person_uid\": \"person:gh:mcp-test-alpha\",\n  \"related_talks\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\"\n    }\n  ],\n  \"state\": \"LOGGED\",\n  \"topics\": [\n    \"mcp-auth\"\n  ]\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learning_uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f",
        "person_uid": "person:gh:mcp-test-alpha",
        "related_talks": [
          {
            "day": "2026-10-29",
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"
          }
        ],
        "state": "LOGGED",
        "topics": [
          "mcp-auth"
        ]
      },
      "wall_ms": 3024,
      "at": "2026-10-24T21:39:11.996Z",
      "verdict": "pass"
    },
    {
      "id": 58,
      "label": "empty learning topics",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-validation",
        "learned": "mcp-test-empty-topic",
        "topics": []
      },
      "expected": "Clean error before mutation",
      "follow": "Validation recall empty",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "topics must be non-empty slugs from list_topics; unknown: []"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "topics must be non-empty slugs from list_topics; unknown: []"
        }
      },
      "wall_ms": 3174,
      "at": "2026-10-24T21:39:15.173Z",
      "verdict": "pass"
    },
    {
      "id": 59,
      "label": "empty issue topics",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-validation",
        "title": "mcp-test-empty-topic",
        "body": "mcp-test-empty-topic",
        "topics": []
      },
      "expected": "Clean error before mutation",
      "follow": "Validation recall empty",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "topics must be non-empty slugs from list_topics; unknown: []"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "topics must be non-empty slugs from list_topics; unknown: []"
        }
      },
      "wall_ms": 2689,
      "at": "2026-10-24T21:39:17.865Z",
      "verdict": "pass"
    },
    {
      "id": 60,
      "label": "empty encounter topics",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-validation",
        "met_name": "mcp-test-validation",
        "note": "mcp-test-empty-topic",
        "topics": []
      },
      "expected": "Clean error before mutation",
      "follow": "Topic count stays 22",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "topics must be non-empty slugs from list_topics; unknown: []"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: topics must be non-empty slugs from list_topics; unknown: []",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "topics must be non-empty slugs from list_topics; unknown: []"
        }
      },
      "wall_ms": 3168,
      "at": "2026-10-24T21:39:21.035Z",
      "verdict": "pass"
    },
    {
      "id": 61,
      "label": "alpha recall after beta encounter",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-alpha#7c136aa7"
      },
      "expected": "Must exclude BETA_ONLY private note",
      "follow": "Reverse beta recall",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-alpha#7c136aa7\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.478114,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-beta-private\",\n      \"contact_uid\": \"person:gh:mcp-test-beta-private\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:39:08.779009+00:00\",\n      \"note\": \"mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024\",\n      \"place\": \"mcp-test-beta-place\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"contact\": \"mcp-test-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:37:03.099287+00:00\",\n      \"note\": \"mcp-test: shared MCP auth experience.\",\n      \"place\": \"mcp-test-hallway\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: MCP auth bridge timing\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-beta-private\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-beta-private\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"person:gh:mcp-test-beta-private\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"person:gh:mcp-test-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-alpha#7c136aa7"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 0.478114,
        "rows": [
          {
            "contact": "mcp-test-beta-private",
            "contact_uid": "person:gh:mcp-test-beta-private",
            "hops": 3,
            "met_at": "2026-10-24T21:39:08.779009+00:00",
            "note": "mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024",
            "place": "mcp-test-beta-place",
            "topic": "MCP Auth"
          },
          {
            "contact": "mcp-test-contact",
            "contact_uid": "person:gh:mcp-test-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:37:03.099287+00:00",
            "note": "mcp-test: shared MCP auth experience.",
            "place": "mcp-test-hallway",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "label": "Issue",
              "name": "mcp-test: MCP auth bridge timing",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-beta-private",
              "label": "Person",
              "name": "mcp-test-beta-private",
              "state": ""
            },
            {
              "uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-contact",
              "label": "Person",
              "name": "mcp-test-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "person:gh:mcp-test-beta-private"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "person:gh:mcp-test-contact"
            }
          ]
        }
      },
      "wall_ms": 2479,
      "at": "2026-10-24T21:39:23.516Z",
      "verdict": "fail"
    },
    {
      "id": 62,
      "label": "beta learning ownership",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-beta"
      },
      "expected": "Only two beta duplicate records, newest first",
      "follow": "Alpha excludes beta uids",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:38:16.715559+00:00\",\n      \"source\": \"mcp-test-duplicate\",\n      \"text\": \"mcp-test: identical lesson.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:13.219946+00:00\",\n      \"source\": \"mcp-test-duplicate\",\n      \"text\": \"mcp-test: identical lesson.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:05362c1e-7200-494e-9185-f8eabf1bcd25\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-beta\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:38:16.715559+00:00",
            "source": "mcp-test-duplicate",
            "text": "mcp-test: identical lesson.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2"
          },
          {
            "learned_at": "2026-10-24T21:38:13.219946+00:00",
            "source": "mcp-test-duplicate",
            "text": "mcp-test: identical lesson.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:05362c1e-7200-494e-9185-f8eabf1bcd25"
          }
        ],
        "person_uid": "person:gh:mcp-test-beta"
      },
      "wall_ms": 5098,
      "at": "2026-10-24T21:39:26.135Z",
      "verdict": "pass"
    },
    {
      "id": 63,
      "label": "beta recall reverse",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-beta#f6cc6fcd"
      },
      "expected": "Must exclude alpha encounter",
      "follow": "Compare path LOGGED constraints",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-beta#f6cc6fcd\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.444888,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-beta-private\",\n      \"contact_uid\": \"person:gh:mcp-test-beta-private\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:39:08.779009+00:00\",\n      \"note\": \"mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024\",\n      \"place\": \"mcp-test-beta-place\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"contact\": \"mcp-test-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:37:03.099287+00:00\",\n      \"note\": \"mcp-test: shared MCP auth experience.\",\n      \"place\": \"mcp-test-hallway\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-beta#f6cc6fcd\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: beta-only auth issue\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-beta-private\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-beta-private\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-beta#f6cc6fcd\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"person:gh:mcp-test-beta-private\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"person:gh:mcp-test-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-beta#f6cc6fcd"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 0.444888,
        "rows": [
          {
            "contact": "mcp-test-beta-private",
            "contact_uid": "person:gh:mcp-test-beta-private",
            "hops": 3,
            "met_at": "2026-10-24T21:39:08.779009+00:00",
            "note": "mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024",
            "place": "mcp-test-beta-place",
            "topic": "MCP Auth"
          },
          {
            "contact": "mcp-test-contact",
            "contact_uid": "person:gh:mcp-test-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:37:03.099287+00:00",
            "note": "mcp-test: shared MCP auth experience.",
            "place": "mcp-test-hallway",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-beta#f6cc6fcd",
              "label": "Issue",
              "name": "mcp-test: beta-only auth issue",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-beta-private",
              "label": "Person",
              "name": "mcp-test-beta-private",
              "state": ""
            },
            {
              "uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-contact",
              "label": "Person",
              "name": "mcp-test-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-beta#f6cc6fcd",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "person:gh:mcp-test-beta-private"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "person:gh:mcp-test-contact"
            }
          ]
        }
      },
      "wall_ms": 8792,
      "at": "2026-10-24T21:39:29.829Z",
      "verdict": "fail"
    },
    {
      "id": 64,
      "label": "alpha all learning roundtrip",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha"
      },
      "expected": "All alpha uids exact and newest first",
      "follow": "Filter by oauth",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [\n        \"oauth\"\n      ],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [\n        \"mcp-auth\",\n        \"oauth\"\n      ],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [
              "oauth"
            ],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [
              "mcp-auth",
              "oauth"
            ],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 11858,
      "at": "2026-10-24T21:39:32.895Z",
      "verdict": "pass"
    },
    {
      "id": 65,
      "label": "coverage agentic-security",
      "tool": "talks_by_topic",
      "args": {
        "topics": "agentic-security"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"agentic-security\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 1.192697,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Jeff An\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"Agentic Security\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"Agentic Security\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:agentic-security\",\n        \"label\": \"Topic\",\n        \"name\": \"Agentic Security\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-07\",\n        \"label\": \"Talk\",\n        \"name\": \"Beyond pass or fail: How agents verify AI-generated code\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jeff-an\",\n        \"label\": \"Person\",\n        \"name\": \"Jeff An\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jeff-an\",\n        \"dst_uid\": \"talk:U26-07\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "agentic-security"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 1.192697,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Jeff An",
            "start_time": "09:00",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "Agentic Security"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "Agentic Security"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:agentic-security",
              "label": "Topic",
              "name": "Agentic Security",
              "state": ""
            },
            {
              "uid": "talk:U26-07",
              "label": "Talk",
              "name": "Beyond pass or fail: How agents verify AI-generated code",
              "state": ""
            },
            {
              "uid": "person:name:jeff-an",
              "label": "Person",
              "name": "Jeff An",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jeff-an",
              "dst_uid": "talk:U26-07"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 2545,
      "at": "2026-10-24T21:39:43.637Z",
      "verdict": "pass"
    },
    {
      "id": 66,
      "label": "coverage ai-context",
      "tool": "talks_by_topic",
      "args": {
        "topics": "ai-context"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"ai-context\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.430364,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Rohan Varma\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"AI Context\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Kate Catlin\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"AI Context\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Cara Phillips\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"AI Context\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Christopher Harrison\",\n      \"start_time\": \"13:00\",\n      \"talk\": \"Treat your AI context as infrastructure\",\n      \"topic\": \"AI Context\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:ai-context\",\n        \"label\": \"Topic\",\n        \"name\": \"AI Context\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-06\",\n        \"label\": \"Talk\",\n        \"name\": \"The view from the labs: What's next for AI-assisted development\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:rohan-varma\",\n        \"label\": \"Person\",\n        \"name\": \"Rohan Varma\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:kate-catlin\",\n        \"label\": \"Person\",\n        \"name\": \"Kate Catlin\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:cara-phillips\",\n        \"label\": \"Person\",\n        \"name\": \"Cara Phillips\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-10\",\n        \"label\": \"Talk\",\n        \"name\": \"Treat your AI context as infrastructure\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:christopher-harrison\",\n        \"label\": \"Person\",\n        \"name\": \"Christopher Harrison\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-06\",\n        \"dst_uid\": \"topic:ai-context\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:rohan-varma\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:kate-catlin\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:cara-phillips\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-10\",\n        \"dst_uid\": \"topic:ai-context\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:christopher-harrison\",\n        \"dst_uid\": \"talk:U26-10\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "ai-context"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.430364,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Rohan Varma",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "AI Context"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Kate Catlin",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "AI Context"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Cara Phillips",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "AI Context"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Christopher Harrison",
            "start_time": "13:00",
            "talk": "Treat your AI context as infrastructure",
            "topic": "AI Context"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:ai-context",
              "label": "Topic",
              "name": "AI Context",
              "state": ""
            },
            {
              "uid": "talk:U26-06",
              "label": "Talk",
              "name": "The view from the labs: What's next for AI-assisted development",
              "state": ""
            },
            {
              "uid": "person:name:rohan-varma",
              "label": "Person",
              "name": "Rohan Varma",
              "state": ""
            },
            {
              "uid": "person:name:kate-catlin",
              "label": "Person",
              "name": "Kate Catlin",
              "state": ""
            },
            {
              "uid": "person:name:cara-phillips",
              "label": "Person",
              "name": "Cara Phillips",
              "state": ""
            },
            {
              "uid": "talk:U26-10",
              "label": "Talk",
              "name": "Treat your AI context as infrastructure",
              "state": ""
            },
            {
              "uid": "person:name:christopher-harrison",
              "label": "Person",
              "name": "Christopher Harrison",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-06",
              "dst_uid": "topic:ai-context"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:rohan-varma",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:kate-catlin",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:cara-phillips",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-10",
              "dst_uid": "topic:ai-context"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:christopher-harrison",
              "dst_uid": "talk:U26-10"
            }
          ]
        }
      },
      "wall_ms": 5584,
      "at": "2026-10-24T21:39:46.676Z",
      "verdict": "pass"
    },
    {
      "id": 67,
      "label": "coverage agent-memory",
      "tool": "talks_by_topic",
      "args": {
        "topics": "agent-memory"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"agent-memory\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.498678,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Christopher Harrison\",\n      \"start_time\": \"13:00\",\n      \"talk\": \"Treat your AI context as infrastructure\",\n      \"topic\": \"Agent Memory\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:agent-memory\",\n        \"label\": \"Topic\",\n        \"name\": \"Agent Memory\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-10\",\n        \"label\": \"Talk\",\n        \"name\": \"Treat your AI context as infrastructure\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:christopher-harrison\",\n        \"label\": \"Person\",\n        \"name\": \"Christopher Harrison\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-10\",\n        \"dst_uid\": \"topic:agent-memory\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:christopher-harrison\",\n        \"dst_uid\": \"talk:U26-10\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "agent-memory"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.498678,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Christopher Harrison",
            "start_time": "13:00",
            "talk": "Treat your AI context as infrastructure",
            "topic": "Agent Memory"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:agent-memory",
              "label": "Topic",
              "name": "Agent Memory",
              "state": ""
            },
            {
              "uid": "talk:U26-10",
              "label": "Talk",
              "name": "Treat your AI context as infrastructure",
              "state": ""
            },
            {
              "uid": "person:name:christopher-harrison",
              "label": "Person",
              "name": "Christopher Harrison",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-10",
              "dst_uid": "topic:agent-memory"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:christopher-harrison",
              "dst_uid": "talk:U26-10"
            }
          ]
        }
      },
      "wall_ms": 8411,
      "at": "2026-10-24T21:39:49.503Z",
      "verdict": "pass"
    },
    {
      "id": 68,
      "label": "coverage ai-assisted-testing",
      "tool": "talks_by_topic",
      "args": {
        "topics": "ai-assisted-testing"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"ai-assisted-testing\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.463197,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Jeff An\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"AI-Assisted Testing\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:ai-assisted-testing\",\n        \"label\": \"Topic\",\n        \"name\": \"AI-Assisted Testing\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-07\",\n        \"label\": \"Talk\",\n        \"name\": \"Beyond pass or fail: How agents verify AI-generated code\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jeff-an\",\n        \"label\": \"Person\",\n        \"name\": \"Jeff An\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:ai-assisted-testing\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jeff-an\",\n        \"dst_uid\": \"talk:U26-07\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "ai-assisted-testing"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.463197,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Jeff An",
            "start_time": "09:00",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "AI-Assisted Testing"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:ai-assisted-testing",
              "label": "Topic",
              "name": "AI-Assisted Testing",
              "state": ""
            },
            {
              "uid": "talk:U26-07",
              "label": "Talk",
              "name": "Beyond pass or fail: How agents verify AI-generated code",
              "state": ""
            },
            {
              "uid": "person:name:jeff-an",
              "label": "Person",
              "name": "Jeff An",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:ai-assisted-testing"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jeff-an",
              "dst_uid": "talk:U26-07"
            }
          ]
        }
      },
      "wall_ms": 11923,
      "at": "2026-10-24T21:39:53.015Z",
      "verdict": "pass"
    },
    {
      "id": 69,
      "label": "coverage agentic-workflows",
      "tool": "talks_by_topic",
      "args": {
        "topics": "agentic-workflows"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"agentic-workflows\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.467757,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Cara Phillips\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"Agentic Workflows\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Kate Catlin\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"Agentic Workflows\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Rohan Varma\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"Agentic Workflows\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:agentic-workflows\",\n        \"label\": \"Topic\",\n        \"name\": \"Agentic Workflows\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-06\",\n        \"label\": \"Talk\",\n        \"name\": \"The view from the labs: What's next for AI-assisted development\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:cara-phillips\",\n        \"label\": \"Person\",\n        \"name\": \"Cara Phillips\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:kate-catlin\",\n        \"label\": \"Person\",\n        \"name\": \"Kate Catlin\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:rohan-varma\",\n        \"label\": \"Person\",\n        \"name\": \"Rohan Varma\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-06\",\n        \"dst_uid\": \"topic:agentic-workflows\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:cara-phillips\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:kate-catlin\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:rohan-varma\",\n        \"dst_uid\": \"talk:U26-06\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "agentic-workflows"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.467757,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Cara Phillips",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "Agentic Workflows"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Kate Catlin",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "Agentic Workflows"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Rohan Varma",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "Agentic Workflows"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:agentic-workflows",
              "label": "Topic",
              "name": "Agentic Workflows",
              "state": ""
            },
            {
              "uid": "talk:U26-06",
              "label": "Talk",
              "name": "The view from the labs: What's next for AI-assisted development",
              "state": ""
            },
            {
              "uid": "person:name:cara-phillips",
              "label": "Person",
              "name": "Cara Phillips",
              "state": ""
            },
            {
              "uid": "person:name:kate-catlin",
              "label": "Person",
              "name": "Kate Catlin",
              "state": ""
            },
            {
              "uid": "person:name:rohan-varma",
              "label": "Person",
              "name": "Rohan Varma",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-06",
              "dst_uid": "topic:agentic-workflows"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:cara-phillips",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:kate-catlin",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:rohan-varma",
              "dst_uid": "talk:U26-06"
            }
          ]
        }
      },
      "wall_ms": 14331,
      "at": "2026-10-24T21:39:55.423Z",
      "verdict": "pass"
    },
    {
      "id": 70,
      "label": "coverage ai-operations",
      "tool": "talks_by_topic",
      "args": {
        "topics": "ai-operations"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"ai-operations\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.415706,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Achin Gupta\",\n      \"start_time\": \"09:30\",\n      \"talk\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n      \"topic\": \"AI Operations\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Divya Mahajan\",\n      \"start_time\": \"09:30\",\n      \"talk\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n      \"topic\": \"AI Operations\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:ai-operations\",\n        \"label\": \"Topic\",\n        \"name\": \"AI Operations\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-08\",\n        \"label\": \"Talk\",\n        \"name\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:achin-gupta\",\n        \"label\": \"Person\",\n        \"name\": \"Achin Gupta\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:divya-mahajan\",\n        \"label\": \"Person\",\n        \"name\": \"Divya Mahajan\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-08\",\n        \"dst_uid\": \"topic:ai-operations\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:achin-gupta\",\n        \"dst_uid\": \"talk:U26-08\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:divya-mahajan\",\n        \"dst_uid\": \"talk:U26-08\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "ai-operations"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.415706,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Achin Gupta",
            "start_time": "09:30",
            "talk": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
            "topic": "AI Operations"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Divya Mahajan",
            "start_time": "09:30",
            "talk": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
            "topic": "AI Operations"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:ai-operations",
              "label": "Topic",
              "name": "AI Operations",
              "state": ""
            },
            {
              "uid": "talk:U26-08",
              "label": "Talk",
              "name": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
              "state": ""
            },
            {
              "uid": "person:name:achin-gupta",
              "label": "Person",
              "name": "Achin Gupta",
              "state": ""
            },
            {
              "uid": "person:name:divya-mahajan",
              "label": "Person",
              "name": "Divya Mahajan",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-08",
              "dst_uid": "topic:ai-operations"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:achin-gupta",
              "dst_uid": "talk:U26-08"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:divya-mahajan",
              "dst_uid": "talk:U26-08"
            }
          ]
        }
      },
      "wall_ms": 17138,
      "at": "2026-10-24T21:39:58.230Z",
      "verdict": "pass"
    },
    {
      "id": 71,
      "label": "coverage cicd",
      "tool": "talks_by_topic",
      "args": {
        "topics": "cicd"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"cicd\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.534091,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "cicd"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.534091,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 20389,
      "at": "2026-10-24T21:40:01.481Z",
      "verdict": "pass"
    },
    {
      "id": 72,
      "label": "coverage community-governance",
      "tool": "talks_by_topic",
      "args": {
        "topics": "community-governance"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"community-governance\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.46919,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"Community Governance\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:community-governance\",\n        \"label\": \"Topic\",\n        \"name\": \"Community Governance\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:community-governance\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "community-governance"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.46919,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "Community Governance"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:community-governance",
              "label": "Topic",
              "name": "Community Governance",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:community-governance"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            }
          ]
        }
      },
      "wall_ms": 23166,
      "at": "2026-10-24T21:40:04.258Z",
      "verdict": "pass"
    },
    {
      "id": 73,
      "label": "coverage copilot",
      "tool": "talks_by_topic",
      "args": {
        "topics": "copilot"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"copilot\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.462428,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Jared Hatfield\",\n      \"start_time\": \"11:30\",\n      \"talk\": \"Building AI fluency at UPS\",\n      \"topic\": \"Copilot\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:copilot\",\n        \"label\": \"Topic\",\n        \"name\": \"Copilot\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-02\",\n        \"label\": \"Talk\",\n        \"name\": \"Building AI fluency at UPS\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jared-hatfield\",\n        \"label\": \"Person\",\n        \"name\": \"Jared Hatfield\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-02\",\n        \"dst_uid\": \"topic:copilot\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jared-hatfield\",\n        \"dst_uid\": \"talk:U26-02\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "copilot"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.462428,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Jared Hatfield",
            "start_time": "11:30",
            "talk": "Building AI fluency at UPS",
            "topic": "Copilot"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:copilot",
              "label": "Topic",
              "name": "Copilot",
              "state": ""
            },
            {
              "uid": "talk:U26-02",
              "label": "Talk",
              "name": "Building AI fluency at UPS",
              "state": ""
            },
            {
              "uid": "person:name:jared-hatfield",
              "label": "Person",
              "name": "Jared Hatfield",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-02",
              "dst_uid": "topic:copilot"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jared-hatfield",
              "dst_uid": "talk:U26-02"
            }
          ]
        }
      },
      "wall_ms": 25796,
      "at": "2026-10-24T21:40:06.888Z",
      "verdict": "pass"
    },
    {
      "id": 74,
      "label": "coverage dependencies",
      "tool": "talks_by_topic",
      "args": {
        "topics": "dependencies"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"dependencies\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.454122,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Karen Li\",\n      \"start_time\": \"16:30\",\n      \"talk\": \"What happens behind the scenes when you run npm install\",\n      \"topic\": \"Dependencies\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Leo Balter\",\n      \"start_time\": \"16:30\",\n      \"talk\": \"What happens behind the scenes when you run npm install\",\n      \"topic\": \"Dependencies\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:dependencies\",\n        \"label\": \"Topic\",\n        \"name\": \"Dependencies\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-05\",\n        \"label\": \"Talk\",\n        \"name\": \"What happens behind the scenes when you run npm install\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:karen-li\",\n        \"label\": \"Person\",\n        \"name\": \"Karen Li\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:leo-balter\",\n        \"label\": \"Person\",\n        \"name\": \"Leo Balter\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-05\",\n        \"dst_uid\": \"topic:dependencies\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:karen-li\",\n        \"dst_uid\": \"talk:U26-05\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:leo-balter\",\n        \"dst_uid\": \"talk:U26-05\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "dependencies"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.454122,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Karen Li",
            "start_time": "16:30",
            "talk": "What happens behind the scenes when you run npm install",
            "topic": "Dependencies"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Leo Balter",
            "start_time": "16:30",
            "talk": "What happens behind the scenes when you run npm install",
            "topic": "Dependencies"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:dependencies",
              "label": "Topic",
              "name": "Dependencies",
              "state": ""
            },
            {
              "uid": "talk:U26-05",
              "label": "Talk",
              "name": "What happens behind the scenes when you run npm install",
              "state": ""
            },
            {
              "uid": "person:name:karen-li",
              "label": "Person",
              "name": "Karen Li",
              "state": ""
            },
            {
              "uid": "person:name:leo-balter",
              "label": "Person",
              "name": "Leo Balter",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-05",
              "dst_uid": "topic:dependencies"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:karen-li",
              "dst_uid": "talk:U26-05"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:leo-balter",
              "dst_uid": "talk:U26-05"
            }
          ]
        }
      },
      "wall_ms": 28329,
      "at": "2026-10-24T21:40:09.421Z",
      "verdict": "pass"
    },
    {
      "id": 75,
      "label": "coverage deterministic-verification",
      "tool": "talks_by_topic",
      "args": {
        "topics": "deterministic-verification"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / learning related talks",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"deterministic-verification\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.494699,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Jeff An\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"Deterministic Verification\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Achin Gupta\",\n      \"start_time\": \"09:30\",\n      \"talk\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n      \"topic\": \"Deterministic Verification\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Divya Mahajan\",\n      \"start_time\": \"09:30\",\n      \"talk\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n      \"topic\": \"Deterministic Verification\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:deterministic-verification\",\n        \"label\": \"Topic\",\n        \"name\": \"Deterministic Verification\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-07\",\n        \"label\": \"Talk\",\n        \"name\": \"Beyond pass or fail: How agents verify AI-generated code\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jeff-an\",\n        \"label\": \"Person\",\n        \"name\": \"Jeff An\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-08\",\n        \"label\": \"Talk\",\n        \"name\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:achin-gupta\",\n        \"label\": \"Person\",\n        \"name\": \"Achin Gupta\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:divya-mahajan\",\n        \"label\": \"Person\",\n        \"name\": \"Divya Mahajan\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:deterministic-verification\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jeff-an\",\n        \"dst_uid\": \"talk:U26-07\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-08\",\n        \"dst_uid\": \"topic:deterministic-verification\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:achin-gupta\",\n        \"dst_uid\": \"talk:U26-08\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:divya-mahajan\",\n        \"dst_uid\": \"talk:U26-08\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "deterministic-verification"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.494699,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Jeff An",
            "start_time": "09:00",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "Deterministic Verification"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Achin Gupta",
            "start_time": "09:30",
            "talk": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
            "topic": "Deterministic Verification"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Divya Mahajan",
            "start_time": "09:30",
            "talk": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
            "topic": "Deterministic Verification"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:deterministic-verification",
              "label": "Topic",
              "name": "Deterministic Verification",
              "state": ""
            },
            {
              "uid": "talk:U26-07",
              "label": "Talk",
              "name": "Beyond pass or fail: How agents verify AI-generated code",
              "state": ""
            },
            {
              "uid": "person:name:jeff-an",
              "label": "Person",
              "name": "Jeff An",
              "state": ""
            },
            {
              "uid": "talk:U26-08",
              "label": "Talk",
              "name": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
              "state": ""
            },
            {
              "uid": "person:name:achin-gupta",
              "label": "Person",
              "name": "Achin Gupta",
              "state": ""
            },
            {
              "uid": "person:name:divya-mahajan",
              "label": "Person",
              "name": "Divya Mahajan",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:deterministic-verification"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jeff-an",
              "dst_uid": "talk:U26-07"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-08",
              "dst_uid": "topic:deterministic-verification"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:achin-gupta",
              "dst_uid": "talk:U26-08"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:divya-mahajan",
              "dst_uid": "talk:U26-08"
            }
          ]
        }
      },
      "wall_ms": 31018,
      "at": "2026-10-24T21:40:12.110Z",
      "verdict": "pass"
    },
    {
      "id": 76,
      "label": "coverage enterprise-ai-adoption",
      "tool": "talks_by_topic",
      "args": {
        "topics": "enterprise-ai-adoption"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"enterprise-ai-adoption\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.509774,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Jared Hatfield\",\n      \"start_time\": \"11:30\",\n      \"talk\": \"Building AI fluency at UPS\",\n      \"topic\": \"Enterprise AI Adoption\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:enterprise-ai-adoption\",\n        \"label\": \"Topic\",\n        \"name\": \"Enterprise AI Adoption\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-02\",\n        \"label\": \"Talk\",\n        \"name\": \"Building AI fluency at UPS\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jared-hatfield\",\n        \"label\": \"Person\",\n        \"name\": \"Jared Hatfield\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-02\",\n        \"dst_uid\": \"topic:enterprise-ai-adoption\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jared-hatfield\",\n        \"dst_uid\": \"talk:U26-02\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "enterprise-ai-adoption"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.509774,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Jared Hatfield",
            "start_time": "11:30",
            "talk": "Building AI fluency at UPS",
            "topic": "Enterprise AI Adoption"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:enterprise-ai-adoption",
              "label": "Topic",
              "name": "Enterprise AI Adoption",
              "state": ""
            },
            {
              "uid": "talk:U26-02",
              "label": "Talk",
              "name": "Building AI fluency at UPS",
              "state": ""
            },
            {
              "uid": "person:name:jared-hatfield",
              "label": "Person",
              "name": "Jared Hatfield",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-02",
              "dst_uid": "topic:enterprise-ai-adoption"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jared-hatfield",
              "dst_uid": "talk:U26-02"
            }
          ]
        }
      },
      "wall_ms": 2457,
      "at": "2026-10-24T21:40:22.604Z",
      "verdict": "pass"
    },
    {
      "id": 77,
      "label": "coverage frontier-models",
      "tool": "talks_by_topic",
      "args": {
        "topics": "frontier-models"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"frontier-models\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.408756,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Cara Phillips\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"Frontier Models\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Kate Catlin\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"Frontier Models\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Main Stage (Festival Pavilion, Fort Mason)\",\n      \"speaker\": \"Rohan Varma\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"Frontier Models\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:frontier-models\",\n        \"label\": \"Topic\",\n        \"name\": \"Frontier Models\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-06\",\n        \"label\": \"Talk\",\n        \"name\": \"The view from the labs: What's next for AI-assisted development\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:cara-phillips\",\n        \"label\": \"Person\",\n        \"name\": \"Cara Phillips\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:kate-catlin\",\n        \"label\": \"Person\",\n        \"name\": \"Kate Catlin\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:rohan-varma\",\n        \"label\": \"Person\",\n        \"name\": \"Rohan Varma\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-06\",\n        \"dst_uid\": \"topic:frontier-models\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:cara-phillips\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:kate-catlin\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:rohan-varma\",\n        \"dst_uid\": \"talk:U26-06\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "frontier-models"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.408756,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Cara Phillips",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "Frontier Models"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Kate Catlin",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "Frontier Models"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Main Stage (Festival Pavilion, Fort Mason)",
            "speaker": "Rohan Varma",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "Frontier Models"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:frontier-models",
              "label": "Topic",
              "name": "Frontier Models",
              "state": ""
            },
            {
              "uid": "talk:U26-06",
              "label": "Talk",
              "name": "The view from the labs: What's next for AI-assisted development",
              "state": ""
            },
            {
              "uid": "person:name:cara-phillips",
              "label": "Person",
              "name": "Cara Phillips",
              "state": ""
            },
            {
              "uid": "person:name:kate-catlin",
              "label": "Person",
              "name": "Kate Catlin",
              "state": ""
            },
            {
              "uid": "person:name:rohan-varma",
              "label": "Person",
              "name": "Rohan Varma",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-06",
              "dst_uid": "topic:frontier-models"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:cara-phillips",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:kate-catlin",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:rohan-varma",
              "dst_uid": "talk:U26-06"
            }
          ]
        }
      },
      "wall_ms": 5214,
      "at": "2026-10-24T21:40:25.361Z",
      "verdict": "pass"
    },
    {
      "id": 78,
      "label": "coverage github-actions",
      "tool": "talks_by_topic",
      "args": {
        "topics": "github-actions"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"github-actions\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.411459,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "github-actions"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.411459,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 8497,
      "at": "2026-10-24T21:40:28.644Z",
      "verdict": "pass"
    },
    {
      "id": 79,
      "label": "coverage developer-experience",
      "tool": "talks_by_topic",
      "args": {
        "topics": "developer-experience"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"developer-experience\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.483338,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Jared Hatfield\",\n      \"start_time\": \"11:30\",\n      \"talk\": \"Building AI fluency at UPS\",\n      \"topic\": \"Developer Experience\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:developer-experience\",\n        \"label\": \"Topic\",\n        \"name\": \"Developer Experience\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-02\",\n        \"label\": \"Talk\",\n        \"name\": \"Building AI fluency at UPS\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jared-hatfield\",\n        \"label\": \"Person\",\n        \"name\": \"Jared Hatfield\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-02\",\n        \"dst_uid\": \"topic:developer-experience\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jared-hatfield\",\n        \"dst_uid\": \"talk:U26-02\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "developer-experience"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.483338,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Jared Hatfield",
            "start_time": "11:30",
            "talk": "Building AI fluency at UPS",
            "topic": "Developer Experience"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:developer-experience",
              "label": "Topic",
              "name": "Developer Experience",
              "state": ""
            },
            {
              "uid": "talk:U26-02",
              "label": "Talk",
              "name": "Building AI fluency at UPS",
              "state": ""
            },
            {
              "uid": "person:name:jared-hatfield",
              "label": "Person",
              "name": "Jared Hatfield",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-02",
              "dst_uid": "topic:developer-experience"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jared-hatfield",
              "dst_uid": "talk:U26-02"
            }
          ]
        }
      },
      "wall_ms": 11141,
      "at": "2026-10-24T21:40:31.288Z",
      "verdict": "pass"
    },
    {
      "id": 80,
      "label": "coverage oauth",
      "tool": "talks_by_topic",
      "args": {
        "topics": "oauth"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"oauth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.581197,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"OAuth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:oauth\",\n        \"label\": \"Topic\",\n        \"name\": \"OAuth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:oauth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "oauth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.581197,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "OAuth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:oauth",
              "label": "Topic",
              "name": "OAuth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:oauth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 14233,
      "at": "2026-10-24T21:40:34.380Z",
      "verdict": "pass"
    },
    {
      "id": 81,
      "label": "coverage mcp-auth",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.456854,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.456854,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 17011,
      "at": "2026-10-24T21:40:37.158Z",
      "verdict": "pass"
    },
    {
      "id": 82,
      "label": "coverage open-source",
      "tool": "talks_by_topic",
      "args": {
        "topics": "open-source"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"open-source\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.388435,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"Open Source\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:open-source\",\n        \"label\": \"Topic\",\n        \"name\": \"Open Source\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:open-source\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "open-source"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.388435,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "Open Source"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:open-source",
              "label": "Topic",
              "name": "Open Source",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:open-source"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            }
          ]
        }
      },
      "wall_ms": 19460,
      "at": "2026-10-24T21:40:39.607Z",
      "verdict": "pass"
    },
    {
      "id": 83,
      "label": "coverage package-management",
      "tool": "talks_by_topic",
      "args": {
        "topics": "package-management"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"package-management\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.406419,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Karen Li\",\n      \"start_time\": \"16:30\",\n      \"talk\": \"What happens behind the scenes when you run npm install\",\n      \"topic\": \"Package Management\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Leo Balter\",\n      \"start_time\": \"16:30\",\n      \"talk\": \"What happens behind the scenes when you run npm install\",\n      \"topic\": \"Package Management\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:package-management\",\n        \"label\": \"Topic\",\n        \"name\": \"Package Management\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-05\",\n        \"label\": \"Talk\",\n        \"name\": \"What happens behind the scenes when you run npm install\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:karen-li\",\n        \"label\": \"Person\",\n        \"name\": \"Karen Li\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:leo-balter\",\n        \"label\": \"Person\",\n        \"name\": \"Leo Balter\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-05\",\n        \"dst_uid\": \"topic:package-management\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:karen-li\",\n        \"dst_uid\": \"talk:U26-05\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:leo-balter\",\n        \"dst_uid\": \"talk:U26-05\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "package-management"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.406419,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Karen Li",
            "start_time": "16:30",
            "talk": "What happens behind the scenes when you run npm install",
            "topic": "Package Management"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Leo Balter",
            "start_time": "16:30",
            "talk": "What happens behind the scenes when you run npm install",
            "topic": "Package Management"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:package-management",
              "label": "Topic",
              "name": "Package Management",
              "state": ""
            },
            {
              "uid": "talk:U26-05",
              "label": "Talk",
              "name": "What happens behind the scenes when you run npm install",
              "state": ""
            },
            {
              "uid": "person:name:karen-li",
              "label": "Person",
              "name": "Karen Li",
              "state": ""
            },
            {
              "uid": "person:name:leo-balter",
              "label": "Person",
              "name": "Leo Balter",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-05",
              "dst_uid": "topic:package-management"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:karen-li",
              "dst_uid": "talk:U26-05"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:leo-balter",
              "dst_uid": "talk:U26-05"
            }
          ]
        }
      },
      "wall_ms": 22733,
      "at": "2026-10-24T21:40:42.880Z",
      "verdict": "pass"
    },
    {
      "id": 84,
      "label": "coverage platform-engineering",
      "tool": "talks_by_topic",
      "args": {
        "topics": "platform-engineering"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"platform-engineering\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.413643,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Christopher Harrison\",\n      \"start_time\": \"13:00\",\n      \"talk\": \"Treat your AI context as infrastructure\",\n      \"topic\": \"Platform Engineering\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:platform-engineering\",\n        \"label\": \"Topic\",\n        \"name\": \"Platform Engineering\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-10\",\n        \"label\": \"Talk\",\n        \"name\": \"Treat your AI context as infrastructure\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:christopher-harrison\",\n        \"label\": \"Person\",\n        \"name\": \"Christopher Harrison\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-10\",\n        \"dst_uid\": \"topic:platform-engineering\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:christopher-harrison\",\n        \"dst_uid\": \"talk:U26-10\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "platform-engineering"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.413643,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Christopher Harrison",
            "start_time": "13:00",
            "talk": "Treat your AI context as infrastructure",
            "topic": "Platform Engineering"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:platform-engineering",
              "label": "Topic",
              "name": "Platform Engineering",
              "state": ""
            },
            {
              "uid": "talk:U26-10",
              "label": "Talk",
              "name": "Treat your AI context as infrastructure",
              "state": ""
            },
            {
              "uid": "person:name:christopher-harrison",
              "label": "Person",
              "name": "Christopher Harrison",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-10",
              "dst_uid": "topic:platform-engineering"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:christopher-harrison",
              "dst_uid": "talk:U26-10"
            }
          ]
        }
      },
      "wall_ms": 25543,
      "at": "2026-10-24T21:40:45.690Z",
      "verdict": "pass"
    },
    {
      "id": 85,
      "label": "coverage rca-agent",
      "tool": "talks_by_topic",
      "args": {
        "topics": "rca-agent"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"rca-agent\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.471767,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Achin Gupta\",\n      \"start_time\": \"09:30\",\n      \"talk\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n      \"topic\": \"RCA Agent\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Divya Mahajan\",\n      \"start_time\": \"09:30\",\n      \"talk\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n      \"topic\": \"RCA Agent\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:rca-agent\",\n        \"label\": \"Topic\",\n        \"name\": \"RCA Agent\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-08\",\n        \"label\": \"Talk\",\n        \"name\": \"The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:achin-gupta\",\n        \"label\": \"Person\",\n        \"name\": \"Achin Gupta\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:divya-mahajan\",\n        \"label\": \"Person\",\n        \"name\": \"Divya Mahajan\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-08\",\n        \"dst_uid\": \"topic:rca-agent\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:achin-gupta\",\n        \"dst_uid\": \"talk:U26-08\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:divya-mahajan\",\n        \"dst_uid\": \"talk:U26-08\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "rca-agent"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.471767,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Achin Gupta",
            "start_time": "09:30",
            "talk": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
            "topic": "RCA Agent"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Divya Mahajan",
            "start_time": "09:30",
            "talk": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
            "topic": "RCA Agent"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:rca-agent",
              "label": "Topic",
              "name": "RCA Agent",
              "state": ""
            },
            {
              "uid": "talk:U26-08",
              "label": "Talk",
              "name": "The 2 a.m. RCA agent: Architecting AI that investigates, not hallucinates",
              "state": ""
            },
            {
              "uid": "person:name:achin-gupta",
              "label": "Person",
              "name": "Achin Gupta",
              "state": ""
            },
            {
              "uid": "person:name:divya-mahajan",
              "label": "Person",
              "name": "Divya Mahajan",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-08",
              "dst_uid": "topic:rca-agent"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:achin-gupta",
              "dst_uid": "talk:U26-08"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:divya-mahajan",
              "dst_uid": "talk:U26-08"
            }
          ]
        }
      },
      "wall_ms": 27960,
      "at": "2026-10-24T21:40:48.107Z",
      "verdict": "pass"
    },
    {
      "id": 86,
      "label": "coverage supply-chain-security",
      "tool": "talks_by_topic",
      "args": {
        "topics": "supply-chain-security"
      },
      "expected": "Valid topic; record whether talks exist",
      "follow": "Compare rarest topic / related talk sets",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"supply-chain-security\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.431127,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"Supply Chain Security\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 2 (Cowell Theater)\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"Supply Chain Security\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Karen Li\",\n      \"start_time\": \"16:30\",\n      \"talk\": \"What happens behind the scenes when you run npm install\",\n      \"topic\": \"Supply Chain Security\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 2,\n      \"room\": \"Breakout Stage 1 (Gateway Pavilion)\",\n      \"speaker\": \"Leo Balter\",\n      \"start_time\": \"16:30\",\n      \"talk\": \"What happens behind the scenes when you run npm install\",\n      \"topic\": \"Supply Chain Security\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:supply-chain-security\",\n        \"label\": \"Topic\",\n        \"name\": \"Supply Chain Security\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-05\",\n        \"label\": \"Talk\",\n        \"name\": \"What happens behind the scenes when you run npm install\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:karen-li\",\n        \"label\": \"Person\",\n        \"name\": \"Karen Li\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:leo-balter\",\n        \"label\": \"Person\",\n        \"name\": \"Leo Balter\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:supply-chain-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-05\",\n        \"dst_uid\": \"topic:supply-chain-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:karen-li\",\n        \"dst_uid\": \"talk:U26-05\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:leo-balter\",\n        \"dst_uid\": \"talk:U26-05\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "supply-chain-security"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.431127,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "Supply Chain Security"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 2 (Cowell Theater)",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "Supply Chain Security"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Karen Li",
            "start_time": "16:30",
            "talk": "What happens behind the scenes when you run npm install",
            "topic": "Supply Chain Security"
          },
          {
            "day": "2026-10-28",
            "hops": 2,
            "room": "Breakout Stage 1 (Gateway Pavilion)",
            "speaker": "Leo Balter",
            "start_time": "16:30",
            "talk": "What happens behind the scenes when you run npm install",
            "topic": "Supply Chain Security"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:supply-chain-security",
              "label": "Topic",
              "name": "Supply Chain Security",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            },
            {
              "uid": "talk:U26-05",
              "label": "Talk",
              "name": "What happens behind the scenes when you run npm install",
              "state": ""
            },
            {
              "uid": "person:name:karen-li",
              "label": "Person",
              "name": "Karen Li",
              "state": ""
            },
            {
              "uid": "person:name:leo-balter",
              "label": "Person",
              "name": "Leo Balter",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:supply-chain-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-05",
              "dst_uid": "topic:supply-chain-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:karen-li",
              "dst_uid": "talk:U26-05"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:leo-balter",
              "dst_uid": "talk:U26-05"
            }
          ]
        }
      },
      "wall_ms": 30350,
      "at": "2026-10-24T21:40:50.497Z",
      "verdict": "pass"
    },
    {
      "id": 87,
      "label": "issue unicode injection",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-payload",
        "title": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
        "body": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`\nline two",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "CLASSIFIED; inert title",
      "follow": "Read exact title via own issue query; body not exposed",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-payload\",\n  \"repo_uid\": \"repo:notes/mcp-test-payload\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-payload#2c4df3e7",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-payload",
        "repo_uid": "repo:notes/mcp-test-payload",
        "state": "CLASSIFIED"
      },
      "wall_ms": 2880,
      "at": "2026-10-24T21:41:16.488Z",
      "verdict": "pass"
    },
    {
      "id": 88,
      "label": "encounter unicode injection",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-payload",
        "met_handle": "mcp-test-payload-contact",
        "met_name": "mcp-test-payload-contact",
        "note": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
        "place": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "PARSED; inert exact note/place",
      "follow": "Read through payload issue",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"contact_uid\": \"person:gh:mcp-test-payload-contact\",\n  \"encounter_uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n  \"state\": \"PARSED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "contact_uid": "person:gh:mcp-test-payload-contact",
        "encounter_uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
        "state": "PARSED"
      },
      "wall_ms": 2906,
      "at": "2026-10-24T21:41:19.399Z",
      "verdict": "pass"
    },
    {
      "id": 89,
      "label": "issue long",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-long",
        "title": "mcp-test-long",
        "body": "mcp-test: zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        "topics": [
          "agent-memory"
        ]
      },
      "expected": "Accept 2000-char body",
      "follow": "Read issue existence; body not exposed",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-long#b34588d5\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-long\",\n  \"repo_uid\": \"repo:notes/mcp-test-long\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-long#b34588d5",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-long",
        "repo_uid": "repo:notes/mcp-test-long",
        "state": "CLASSIFIED"
      },
      "wall_ms": 3089,
      "at": "2026-10-24T21:41:22.494Z",
      "verdict": "pass"
    },
    {
      "id": 90,
      "label": "encounter long",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-long",
        "met_handle": "mcp-test-long-contact",
        "met_name": "mcp-test-long-contact",
        "note": "mcp-test: zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        "topics": [
          "agent-memory"
        ]
      },
      "expected": "Accept full 2000-char note",
      "follow": "Readback exact note",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"contact_uid\": \"person:gh:mcp-test-long-contact\",\n  \"encounter_uid\": \"enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57\",\n  \"state\": \"PARSED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "contact_uid": "person:gh:mcp-test-long-contact",
        "encounter_uid": "enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57",
        "state": "PARSED"
      },
      "wall_ms": 3533,
      "at": "2026-10-24T21:41:26.031Z",
      "verdict": "pass"
    },
    {
      "id": 91,
      "label": "issue singleword",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-single",
        "title": "mcp-test-singleword",
        "body": "mcp-test-singleword",
        "topics": [
          "community-governance"
        ]
      },
      "expected": "Accept single word",
      "follow": "Read via own issues",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-single#21c47f94\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-single\",\n  \"repo_uid\": \"repo:notes/mcp-test-single\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-single#21c47f94",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-single",
        "repo_uid": "repo:notes/mcp-test-single",
        "state": "CLASSIFIED"
      },
      "wall_ms": 2711,
      "at": "2026-10-24T21:41:28.746Z",
      "verdict": "pass"
    },
    {
      "id": 92,
      "label": "encounter singleword",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-single",
        "met_name": "mcp-test-single-contact",
        "note": "mcp-test-singleword",
        "topics": [
          "community-governance"
        ]
      },
      "expected": "Accept omitted handle/place",
      "follow": "Read default place and uid",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"contact_uid\": \"person:name:mcp-test-single-contact\",\n  \"encounter_uid\": \"enc:8b96002b-328a-48cc-9ae7-6777d69b00e5\",\n  \"state\": \"PARSED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "contact_uid": "person:name:mcp-test-single-contact",
        "encounter_uid": "enc:8b96002b-328a-48cc-9ae7-6777d69b00e5",
        "state": "PARSED"
      },
      "wall_ms": 4195,
      "at": "2026-10-24T21:41:32.945Z",
      "verdict": "pass"
    },
    {
      "id": 93,
      "label": "blank learned text",
      "tool": "log_learning",
      "args": {
        "person": "mcp-test-validation",
        "learned": "",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Reject empty content",
      "follow": "Recall whether blank learning persisted",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: person and learned are required"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: person and learned are required",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "person and learned are required"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: person and learned are required",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "person and learned are required"
        }
      },
      "wall_ms": 2660,
      "at": "2026-10-24T21:41:35.610Z",
      "verdict": "pass"
    },
    {
      "id": 94,
      "label": "blank issue content",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-validation",
        "title": "",
        "body": "",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Reject empty content",
      "follow": "Read blank issue if accepted",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: person, title and body are required"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: person, title and body are required",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "person, title and body are required"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: person, title and body are required",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "person, title and body are required"
        }
      },
      "wall_ms": 2713,
      "at": "2026-10-24T21:41:38.327Z",
      "verdict": "pass"
    },
    {
      "id": 95,
      "label": "blank encounter note",
      "tool": "log_encounter",
      "args": {
        "person": "person:gh:mcp-test-validation",
        "met_handle": "mcp-test-blank-contact",
        "met_name": "mcp-test-blank-contact",
        "note": "",
        "topics": [
          "mcp-auth"
        ]
      },
      "expected": "Reject empty note or document allowance",
      "follow": "Read blank note if accepted",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "McpServerError: note and met_name are required"
          }
        ],
        "structuredContent": {
          "error": "McpServerError: note and met_name are required",
          "error_code": "INVALID_ARGUMENT",
          "error_data": {
            "type": "json_rpc_error",
            "code": -32602,
            "message": "note and met_name are required"
          }
        },
        "isError": true
      },
      "data": {
        "error": "McpServerError: note and met_name are required",
        "error_code": "INVALID_ARGUMENT",
        "error_data": {
          "type": "json_rpc_error",
          "code": -32602,
          "message": "note and met_name are required"
        }
      },
      "wall_ms": 3134,
      "at": "2026-10-24T21:41:41.465Z",
      "verdict": "pass"
    },
    {
      "id": 96,
      "label": "payload issue title readback",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:mcp-test-payload"
      },
      "expected": "Exact Unicode title and own issue only",
      "follow": "Graph integrity check",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:mcp-test-payload\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.818246,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:mcp-test-payload\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-payload\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"repo:notes/mcp-test-payload\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:mcp-test-payload\",\n        \"dst_uid\": \"repo:notes/mcp-test-payload\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:notes/mcp-test-payload\",\n        \"dst_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:mcp-test-payload"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.818246,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:mcp-test-payload",
              "label": "Person",
              "name": "mcp-test-payload",
              "state": ""
            },
            {
              "uid": "repo:notes/mcp-test-payload",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:notes/mcp-test-payload#2c4df3e7",
              "label": "Issue",
              "name": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:mcp-test-payload",
              "dst_uid": "repo:notes/mcp-test-payload"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:notes/mcp-test-payload",
              "dst_uid": "issue:notes/mcp-test-payload#2c4df3e7"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-payload#2c4df3e7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 3038,
      "at": "2026-10-24T21:41:56.205Z",
      "verdict": "pass"
    },
    {
      "id": 97,
      "label": "payload encounter readback",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-payload#2c4df3e7"
      },
      "expected": "Exact inert Unicode note/place",
      "follow": "Compare equality automatically",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-payload#2c4df3e7\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.474898,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-payload-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-payload-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:41:19.208339+00:00\",\n      \"note\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"place\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"contact\": \"mcp-test-beta-private\",\n      \"contact_uid\": \"person:gh:mcp-test-beta-private\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:39:08.779009+00:00\",\n      \"note\": \"mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024\",\n      \"place\": \"mcp-test-beta-place\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"contact\": \"mcp-test-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:37:03.099287+00:00\",\n      \"note\": \"mcp-test: shared MCP auth experience.\",\n      \"place\": \"mcp-test-hallway\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-payload-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-payload-contact\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-beta-private\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-beta-private\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n        \"dst_uid\": \"person:gh:mcp-test-payload-contact\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"person:gh:mcp-test-beta-private\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"person:gh:mcp-test-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-payload#2c4df3e7"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 0.474898,
        "rows": [
          {
            "contact": "mcp-test-payload-contact",
            "contact_uid": "person:gh:mcp-test-payload-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:41:19.208339+00:00",
            "note": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
            "place": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
            "topic": "MCP Auth"
          },
          {
            "contact": "mcp-test-beta-private",
            "contact_uid": "person:gh:mcp-test-beta-private",
            "hops": 3,
            "met_at": "2026-10-24T21:39:08.779009+00:00",
            "note": "mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024",
            "place": "mcp-test-beta-place",
            "topic": "MCP Auth"
          },
          {
            "contact": "mcp-test-contact",
            "contact_uid": "person:gh:mcp-test-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:37:03.099287+00:00",
            "note": "mcp-test: shared MCP auth experience.",
            "place": "mcp-test-hallway",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-payload#2c4df3e7",
              "label": "Issue",
              "name": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-payload-contact",
              "label": "Person",
              "name": "mcp-test-payload-contact",
              "state": ""
            },
            {
              "uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-beta-private",
              "label": "Person",
              "name": "mcp-test-beta-private",
              "state": ""
            },
            {
              "uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-contact",
              "label": "Person",
              "name": "mcp-test-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-payload#2c4df3e7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
              "dst_uid": "person:gh:mcp-test-payload-contact"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "person:gh:mcp-test-beta-private"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "person:gh:mcp-test-contact"
            }
          ]
        }
      },
      "wall_ms": 5754,
      "at": "2026-10-24T21:41:58.921Z",
      "verdict": "pass"
    },
    {
      "id": 98,
      "label": "long encounter readback",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-long#b34588d5"
      },
      "expected": "Exact 2000-char note; default place",
      "follow": "Compare length",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-long#b34588d5\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.42292,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-long-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-long-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:41:25.818714+00:00\",\n      \"note\": \"mcp-test: zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\",\n      \"place\": \"\",\n      \"topic\": \"Agent Memory\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-long#b34588d5\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test-long\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:agent-memory\",\n        \"label\": \"Topic\",\n        \"name\": \"Agent Memory\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-long-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-long-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-long#b34588d5\",\n        \"dst_uid\": \"topic:agent-memory\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57\",\n        \"dst_uid\": \"topic:agent-memory\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57\",\n        \"dst_uid\": \"person:gh:mcp-test-long-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-long#b34588d5"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 0.42292,
        "rows": [
          {
            "contact": "mcp-test-long-contact",
            "contact_uid": "person:gh:mcp-test-long-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:41:25.818714+00:00",
            "note": "mcp-test: zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
            "place": "",
            "topic": "Agent Memory"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-long#b34588d5",
              "label": "Issue",
              "name": "mcp-test-long",
              "state": "LINKED"
            },
            {
              "uid": "topic:agent-memory",
              "label": "Topic",
              "name": "Agent Memory",
              "state": ""
            },
            {
              "uid": "enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-long-contact",
              "label": "Person",
              "name": "mcp-test-long-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-long#b34588d5",
              "dst_uid": "topic:agent-memory"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57",
              "dst_uid": "topic:agent-memory"
            },
            {
              "type": "MET",
              "src_uid": "enc:d03d5653-d4ed-44ae-84cf-06d4c0203d57",
              "dst_uid": "person:gh:mcp-test-long-contact"
            }
          ]
        }
      },
      "wall_ms": 8704,
      "at": "2026-10-24T21:42:01.871Z",
      "verdict": "pass"
    },
    {
      "id": 99,
      "label": "singleword issue readback",
      "tool": "talks_for_issue",
      "args": {
        "issue": "issue:notes/mcp-test-single#21c47f94"
      },
      "expected": "3-hop result",
      "follow": "Confirm LINKED state",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-single#21c47f94\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.429831,\n  \"rows\": [\n    {\n      \"hops\": 3,\n      \"speaker\": \"Franck Nijhof\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"talk_uid\": \"talk:U26-03\",\n      \"topic\": \"community-governance\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-single#21c47f94\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test-singleword\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:community-governance\",\n        \"label\": \"Topic\",\n        \"name\": \"Community Governance\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-single#21c47f94\",\n        \"dst_uid\": \"topic:community-governance\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:community-governance\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "issue:notes/mcp-test-single#21c47f94"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 3,
        "run_time_ms": 0.429831,
        "rows": [
          {
            "hops": 3,
            "speaker": "Franck Nijhof",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "talk_uid": "talk:U26-03",
            "topic": "community-governance"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-single#21c47f94",
              "label": "Issue",
              "name": "mcp-test-singleword",
              "state": "LINKED"
            },
            {
              "uid": "topic:community-governance",
              "label": "Topic",
              "name": "Community Governance",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-single#21c47f94",
              "dst_uid": "topic:community-governance"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:community-governance"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            }
          ]
        }
      },
      "wall_ms": 11610,
      "at": "2026-10-24T21:42:04.777Z",
      "verdict": "pass"
    },
    {
      "id": 100,
      "label": "singleword encounter readback",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-single#21c47f94"
      },
      "expected": "Exact note; no-handle identity",
      "follow": "Repeat no duplicate rows",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-single#21c47f94\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.418855,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-single-contact\",\n      \"contact_uid\": \"person:name:mcp-test-single-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:41:32.753211+00:00\",\n      \"note\": \"mcp-test-singleword\",\n      \"place\": \"\",\n      \"topic\": \"Community Governance\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-single#21c47f94\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test-singleword\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:community-governance\",\n        \"label\": \"Topic\",\n        \"name\": \"Community Governance\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:8b96002b-328a-48cc-9ae7-6777d69b00e5\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:name:mcp-test-single-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-single-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-single#21c47f94\",\n        \"dst_uid\": \"topic:community-governance\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:8b96002b-328a-48cc-9ae7-6777d69b00e5\",\n        \"dst_uid\": \"topic:community-governance\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:8b96002b-328a-48cc-9ae7-6777d69b00e5\",\n        \"dst_uid\": \"person:name:mcp-test-single-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-single#21c47f94"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 0.418855,
        "rows": [
          {
            "contact": "mcp-test-single-contact",
            "contact_uid": "person:name:mcp-test-single-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:41:32.753211+00:00",
            "note": "mcp-test-singleword",
            "place": "",
            "topic": "Community Governance"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-single#21c47f94",
              "label": "Issue",
              "name": "mcp-test-singleword",
              "state": "LINKED"
            },
            {
              "uid": "topic:community-governance",
              "label": "Topic",
              "name": "Community Governance",
              "state": ""
            },
            {
              "uid": "enc:8b96002b-328a-48cc-9ae7-6777d69b00e5",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:name:mcp-test-single-contact",
              "label": "Person",
              "name": "mcp-test-single-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-single#21c47f94",
              "dst_uid": "topic:community-governance"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:8b96002b-328a-48cc-9ae7-6777d69b00e5",
              "dst_uid": "topic:community-governance"
            },
            {
              "type": "MET",
              "src_uid": "enc:8b96002b-328a-48cc-9ae7-6777d69b00e5",
              "dst_uid": "person:name:mcp-test-single-contact"
            }
          ]
        }
      },
      "wall_ms": 15635,
      "at": "2026-10-24T21:42:08.802Z",
      "verdict": "pass"
    },
    {
      "id": 101,
      "label": "alpha oauth filter",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": "oauth"
      },
      "expected": "Only OAuth tagged alpha learnings",
      "follow": "Compare unknown topic",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [\n        \"oauth\"\n      ],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [\n        \"oauth\"\n      ],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [
              "oauth"
            ],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [
              "oauth"
            ],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 18254,
      "at": "2026-10-24T21:42:11.421Z",
      "verdict": "fail"
    },
    {
      "id": 102,
      "label": "validation person remains empty",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-validation"
      },
      "expected": "No writes from rejected attempts",
      "follow": "Repeat unknown read topic",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [],\n  \"person_uid\": \"person:gh:mcp-test-validation\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [],
        "person_uid": "person:gh:mcp-test-validation"
      },
      "wall_ms": 20949,
      "at": "2026-10-24T21:42:14.116Z",
      "verdict": "pass"
    },
    {
      "id": 103,
      "label": "topics after injection",
      "tool": "list_topics",
      "args": {},
      "expected": "Exactly original 22 topics",
      "follow": "Repeat final topics",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[\n  {\n    \"name\": \"Agent Memory\",\n    \"slug\": \"agent-memory\"\n  },\n  {\n    \"name\": \"Agentic Security\",\n    \"slug\": \"agentic-security\"\n  },\n  {\n    \"name\": \"Agentic Workflows\",\n    \"slug\": \"agentic-workflows\"\n  },\n  {\n    \"name\": \"AI-Assisted Testing\",\n    \"slug\": \"ai-assisted-testing\"\n  },\n  {\n    \"name\": \"AI Context\",\n    \"slug\": \"ai-context\"\n  },\n  {\n    \"name\": \"AI Operations\",\n    \"slug\": \"ai-operations\"\n  },\n  {\n    \"name\": \"CI/CD\",\n    \"slug\": \"cicd\"\n  },\n  {\n    \"name\": \"Community Governance\",\n    \"slug\": \"community-governance\"\n  },\n  {\n    \"name\": \"Copilot\",\n    \"slug\": \"copilot\"\n  },\n  {\n    \"name\": \"Dependencies\",\n    \"slug\": \"dependencies\"\n  },\n  {\n    \"name\": \"Deterministic Verification\",\n    \"slug\": \"deterministic-verification\"\n  },\n  {\n    \"name\": \"Developer Experience\",\n    \"slug\": \"developer-experience\"\n  },\n  {\n    \"name\": \"Enterprise AI Adoption\",\n    \"slug\": \"enterprise-ai-adoption\"\n  },\n  {\n    \"name\": \"Frontier Models\",\n    \"slug\": \"frontier-models\"\n  },\n  {\n    \"name\": \"GitHub Actions\",\n    \"slug\": \"github-actions\"\n  },\n  {\n    \"name\": \"MCP Auth\",\n    \"slug\": \"mcp-auth\"\n  },\n  {\n    \"name\": \"OAuth\",\n    \"slug\": \"oauth\"\n  },\n  {\n    \"name\": \"Open Source\",\n    \"slug\": \"open-source\"\n  },\n  {\n    \"name\": \"Package Management\",\n    \"slug\": \"package-management\"\n  },\n  {\n    \"name\": \"Platform Engineering\",\n    \"slug\": \"platform-engineering\"\n  },\n  {\n    \"name\": \"RCA Agent\",\n    \"slug\": \"rca-agent\"\n  },\n  {\n    \"name\": \"Supply Chain Security\",\n    \"slug\": \"supply-chain-security\"\n  }\n]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": [
        {
          "name": "Agent Memory",
          "slug": "agent-memory"
        },
        {
          "name": "Agentic Security",
          "slug": "agentic-security"
        },
        {
          "name": "Agentic Workflows",
          "slug": "agentic-workflows"
        },
        {
          "name": "AI-Assisted Testing",
          "slug": "ai-assisted-testing"
        },
        {
          "name": "AI Context",
          "slug": "ai-context"
        },
        {
          "name": "AI Operations",
          "slug": "ai-operations"
        },
        {
          "name": "CI/CD",
          "slug": "cicd"
        },
        {
          "name": "Community Governance",
          "slug": "community-governance"
        },
        {
          "name": "Copilot",
          "slug": "copilot"
        },
        {
          "name": "Dependencies",
          "slug": "dependencies"
        },
        {
          "name": "Deterministic Verification",
          "slug": "deterministic-verification"
        },
        {
          "name": "Developer Experience",
          "slug": "developer-experience"
        },
        {
          "name": "Enterprise AI Adoption",
          "slug": "enterprise-ai-adoption"
        },
        {
          "name": "Frontier Models",
          "slug": "frontier-models"
        },
        {
          "name": "GitHub Actions",
          "slug": "github-actions"
        },
        {
          "name": "MCP Auth",
          "slug": "mcp-auth"
        },
        {
          "name": "OAuth",
          "slug": "oauth"
        },
        {
          "name": "Open Source",
          "slug": "open-source"
        },
        {
          "name": "Package Management",
          "slug": "package-management"
        },
        {
          "name": "Platform Engineering",
          "slug": "platform-engineering"
        },
        {
          "name": "RCA Agent",
          "slug": "rca-agent"
        },
        {
          "name": "Supply Chain Security",
          "slug": "supply-chain-security"
        }
      ],
      "wall_ms": 25797,
      "at": "2026-10-24T21:42:18.964Z",
      "verdict": "pass"
    },
    {
      "id": 104,
      "label": "presenter after injection",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21"
      },
      "expected": "Same original issue rows, no lost graph",
      "follow": "Compare full baseline row projection",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.349588,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.349588,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 29043,
      "at": "2026-10-24T21:42:22.210Z",
      "verdict": "pass"
    },
    {
      "id": 105,
      "label": "known opposite date excludes",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2026-10-28"
      },
      "expected": "Zero rows",
      "follow": "Allowed 10-29 date includes",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"2026-10-28\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 1.041366,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "2026-10-28",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 1.041366,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 31884,
      "at": "2026-10-24T21:42:25.051Z",
      "verdict": "pass"
    },
    {
      "id": 106,
      "label": "presenter day29 filters",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21",
        "day": "2026-10-29"
      },
      "expected": "Only 10-29 rows",
      "follow": "Compare baseline",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"2026-10-29\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.034697,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n      \"speaker\": \"Jeff An\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"Agentic Security\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"Standardize shared agent memory & context infrastructure across local and cloud runtimes\",\n      \"speaker\": \"Kate Catlin\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"AI Context\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"Standardize shared agent memory & context infrastructure across local and cloud runtimes\",\n      \"speaker\": \"Cara Phillips\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"AI Context\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"Standardize shared agent memory & context infrastructure across local and cloud runtimes\",\n      \"speaker\": \"Rohan Varma\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"The view from the labs: What's next for AI-assisted development\",\n      \"topic\": \"AI Context\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 5,\n      \"issue\": \"Race condition & hallucinated root-cause in multi-agent state handoff during 2am incident triage\",\n      \"speaker\": \"Jeff An\",\n      \"start_time\": \"09:00\",\n      \"talk\": \"Beyond pass or fail: How agents verify AI-generated code\",\n      \"topic\": \"Deterministic Verification\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"label\": \"Issue\",\n        \"name\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:agentic-security\",\n        \"label\": \"Topic\",\n        \"name\": \"Agentic Security\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-07\",\n        \"label\": \"Talk\",\n        \"name\": \"Beyond pass or fail: How agents verify AI-generated code\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jeff-an\",\n        \"label\": \"Person\",\n        \"name\": \"Jeff An\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#4\",\n        \"label\": \"Issue\",\n        \"name\": \"Standardize shared agent memory & context infrastructure across local and cloud runtimes\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:ai-context\",\n        \"label\": \"Topic\",\n        \"name\": \"AI Context\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-06\",\n        \"label\": \"Talk\",\n        \"name\": \"The view from the labs: What's next for AI-assisted development\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:kate-catlin\",\n        \"label\": \"Person\",\n        \"name\": \"Kate Catlin\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:cara-phillips\",\n        \"label\": \"Person\",\n        \"name\": \"Cara Phillips\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:rohan-varma\",\n        \"label\": \"Person\",\n        \"name\": \"Rohan Varma\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#2\",\n        \"label\": \"Issue\",\n        \"name\": \"Race condition & hallucinated root-cause in multi-agent state handoff during 2am incident triage\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:deterministic-verification\",\n        \"label\": \"Topic\",\n        \"name\": \"Deterministic Verification\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#1\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:agentic-security\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jeff-an\",\n        \"dst_uid\": \"talk:U26-07\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#4\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#4\",\n        \"dst_uid\": \"topic:ai-context\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-06\",\n        \"dst_uid\": \"topic:ai-context\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:kate-catlin\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:cara-phillips\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:rohan-varma\",\n        \"dst_uid\": \"talk:U26-06\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#2\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#2\",\n        \"dst_uid\": \"topic:deterministic-verification\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-07\",\n        \"dst_uid\": \"topic:deterministic-verification\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "2026-10-29",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.034697,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
            "speaker": "Jeff An",
            "start_time": "09:00",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "Agentic Security"
          },
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "Standardize shared agent memory & context infrastructure across local and cloud runtimes",
            "speaker": "Kate Catlin",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "AI Context"
          },
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "Standardize shared agent memory & context infrastructure across local and cloud runtimes",
            "speaker": "Cara Phillips",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "AI Context"
          },
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "Standardize shared agent memory & context infrastructure across local and cloud runtimes",
            "speaker": "Rohan Varma",
            "start_time": "09:00",
            "talk": "The view from the labs: What's next for AI-assisted development",
            "topic": "AI Context"
          },
          {
            "day": "2026-10-29",
            "hops": 5,
            "issue": "Race condition & hallucinated root-cause in multi-agent state handoff during 2am incident triage",
            "speaker": "Jeff An",
            "start_time": "09:00",
            "talk": "Beyond pass or fail: How agents verify AI-generated code",
            "topic": "Deterministic Verification"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#1",
              "label": "Issue",
              "name": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
              "state": "LINKED"
            },
            {
              "uid": "topic:agentic-security",
              "label": "Topic",
              "name": "Agentic Security",
              "state": ""
            },
            {
              "uid": "talk:U26-07",
              "label": "Talk",
              "name": "Beyond pass or fail: How agents verify AI-generated code",
              "state": ""
            },
            {
              "uid": "person:name:jeff-an",
              "label": "Person",
              "name": "Jeff An",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#4",
              "label": "Issue",
              "name": "Standardize shared agent memory & context infrastructure across local and cloud runtimes",
              "state": "LINKED"
            },
            {
              "uid": "topic:ai-context",
              "label": "Topic",
              "name": "AI Context",
              "state": ""
            },
            {
              "uid": "talk:U26-06",
              "label": "Talk",
              "name": "The view from the labs: What's next for AI-assisted development",
              "state": ""
            },
            {
              "uid": "person:name:kate-catlin",
              "label": "Person",
              "name": "Kate Catlin",
              "state": ""
            },
            {
              "uid": "person:name:cara-phillips",
              "label": "Person",
              "name": "Cara Phillips",
              "state": ""
            },
            {
              "uid": "person:name:rohan-varma",
              "label": "Person",
              "name": "Rohan Varma",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#2",
              "label": "Issue",
              "name": "Race condition & hallucinated root-cause in multi-agent state handoff during 2am incident triage",
              "state": "LINKED"
            },
            {
              "uid": "topic:deterministic-verification",
              "label": "Topic",
              "name": "Deterministic Verification",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#1"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#1",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:agentic-security"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jeff-an",
              "dst_uid": "talk:U26-07"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#4"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#4",
              "dst_uid": "topic:ai-context"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-06",
              "dst_uid": "topic:ai-context"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:kate-catlin",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:cara-phillips",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:rohan-varma",
              "dst_uid": "talk:U26-06"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#2"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#2",
              "dst_uid": "topic:deterministic-verification"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-07",
              "dst_uid": "topic:deterministic-verification"
            }
          ]
        }
      },
      "wall_ms": 34975,
      "at": "2026-10-24T21:42:28.142Z",
      "verdict": "pass"
    },
    {
      "id": 107,
      "label": "valid topic no time matches",
      "tool": "talks_by_topic",
      "args": {
        "topics": "community-governance",
        "after": "23:59"
      },
      "expected": "Graceful empty rows and envelope",
      "follow": "Final empty-envelope structural check",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"23:59\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"community-governance\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.235893,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "23:59",
          "before": "",
          "day": "",
          "topics": "community-governance"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.235893,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 38364,
      "at": "2026-10-24T21:42:31.531Z",
      "verdict": "pass"
    },
    {
      "id": 108,
      "label": "learning known absent topic",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": "agent-memory"
      },
      "expected": "Zero records; alpha has no agent-memory learning",
      "follow": "Unknown topic comparison",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 3009,
      "at": "2026-10-24T21:42:56.430Z",
      "verdict": "fail"
    },
    {
      "id": 109,
      "label": "learning unknown topic postwrites",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": "mcp-test-unknown"
      },
      "expected": "Unknown slug error",
      "follow": "Empty topic comparison",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 5916,
      "at": "2026-10-24T21:42:59.337Z",
      "verdict": "fail"
    },
    {
      "id": 110,
      "label": "learning empty topic all",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": ""
      },
      "expected": "Same all records and full topic arrays",
      "follow": "Multi-topic preservation",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [\n        \"oauth\"\n      ],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [\n        \"mcp-auth\",\n        \"oauth\"\n      ],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [
              "oauth"
            ],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [
              "mcp-auth",
              "oauth"
            ],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 8793,
      "at": "2026-10-24T21:43:02.214Z",
      "verdict": "pass"
    },
    {
      "id": 111,
      "label": "learning mcp-auth filter",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-alpha",
        "topic": "mcp-auth"
      },
      "expected": "Only mcp-auth records; full tags preserved",
      "follow": "Compare oauth record",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 11659,
      "at": "2026-10-24T21:43:05.080Z",
      "verdict": "fail"
    },
    {
      "id": 112,
      "label": "repeat contact alerts",
      "tool": "check_alerts",
      "args": {},
      "expected": "CONTACT only and stable bridge identities",
      "follow": "Record all observed bridge ids",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[\n  {\n    \"bridge_uid\": \"bridge:d20f4fac-77b0-4482-8757-ed74dbf4208c\",\n    \"contact\": \"mcp-test-long-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Standardize shared agent memory & context infrastructure across local and cloud runtimes\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#4\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"Agent Memory\"\n  },\n  {\n    \"bridge_uid\": \"bridge:483f0dfa-7642-445f-a35e-538166eceef8\",\n    \"contact\": \"mcp-test-payload-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:e5ad65fd-72b5-46b0-abc5-80f3fa12f158\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"3\",\n    \"issue\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n    \"issue_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:34be0887-cf6b-4dfa-9c3f-c0f60f756d83\",\n    \"contact\": \"mcp-test-beta-private\",\n    \"hops\": \"3\",\n    \"issue\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n    \"issue_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:9cf26a79-4ad7-470a-8cec-b5f2454dd6f5\",\n    \"contact\": \"mcp-test-beta-private\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:1a81cd1e-a24c-4723-8c2d-6b7bbacff29b\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"3\",\n    \"issue\": \"mcp-test: beta-only auth issue\",\n    \"issue_uid\": \"issue:notes/mcp-test-beta#f6cc6fcd\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  }\n]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": [
        {
          "bridge_uid": "bridge:d20f4fac-77b0-4482-8757-ed74dbf4208c",
          "contact": "mcp-test-long-contact",
          "hops": "4",
          "issue": "Standardize shared agent memory & context infrastructure across local and cloud runtimes",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#4",
          "state": "ACTION_READY",
          "topic": "Agent Memory"
        },
        {
          "bridge_uid": "bridge:483f0dfa-7642-445f-a35e-538166eceef8",
          "contact": "mcp-test-payload-contact",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:e5ad65fd-72b5-46b0-abc5-80f3fa12f158",
          "contact": "mcp-test-contact",
          "hops": "3",
          "issue": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
          "issue_uid": "issue:notes/mcp-test-payload#2c4df3e7",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:34be0887-cf6b-4dfa-9c3f-c0f60f756d83",
          "contact": "mcp-test-beta-private",
          "hops": "3",
          "issue": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
          "issue_uid": "issue:notes/mcp-test-payload#2c4df3e7",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:9cf26a79-4ad7-470a-8cec-b5f2454dd6f5",
          "contact": "mcp-test-beta-private",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:1a81cd1e-a24c-4723-8c2d-6b7bbacff29b",
          "contact": "mcp-test-contact",
          "hops": "3",
          "issue": "mcp-test: beta-only auth issue",
          "issue_uid": "issue:notes/mcp-test-beta#f6cc6fcd",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd",
          "contact": "mcp-test-contact",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        }
      ],
      "wall_ms": 14700,
      "at": "2026-10-24T21:43:08.121Z",
      "verdict": "fail"
    },
    {
      "id": 113,
      "label": "presenter memory isolation proof",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:manvendersingh21/deepcore-n1#1"
      },
      "expected": "No test encounters owned by other people",
      "follow": "Confirm synthetic leakage",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:manvendersingh21/deepcore-n1#1\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.531167,\n  \"rows\": [\n    {\n      \"contact\": \"mcp-test-payload-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-payload-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:41:19.208339+00:00\",\n      \"note\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"place\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"contact\": \"mcp-test-beta-private\",\n      \"contact_uid\": \"person:gh:mcp-test-beta-private\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:39:08.779009+00:00\",\n      \"note\": \"mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024\",\n      \"place\": \"mcp-test-beta-place\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"contact\": \"mcp-test-contact\",\n      \"contact_uid\": \"person:gh:mcp-test-contact\",\n      \"hops\": 3,\n      \"met_at\": \"2026-10-24T21:37:03.099287+00:00\",\n      \"note\": \"mcp-test: shared MCP auth experience.\",\n      \"place\": \"mcp-test-hallway\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"label\": \"Issue\",\n        \"name\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-payload-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-payload-contact\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-beta-private\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-beta-private\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"label\": \"Encounter\",\n        \"name\": \"\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"person:gh:mcp-test-contact\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-contact\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:80da1b5b-f799-4ad6-82b2-435c2adc194e\",\n        \"dst_uid\": \"person:gh:mcp-test-payload-contact\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a\",\n        \"dst_uid\": \"person:gh:mcp-test-beta-private\"\n      },\n      {\n        \"type\": \"DISCUSSED\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"MET\",\n        \"src_uid\": \"enc:308e4ce9-9740-456d-85b4-dc315561230f\",\n        \"dst_uid\": \"person:gh:mcp-test-contact\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:manvendersingh21/deepcore-n1#1"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 3,
        "run_time_ms": 0.531167,
        "rows": [
          {
            "contact": "mcp-test-payload-contact",
            "contact_uid": "person:gh:mcp-test-payload-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:41:19.208339+00:00",
            "note": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
            "place": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
            "topic": "MCP Auth"
          },
          {
            "contact": "mcp-test-beta-private",
            "contact_uid": "person:gh:mcp-test-beta-private",
            "hops": 3,
            "met_at": "2026-10-24T21:39:08.779009+00:00",
            "note": "mcp-test: BETA_ONLY_PRIVATE_NOTE_20261024",
            "place": "mcp-test-beta-place",
            "topic": "MCP Auth"
          },
          {
            "contact": "mcp-test-contact",
            "contact_uid": "person:gh:mcp-test-contact",
            "hops": 3,
            "met_at": "2026-10-24T21:37:03.099287+00:00",
            "note": "mcp-test: shared MCP auth experience.",
            "place": "mcp-test-hallway",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:manvendersingh21/deepcore-n1#1",
              "label": "Issue",
              "name": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-payload-contact",
              "label": "Person",
              "name": "mcp-test-payload-contact",
              "state": ""
            },
            {
              "uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-beta-private",
              "label": "Person",
              "name": "mcp-test-beta-private",
              "state": ""
            },
            {
              "uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "label": "Encounter",
              "name": "",
              "state": "LINKED"
            },
            {
              "uid": "person:gh:mcp-test-contact",
              "label": "Person",
              "name": "mcp-test-contact",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#1",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:80da1b5b-f799-4ad6-82b2-435c2adc194e",
              "dst_uid": "person:gh:mcp-test-payload-contact"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:19bd82c6-8eb7-4c9b-adc9-1e990594ef5a",
              "dst_uid": "person:gh:mcp-test-beta-private"
            },
            {
              "type": "DISCUSSED",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "MET",
              "src_uid": "enc:308e4ce9-9740-456d-85b4-dc315561230f",
              "dst_uid": "person:gh:mcp-test-contact"
            }
          ]
        }
      },
      "wall_ms": 18266,
      "at": "2026-10-24T21:43:11.687Z",
      "verdict": "fail"
    },
    {
      "id": 114,
      "label": "unknown person learnings",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-missing"
      },
      "expected": "Empty with canonical uid; no leak",
      "follow": "Repeat no mutation",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [],\n  \"person_uid\": \"person:gh:mcp-test-missing\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [],
        "person_uid": "person:gh:mcp-test-missing"
      },
      "wall_ms": 21847,
      "at": "2026-10-24T21:43:15.268Z",
      "verdict": "pass"
    },
    {
      "id": 115,
      "label": "missing person read validation",
      "tool": "talks_for_my_issues",
      "args": {},
      "expected": "Schema rejects missing required person",
      "follow": "Blank uid comparison",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "failed to deserialize parameters: missing field `person`"
          }
        ],
        "structuredContent": {
          "error_code": "INVALID_ARGUMENT"
        },
        "isError": true
      },
      "data": {
        "error_code": "INVALID_ARGUMENT"
      },
      "wall_ms": 24824,
      "at": "2026-10-24T21:43:18.245Z",
      "verdict": "pass"
    },
    {
      "id": 116,
      "label": "missing issue read validation",
      "tool": "talks_for_issue",
      "args": {},
      "expected": "Schema rejects missing issue",
      "follow": "Malformed uid comparison",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "failed to deserialize parameters: missing field `issue`"
          }
        ],
        "structuredContent": {
          "error_code": "INVALID_ARGUMENT"
        },
        "isError": true
      },
      "data": {
        "error_code": "INVALID_ARGUMENT"
      },
      "wall_ms": 27651,
      "at": "2026-10-24T21:43:21.072Z",
      "verdict": "pass"
    },
    {
      "id": 117,
      "label": "topics duplicate",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth,mcp-auth"
      },
      "expected": "One row, no multiplication",
      "follow": "Whitespace trimming",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth,mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.507501,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth,mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.507501,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 30738,
      "at": "2026-10-24T21:43:24.159Z",
      "verdict": "pass"
    },
    {
      "id": 118,
      "label": "timing sample 1",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-timing-1",
        "title": "mcp-test: timing sample 1",
        "body": "mcp-test: measure bridge readiness",
        "topics": [
          "copilot"
        ]
      },
      "expected": "CLASSIFIED then LINKED within ~10sec",
      "follow": "Immediate poll then repeat",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-timing-1#c30e2013\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-timing-1\",\n  \"repo_uid\": \"repo:notes/mcp-test-timing-1\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-timing-1#c30e2013",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-timing-1",
        "repo_uid": "repo:notes/mcp-test-timing-1",
        "state": "CLASSIFIED"
      },
      "wall_ms": 3143,
      "at": "2026-10-24T21:43:40.244Z",
      "verdict": "pass"
    },
    {
      "id": 119,
      "label": "timing poll 1",
      "tool": "talks_for_issue",
      "args": {
        "issue": "issue:notes/mcp-test-timing-1#c30e2013"
      },
      "expected": "3 hops with LINKED issue",
      "follow": "Repeat final state read",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-timing-1#c30e2013\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.466554,\n  \"rows\": [\n    {\n      \"hops\": 3,\n      \"speaker\": \"Jared Hatfield\",\n      \"talk\": \"Building AI fluency at UPS\",\n      \"talk_uid\": \"talk:U26-02\",\n      \"topic\": \"copilot\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-timing-1#c30e2013\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: timing sample 1\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:copilot\",\n        \"label\": \"Topic\",\n        \"name\": \"Copilot\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-02\",\n        \"label\": \"Talk\",\n        \"name\": \"Building AI fluency at UPS\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jared-hatfield\",\n        \"label\": \"Person\",\n        \"name\": \"Jared Hatfield\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-timing-1#c30e2013\",\n        \"dst_uid\": \"topic:copilot\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-02\",\n        \"dst_uid\": \"topic:copilot\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jared-hatfield\",\n        \"dst_uid\": \"talk:U26-02\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "issue:notes/mcp-test-timing-1#c30e2013"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 3,
        "run_time_ms": 0.466554,
        "rows": [
          {
            "hops": 3,
            "speaker": "Jared Hatfield",
            "talk": "Building AI fluency at UPS",
            "talk_uid": "talk:U26-02",
            "topic": "copilot"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-timing-1#c30e2013",
              "label": "Issue",
              "name": "mcp-test: timing sample 1",
              "state": "LINKED"
            },
            {
              "uid": "topic:copilot",
              "label": "Topic",
              "name": "Copilot",
              "state": ""
            },
            {
              "uid": "talk:U26-02",
              "label": "Talk",
              "name": "Building AI fluency at UPS",
              "state": ""
            },
            {
              "uid": "person:name:jared-hatfield",
              "label": "Person",
              "name": "Jared Hatfield",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-timing-1#c30e2013",
              "dst_uid": "topic:copilot"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-02",
              "dst_uid": "topic:copilot"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jared-hatfield",
              "dst_uid": "talk:U26-02"
            }
          ]
        }
      },
      "wall_ms": 2962,
      "at": "2026-10-24T21:43:43.212Z",
      "verdict": "pass"
    },
    {
      "id": 120,
      "label": "timing sample 2",
      "tool": "log_issue",
      "args": {
        "person": "mcp-test-timing-2",
        "title": "mcp-test: timing sample 2",
        "body": "mcp-test: measure bridge readiness",
        "topics": [
          "copilot"
        ]
      },
      "expected": "CLASSIFIED then LINKED within ~10sec",
      "follow": "Immediate poll then repeat",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"issue_uid\": \"issue:notes/mcp-test-timing-2#bc4fd6eb\",\n  \"next\": \"Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.\",\n  \"person_uid\": \"person:gh:mcp-test-timing-2\",\n  \"repo_uid\": \"repo:notes/mcp-test-timing-2\",\n  \"state\": \"CLASSIFIED\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "issue_uid": "issue:notes/mcp-test-timing-2#bc4fd6eb",
        "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts.",
        "person_uid": "person:gh:mcp-test-timing-2",
        "repo_uid": "repo:notes/mcp-test-timing-2",
        "state": "CLASSIFIED"
      },
      "wall_ms": 3011,
      "at": "2026-10-24T21:43:46.229Z",
      "verdict": "pass"
    },
    {
      "id": 121,
      "label": "timing poll 2",
      "tool": "talks_for_issue",
      "args": {
        "issue": "issue:notes/mcp-test-timing-2#bc4fd6eb"
      },
      "expected": "3 hops with LINKED issue",
      "follow": "Repeat final state read",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-timing-2#bc4fd6eb\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 3,\n  \"run_time_ms\": 0.438109,\n  \"rows\": [\n    {\n      \"hops\": 3,\n      \"speaker\": \"Jared Hatfield\",\n      \"talk\": \"Building AI fluency at UPS\",\n      \"talk_uid\": \"talk:U26-02\",\n      \"topic\": \"copilot\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"issue:notes/mcp-test-timing-2#bc4fd6eb\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: timing sample 2\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:copilot\",\n        \"label\": \"Topic\",\n        \"name\": \"Copilot\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-02\",\n        \"label\": \"Talk\",\n        \"name\": \"Building AI fluency at UPS\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:jared-hatfield\",\n        \"label\": \"Person\",\n        \"name\": \"Jared Hatfield\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-timing-2#bc4fd6eb\",\n        \"dst_uid\": \"topic:copilot\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-02\",\n        \"dst_uid\": \"topic:copilot\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:jared-hatfield\",\n        \"dst_uid\": \"talk:U26-02\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "issue:notes/mcp-test-timing-2#bc4fd6eb"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 3,
        "run_time_ms": 0.438109,
        "rows": [
          {
            "hops": 3,
            "speaker": "Jared Hatfield",
            "talk": "Building AI fluency at UPS",
            "talk_uid": "talk:U26-02",
            "topic": "copilot"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "issue:notes/mcp-test-timing-2#bc4fd6eb",
              "label": "Issue",
              "name": "mcp-test: timing sample 2",
              "state": "LINKED"
            },
            {
              "uid": "topic:copilot",
              "label": "Topic",
              "name": "Copilot",
              "state": ""
            },
            {
              "uid": "talk:U26-02",
              "label": "Talk",
              "name": "Building AI fluency at UPS",
              "state": ""
            },
            {
              "uid": "person:name:jared-hatfield",
              "label": "Person",
              "name": "Jared Hatfield",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-timing-2#bc4fd6eb",
              "dst_uid": "topic:copilot"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-02",
              "dst_uid": "topic:copilot"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:jared-hatfield",
              "dst_uid": "talk:U26-02"
            }
          ]
        }
      },
      "wall_ms": 2714,
      "at": "2026-10-24T21:43:48.948Z",
      "verdict": "pass"
    },
    {
      "id": 122,
      "label": "unknown topic recall reproducer beta",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-beta",
        "topic": "agent-memory"
      },
      "expected": "Empty since beta only mcp-auth",
      "follow": "Confirm same filter defect on beta",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:38:16.715559+00:00\",\n      \"source\": \"mcp-test-duplicate\",\n      \"text\": \"mcp-test: identical lesson.\",\n      \"topics\": [],\n      \"uid\": \"learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:13.219946+00:00\",\n      \"source\": \"mcp-test-duplicate\",\n      \"text\": \"mcp-test: identical lesson.\",\n      \"topics\": [],\n      \"uid\": \"learning:05362c1e-7200-494e-9185-f8eabf1bcd25\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-beta\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:38:16.715559+00:00",
            "source": "mcp-test-duplicate",
            "text": "mcp-test: identical lesson.",
            "topics": [],
            "uid": "learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2"
          },
          {
            "learned_at": "2026-10-24T21:38:13.219946+00:00",
            "source": "mcp-test-duplicate",
            "text": "mcp-test: identical lesson.",
            "topics": [],
            "uid": "learning:05362c1e-7200-494e-9185-f8eabf1bcd25"
          }
        ],
        "person_uid": "person:gh:mcp-test-beta"
      },
      "wall_ms": 2599,
      "at": "2026-10-24T21:43:51.552Z",
      "verdict": "fail"
    },
    {
      "id": 123,
      "label": "read raw handle normalization",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "mcp-test-alpha"
      },
      "expected": "Canonicalize or reject incorrect UID; document behavior",
      "follow": "Compare full canonical UID",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"mcp-test-alpha\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.968921,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "mcp-test-alpha"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.968921,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 5994,
      "at": "2026-10-24T21:43:54.947Z",
      "verdict": "fail"
    },
    {
      "id": 124,
      "label": "recall canonical person",
      "tool": "recall_learnings",
      "args": {
        "person": "person:gh:mcp-test-alpha"
      },
      "expected": "Same records as raw handle",
      "follow": "Compare canonical uid",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:39:11.832254+00:00\",\n      \"source\": \"mcp-test: 雪 ❄️ ' \\\" \\\\ newline\\nend\",\n      \"text\": \"mcp-test: नमस्ते 🧵 café é \\\"quotes\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:27.884140+00:00\",\n      \"source\": \"\",\n      \"text\": \"mcp-test-singleword\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:58efc5ad-c647-4c10-8137-d0f25a004ae9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:23.528946+00:00\",\n      \"source\": \"mcp-test-long\",\n      \"text\": \"mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\",\n      \"topics\": [\n        \"oauth\"\n      ],\n      \"uid\": \"learning:90d70c0a-1147-414c-b29e-c659500bcb48\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:10.190072+00:00\",\n      \"source\": \"mcp-test-overlap\",\n      \"text\": \"mcp-test: second overlapping lesson.\",\n      \"topics\": [\n        \"mcp-auth\",\n        \"oauth\"\n      ],\n      \"uid\": \"learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:36:53.244033+00:00\",\n      \"source\": \"mcp-test-roundtrip\",\n      \"text\": \"mcp-test: MCP auth scopes require explicit checks.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9d307e75-e63d-42d8-985b-f2471f6a6303\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-alpha\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:39:11.832254+00:00",
            "source": "mcp-test: 雪 ❄️ ' \" \\ newline\nend",
            "text": "mcp-test: नमस्ते 🧵 café é \"quotes\" 'single' `MATCH (n) DETACH DELETE n`",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:048adc75-25e4-4a20-9ec1-e7e245e5fc6f"
          },
          {
            "learned_at": "2026-10-24T21:38:27.884140+00:00",
            "source": "",
            "text": "mcp-test-singleword",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:58efc5ad-c647-4c10-8137-d0f25a004ae9"
          },
          {
            "learned_at": "2026-10-24T21:38:23.528946+00:00",
            "source": "mcp-test-long",
            "text": "mcp-test: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "topics": [
              "oauth"
            ],
            "uid": "learning:90d70c0a-1147-414c-b29e-c659500bcb48"
          },
          {
            "learned_at": "2026-10-24T21:38:10.190072+00:00",
            "source": "mcp-test-overlap",
            "text": "mcp-test: second overlapping lesson.",
            "topics": [
              "mcp-auth",
              "oauth"
            ],
            "uid": "learning:ee200c29-34e6-4f23-84c5-ccd22a83c1b9"
          },
          {
            "learned_at": "2026-10-24T21:36:53.244033+00:00",
            "source": "mcp-test-roundtrip",
            "text": "mcp-test: MCP auth scopes require explicit checks.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9d307e75-e63d-42d8-985b-f2471f6a6303"
          }
        ],
        "person_uid": "person:gh:mcp-test-alpha"
      },
      "wall_ms": 8654,
      "at": "2026-10-24T21:43:57.607Z",
      "verdict": "pass"
    },
    {
      "id": 125,
      "label": "malformed time short",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "after": "9:00"
      },
      "expected": "Reject invalid HH:MM",
      "follow": "Canonical 09:00 comparison",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"09:00\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.487644,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "09:00",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.487644,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 11328,
      "at": "2026-10-24T21:44:00.281Z",
      "verdict": "fail"
    },
    {
      "id": 126,
      "label": "before invalid",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "before": "99:99"
      },
      "expected": "Reject invalid time",
      "follow": "Echoed params",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.475471,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.475471,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 14696,
      "at": "2026-10-24T21:44:03.649Z",
      "verdict": "fail"
    },
    {
      "id": 127,
      "label": "whitespace topics trim",
      "tool": "talks_by_topic",
      "args": {
        "topics": " mcp-auth , oauth "
      },
      "expected": "Trim slug boundaries or reject cleanly",
      "follow": "No topic mutation",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \" mcp-auth , oauth \"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.347723,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": " mcp-auth , oauth "
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.347723,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 18096,
      "at": "2026-10-24T21:44:07.049Z",
      "verdict": "fail"
    },
    {
      "id": 128,
      "label": "R1 conjunctive date time excludes",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2026-10-28",
        "after": "10:30",
        "before": "10:31"
      },
      "expected": "Zero rows; all filters applied",
      "follow": "R2 matching interval",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"10:30\",\n    \"before\": \"10:31\",\n    \"day\": \"2026-10-28\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.185115,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "10:30",
          "before": "10:31",
          "day": "2026-10-28",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.185115,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 2771,
      "at": "2026-10-24T21:44:28.786Z",
      "verdict": "pass"
    },
    {
      "id": 129,
      "label": "R1 equal bounds",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "after": "10:30",
        "before": "10:30"
      },
      "expected": "Empty half-open interval",
      "follow": "R2 one-minute interval",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"10:30\",\n    \"before\": \"10:30\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.224525,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "10:30",
          "before": "10:30",
          "day": "",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.224525,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 6206,
      "at": "2026-10-24T21:44:32.221Z",
      "verdict": "pass"
    },
    {
      "id": 130,
      "label": "R1 topic union dedupe",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth,oauth"
      },
      "expected": "Two topic rows, same one talk",
      "follow": "R2 compare related talks unique set",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-auth,oauth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.419058,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    },\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"OAuth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:oauth\",\n        \"label\": \"Topic\",\n        \"name\": \"OAuth\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:oauth\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-auth,oauth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.419058,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          },
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "OAuth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            },
            {
              "uid": "topic:oauth",
              "label": "Topic",
              "name": "OAuth",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:oauth"
            }
          ]
        }
      },
      "wall_ms": 9043,
      "at": "2026-10-24T21:44:35.058Z",
      "verdict": "pass"
    },
    {
      "id": 131,
      "label": "R1 missing memory required",
      "tool": "memory_recall",
      "args": {},
      "expected": "INVALID_ARGUMENT",
      "follow": "R2 malformed nonempty uid",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "failed to deserialize parameters: missing field `issue`"
          }
        ],
        "structuredContent": {
          "error_code": "INVALID_ARGUMENT"
        },
        "isError": true
      },
      "data": {
        "error_code": "INVALID_ARGUMENT"
      },
      "wall_ms": 12622,
      "at": "2026-10-24T21:44:38.637Z",
      "verdict": "pass"
    },
    {
      "id": 132,
      "label": "R1 missing recall required",
      "tool": "recall_learnings",
      "args": {},
      "expected": "INVALID_ARGUMENT",
      "follow": "R2 unknown prefixed person",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "failed to deserialize parameters: missing field `person`"
          }
        ],
        "structuredContent": {
          "error_code": "INVALID_ARGUMENT"
        },
        "isError": true
      },
      "data": {
        "error_code": "INVALID_ARGUMENT"
      },
      "wall_ms": 15363,
      "at": "2026-10-24T21:44:41.378Z",
      "verdict": "pass"
    },
    {
      "id": 133,
      "label": "R1 missing people required",
      "tool": "people_to_meet",
      "args": {},
      "expected": "INVALID_ARGUMENT",
      "follow": "R2 correct alpha uid",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "failed to deserialize parameters: missing field `person`"
          }
        ],
        "structuredContent": {
          "error_code": "INVALID_ARGUMENT"
        },
        "isError": true
      },
      "data": {
        "error_code": "INVALID_ARGUMENT"
      },
      "wall_ms": 18507,
      "at": "2026-10-24T21:44:44.522Z",
      "verdict": "pass"
    },
    {
      "id": 134,
      "label": "R1 missing topics required",
      "tool": "talks_by_topic",
      "args": {},
      "expected": "INVALID_ARGUMENT",
      "follow": "R2 invalid topic",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "failed to deserialize parameters: missing field `topics`"
          }
        ],
        "structuredContent": {
          "error_code": "INVALID_ARGUMENT"
        },
        "isError": true
      },
      "data": {
        "error_code": "INVALID_ARGUMENT"
      },
      "wall_ms": 22134,
      "at": "2026-10-24T21:44:48.149Z",
      "verdict": "pass"
    },
    {
      "id": 135,
      "label": "R1 injection read inert",
      "tool": "talks_for_issue",
      "args": {
        "issue": "mcp-test: ' MATCH (n) RETURN n //"
      },
      "expected": "No rows; must not execute Cypher; existing missing-UID validation defect",
      "follow": "R2 presenter graph intact",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T2\",\n  \"params\": {\n    \"issue\": \"mcp-test: ' MATCH (n) RETURN n //\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.216217,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T2",
        "params": {
          "issue": "mcp-test: ' MATCH (n) RETURN n //"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.uid AS talk_uid, k.title AS talk, t.slug AS topic, s.name AS speaker LIMIT 2",
        "hops": 0,
        "run_time_ms": 0.216217,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 25656,
      "at": "2026-10-24T21:44:51.671Z",
      "verdict": "fail"
    },
    {
      "id": 136,
      "label": "R2 matching interval",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-auth",
        "day": "2026-10-29",
        "after": "10:30",
        "before": "10:31"
      },
      "expected": "One talk at 10:30",
      "follow": "Closure: exact bounds checked",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"10:30\",\n    \"before\": \"10:31\",\n    \"day\": \"2026-10-29\",\n    \"topics\": \"mcp-auth\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 2,\n  \"run_time_ms\": 0.44466,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-29\",\n      \"hops\": 2,\n      \"room\": \"Demo Stage (Festival Pavilion)\",\n      \"speaker\": \"Nick Taylor\",\n      \"start_time\": \"10:30\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "10:30",
          "before": "10:31",
          "day": "2026-10-29",
          "topics": "mcp-auth"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 2,
        "run_time_ms": 0.44466,
        "rows": [
          {
            "day": "2026-10-29",
            "hops": 2,
            "room": "Demo Stage (Festival Pavilion)",
            "speaker": "Nick Taylor",
            "start_time": "10:30",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 3154,
      "at": "2026-10-24T21:45:09.956Z",
      "verdict": "pass"
    },
    {
      "id": 137,
      "label": "R2 missing uid confirmation",
      "tool": "memory_recall",
      "args": {
        "issue": "issue:notes/mcp-test-missing#none"
      },
      "expected": "Known defect: empty success instead of nonexistent uid error",
      "follow": "Closure: missing-uid defect reproduced",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T4\",\n  \"params\": {\n    \"issue\": \"issue:notes/mcp-test-missing#none\"\n  },\n  \"cypher\": \"MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.159916,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T4",
        "params": {
          "issue": "issue:notes/mcp-test-missing#none"
        },
        "cypher": "MATCH p = (i:Issue {uid: $issue})-[:ABOUT]->(t:Topic)<-[:DISCUSSED]-(e:Encounter)-[:MET]->(c:Person) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, c.uid AS contact_uid, c.name AS contact, e.note AS note, e.place AS place, e.met_at AS met_at, t.name AS topic ORDER BY met_at DESC LIMIT 3",
        "hops": 0,
        "run_time_ms": 0.159916,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 6629,
      "at": "2026-10-24T21:45:13.431Z",
      "verdict": "fail"
    },
    {
      "id": 138,
      "label": "R2 isolated beta learnings",
      "tool": "recall_learnings",
      "args": {
        "person": "mcp-test-beta"
      },
      "expected": "Exactly own 2 duplicate records",
      "follow": "Closure: ownership holds",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"learnings\": [\n    {\n      \"learned_at\": \"2026-10-24T21:38:16.715559+00:00\",\n      \"source\": \"mcp-test-duplicate\",\n      \"text\": \"mcp-test: identical lesson.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2\"\n    },\n    {\n      \"learned_at\": \"2026-10-24T21:38:13.219946+00:00\",\n      \"source\": \"mcp-test-duplicate\",\n      \"text\": \"mcp-test: identical lesson.\",\n      \"topics\": [\n        \"mcp-auth\"\n      ],\n      \"uid\": \"learning:05362c1e-7200-494e-9185-f8eabf1bcd25\"\n    }\n  ],\n  \"person_uid\": \"person:gh:mcp-test-beta\"\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "learnings": [
          {
            "learned_at": "2026-10-24T21:38:16.715559+00:00",
            "source": "mcp-test-duplicate",
            "text": "mcp-test: identical lesson.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:9ff2a712-e424-4fdd-9660-7be15b15b8a2"
          },
          {
            "learned_at": "2026-10-24T21:38:13.219946+00:00",
            "source": "mcp-test-duplicate",
            "text": "mcp-test: identical lesson.",
            "topics": [
              "mcp-auth"
            ],
            "uid": "learning:05362c1e-7200-494e-9185-f8eabf1bcd25"
          }
        ],
        "person_uid": "person:gh:mcp-test-beta"
      },
      "wall_ms": 9620,
      "at": "2026-10-24T21:45:16.422Z",
      "verdict": "pass"
    },
    {
      "id": 139,
      "label": "R2 alpha speakers",
      "tool": "people_to_meet",
      "args": {
        "person": "person:gh:mcp-test-alpha"
      },
      "expected": "Own issue path; no test contact listed as speaker",
      "follow": "Closure: speaker query unchanged",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T3\",\n  \"params\": {\n    \"limit\": 5,\n    \"person\": \"person:gh:mcp-test-alpha\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.413735,\n  \"rows\": [\n    {\n      \"company\": \"Pomerium\",\n      \"hops\": 5,\n      \"person\": \"Nick Taylor\",\n      \"talk\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n      \"topic\": \"MCP Auth\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:mcp-test-alpha\",\n        \"label\": \"Person\",\n        \"name\": \"mcp-test-alpha\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"repo:notes/mcp-test-alpha\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"label\": \"Issue\",\n        \"name\": \"mcp-test: MCP auth bridge timing\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:mcp-auth\",\n        \"label\": \"Topic\",\n        \"name\": \"MCP Auth\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-09\",\n        \"label\": \"Talk\",\n        \"name\": \"Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:nick-taylor\",\n        \"label\": \"Person\",\n        \"name\": \"Nick Taylor\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:mcp-test-alpha\",\n        \"dst_uid\": \"repo:notes/mcp-test-alpha\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:notes/mcp-test-alpha\",\n        \"dst_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:notes/mcp-test-alpha#7c136aa7\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-09\",\n        \"dst_uid\": \"topic:mcp-auth\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:nick-taylor\",\n        \"dst_uid\": \"talk:U26-09\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T3",
        "params": {
          "limit": 5,
          "person": "person:gh:mcp-test-alpha"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE NOT (me)-[:LOGGED]->(:Encounter)-[:MET]->(s) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, s.name AS person, s.company AS company, t.name AS topic, k.title AS talk LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.413735,
        "rows": [
          {
            "company": "Pomerium",
            "hops": 5,
            "person": "Nick Taylor",
            "talk": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
            "topic": "MCP Auth"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:mcp-test-alpha",
              "label": "Person",
              "name": "mcp-test-alpha",
              "state": ""
            },
            {
              "uid": "repo:notes/mcp-test-alpha",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "label": "Issue",
              "name": "mcp-test: MCP auth bridge timing",
              "state": "LINKED"
            },
            {
              "uid": "topic:mcp-auth",
              "label": "Topic",
              "name": "MCP Auth",
              "state": ""
            },
            {
              "uid": "talk:U26-09",
              "label": "Talk",
              "name": "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers",
              "state": ""
            },
            {
              "uid": "person:name:nick-taylor",
              "label": "Person",
              "name": "Nick Taylor",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:mcp-test-alpha",
              "dst_uid": "repo:notes/mcp-test-alpha"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:notes/mcp-test-alpha",
              "dst_uid": "issue:notes/mcp-test-alpha#7c136aa7"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:notes/mcp-test-alpha#7c136aa7",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-09",
              "dst_uid": "topic:mcp-auth"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:nick-taylor",
              "dst_uid": "talk:U26-09"
            }
          ]
        }
      },
      "wall_ms": 12524,
      "at": "2026-10-24T21:45:19.326Z",
      "verdict": "pass"
    },
    {
      "id": 140,
      "label": "R2 unknown topic confirmation",
      "tool": "talks_by_topic",
      "args": {
        "topics": "mcp-test-unknown"
      },
      "expected": "Known defect: empty success",
      "follow": "Closure: unknown-slug defect reproduced",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T6\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"topics\": \"mcp-test-unknown\"\n  },\n  \"cypher\": \"MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 0,\n  \"run_time_ms\": 0.329764,\n  \"rows\": [],\n  \"path\": {\n    \"nodes\": [],\n    \"edges\": []\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T6",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "topics": "mcp-test-unknown"
        },
        "cypher": "MATCH p = (t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE t.slug IN split($topics, ',') AND ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker, t.name AS topic ORDER BY day, start_time LIMIT 5",
        "hops": 0,
        "run_time_ms": 0.329764,
        "rows": [],
        "path": {
          "nodes": [],
          "edges": []
        }
      },
      "wall_ms": 15216,
      "at": "2026-10-24T21:45:22.018Z",
      "verdict": "fail"
    },
    {
      "id": 141,
      "label": "R2 presenter intact",
      "tool": "talks_for_my_issues",
      "args": {
        "person": "person:gh:manvendersingh21"
      },
      "expected": "Same baseline rows and 5-hop structure",
      "follow": "Closure: no observed graph loss",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "{\n  \"template\": \"T1\",\n  \"params\": {\n    \"after\": \"\",\n    \"before\": \"\",\n    \"day\": \"\",\n    \"person\": \"person:gh:manvendersingh21\"\n  },\n  \"cypher\": \"MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5\",\n  \"hops\": 5,\n  \"run_time_ms\": 1.325953,\n  \"rows\": [\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Franck Nijhof\",\n      \"start_time\": \"14:10\",\n      \"talk\": \"Code is the easy part: Building Home Assistant in the open\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Greg Ose\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"CI/CD\"\n    },\n    {\n      \"day\": \"2026-10-28\",\n      \"hops\": 5,\n      \"issue\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n      \"speaker\": \"Steve Glass\",\n      \"start_time\": \"14:30\",\n      \"talk\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n      \"topic\": \"GitHub Actions\"\n    }\n  ],\n  \"path\": {\n    \"nodes\": [\n      {\n        \"uid\": \"person:gh:manvendersingh21\",\n        \"label\": \"Person\",\n        \"name\": \"manvendersingh21\",\n        \"state\": \"SYNCED\"\n      },\n      {\n        \"uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"label\": \"Repo\",\n        \"name\": \"\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"label\": \"Issue\",\n        \"name\": \"Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks\",\n        \"state\": \"LINKED\"\n      },\n      {\n        \"uid\": \"topic:cicd\",\n        \"label\": \"Topic\",\n        \"name\": \"CI/CD\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-03\",\n        \"label\": \"Talk\",\n        \"name\": \"Code is the easy part: Building Home Assistant in the open\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:franck-nijhof\",\n        \"label\": \"Person\",\n        \"name\": \"Franck Nijhof\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"topic:github-actions\",\n        \"label\": \"Topic\",\n        \"name\": \"GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"talk:U26-04\",\n        \"label\": \"Talk\",\n        \"name\": \"Disrupting supply chain attacks: A threat framework for GitHub Actions\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:greg-ose\",\n        \"label\": \"Person\",\n        \"name\": \"Greg Ose\",\n        \"state\": \"\"\n      },\n      {\n        \"uid\": \"person:name:steve-glass\",\n        \"label\": \"Person\",\n        \"name\": \"Steve Glass\",\n        \"state\": \"\"\n      }\n    ],\n    \"edges\": [\n      {\n        \"type\": \"MAINTAINS\",\n        \"src_uid\": \"person:gh:manvendersingh21\",\n        \"dst_uid\": \"repo:manvendersingh21/deepcore-n1\"\n      },\n      {\n        \"type\": \"HAS_ISSUE\",\n        \"src_uid\": \"repo:manvendersingh21/deepcore-n1\",\n        \"dst_uid\": \"issue:manvendersingh21/deepcore-n1#3\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-03\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:franck-nijhof\",\n        \"dst_uid\": \"talk:U26-03\"\n      },\n      {\n        \"type\": \"ABOUT\",\n        \"src_uid\": \"issue:manvendersingh21/deepcore-n1#3\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:github-actions\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:greg-ose\",\n        \"dst_uid\": \"talk:U26-04\"\n      },\n      {\n        \"type\": \"COVERS\",\n        \"src_uid\": \"talk:U26-04\",\n        \"dst_uid\": \"topic:cicd\"\n      },\n      {\n        \"type\": \"SPEAKS_AT\",\n        \"src_uid\": \"person:name:steve-glass\",\n        \"dst_uid\": \"talk:U26-04\"\n      }\n    ]\n  }\n}"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": {
        "template": "T1",
        "params": {
          "after": "",
          "before": "",
          "day": "",
          "person": "person:gh:manvendersingh21"
        },
        "cypher": "MATCH p = (me:Person {uid: $person})-[:MAINTAINS|TRACKS]->(r:Repo)-[:HAS_ISSUE]->(i:Issue {gh_state: 'open'})-[:ABOUT]->(t:Topic)<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) WHERE ($day = '' OR k.day = $day) AND ($after = '' OR k.start_time >= $after) AND ($before = '' OR k.start_time < $before) RETURN [n IN nodes(p) | [n.uid, labels(n)[0], coalesce(n.name, n.title, ''), coalesce(n.state, '')]] AS nodes, [r IN relationships(p) | [type(r), startNode(r).uid, endNode(r).uid]] AS edges, size(relationships(p)) AS hops, i.title AS issue, t.name AS topic, k.title AS talk, k.day AS day, k.start_time AS start_time, s.name AS speaker ORDER BY day, start_time LIMIT 5",
        "hops": 5,
        "run_time_ms": 1.325953,
        "rows": [
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Franck Nijhof",
            "start_time": "14:10",
            "talk": "Code is the easy part: Building Home Assistant in the open",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Greg Ose",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "CI/CD"
          },
          {
            "day": "2026-10-28",
            "hops": 5,
            "issue": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
            "speaker": "Steve Glass",
            "start_time": "14:30",
            "talk": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
            "topic": "GitHub Actions"
          }
        ],
        "path": {
          "nodes": [
            {
              "uid": "person:gh:manvendersingh21",
              "label": "Person",
              "name": "manvendersingh21",
              "state": "SYNCED"
            },
            {
              "uid": "repo:manvendersingh21/deepcore-n1",
              "label": "Repo",
              "name": "",
              "state": ""
            },
            {
              "uid": "issue:manvendersingh21/deepcore-n1#3",
              "label": "Issue",
              "name": "Lock down GitHub Actions runner workflow permissions and npm dependency supply-chain checks",
              "state": "LINKED"
            },
            {
              "uid": "topic:cicd",
              "label": "Topic",
              "name": "CI/CD",
              "state": ""
            },
            {
              "uid": "talk:U26-03",
              "label": "Talk",
              "name": "Code is the easy part: Building Home Assistant in the open",
              "state": ""
            },
            {
              "uid": "person:name:franck-nijhof",
              "label": "Person",
              "name": "Franck Nijhof",
              "state": ""
            },
            {
              "uid": "topic:github-actions",
              "label": "Topic",
              "name": "GitHub Actions",
              "state": ""
            },
            {
              "uid": "talk:U26-04",
              "label": "Talk",
              "name": "Disrupting supply chain attacks: A threat framework for GitHub Actions",
              "state": ""
            },
            {
              "uid": "person:name:greg-ose",
              "label": "Person",
              "name": "Greg Ose",
              "state": ""
            },
            {
              "uid": "person:name:steve-glass",
              "label": "Person",
              "name": "Steve Glass",
              "state": ""
            }
          ],
          "edges": [
            {
              "type": "MAINTAINS",
              "src_uid": "person:gh:manvendersingh21",
              "dst_uid": "repo:manvendersingh21/deepcore-n1"
            },
            {
              "type": "HAS_ISSUE",
              "src_uid": "repo:manvendersingh21/deepcore-n1",
              "dst_uid": "issue:manvendersingh21/deepcore-n1#3"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-03",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:franck-nijhof",
              "dst_uid": "talk:U26-03"
            },
            {
              "type": "ABOUT",
              "src_uid": "issue:manvendersingh21/deepcore-n1#3",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:github-actions"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:greg-ose",
              "dst_uid": "talk:U26-04"
            },
            {
              "type": "COVERS",
              "src_uid": "talk:U26-04",
              "dst_uid": "topic:cicd"
            },
            {
              "type": "SPEAKS_AT",
              "src_uid": "person:name:steve-glass",
              "dst_uid": "talk:U26-04"
            }
          ]
        }
      },
      "wall_ms": 18227,
      "at": "2026-10-24T21:45:25.029Z",
      "verdict": "pass"
    },
    {
      "id": 142,
      "label": "R2 topics final",
      "tool": "list_topics",
      "args": {},
      "expected": "Exactly original 22 topics",
      "follow": "Closure: no new topics created",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[\n  {\n    \"name\": \"Agent Memory\",\n    \"slug\": \"agent-memory\"\n  },\n  {\n    \"name\": \"Agentic Security\",\n    \"slug\": \"agentic-security\"\n  },\n  {\n    \"name\": \"Agentic Workflows\",\n    \"slug\": \"agentic-workflows\"\n  },\n  {\n    \"name\": \"AI-Assisted Testing\",\n    \"slug\": \"ai-assisted-testing\"\n  },\n  {\n    \"name\": \"AI Context\",\n    \"slug\": \"ai-context\"\n  },\n  {\n    \"name\": \"AI Operations\",\n    \"slug\": \"ai-operations\"\n  },\n  {\n    \"name\": \"CI/CD\",\n    \"slug\": \"cicd\"\n  },\n  {\n    \"name\": \"Community Governance\",\n    \"slug\": \"community-governance\"\n  },\n  {\n    \"name\": \"Copilot\",\n    \"slug\": \"copilot\"\n  },\n  {\n    \"name\": \"Dependencies\",\n    \"slug\": \"dependencies\"\n  },\n  {\n    \"name\": \"Deterministic Verification\",\n    \"slug\": \"deterministic-verification\"\n  },\n  {\n    \"name\": \"Developer Experience\",\n    \"slug\": \"developer-experience\"\n  },\n  {\n    \"name\": \"Enterprise AI Adoption\",\n    \"slug\": \"enterprise-ai-adoption\"\n  },\n  {\n    \"name\": \"Frontier Models\",\n    \"slug\": \"frontier-models\"\n  },\n  {\n    \"name\": \"GitHub Actions\",\n    \"slug\": \"github-actions\"\n  },\n  {\n    \"name\": \"MCP Auth\",\n    \"slug\": \"mcp-auth\"\n  },\n  {\n    \"name\": \"OAuth\",\n    \"slug\": \"oauth\"\n  },\n  {\n    \"name\": \"Open Source\",\n    \"slug\": \"open-source\"\n  },\n  {\n    \"name\": \"Package Management\",\n    \"slug\": \"package-management\"\n  },\n  {\n    \"name\": \"Platform Engineering\",\n    \"slug\": \"platform-engineering\"\n  },\n  {\n    \"name\": \"RCA Agent\",\n    \"slug\": \"rca-agent\"\n  },\n  {\n    \"name\": \"Supply Chain Security\",\n    \"slug\": \"supply-chain-security\"\n  }\n]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": [
        {
          "name": "Agent Memory",
          "slug": "agent-memory"
        },
        {
          "name": "Agentic Security",
          "slug": "agentic-security"
        },
        {
          "name": "Agentic Workflows",
          "slug": "agentic-workflows"
        },
        {
          "name": "AI-Assisted Testing",
          "slug": "ai-assisted-testing"
        },
        {
          "name": "AI Context",
          "slug": "ai-context"
        },
        {
          "name": "AI Operations",
          "slug": "ai-operations"
        },
        {
          "name": "CI/CD",
          "slug": "cicd"
        },
        {
          "name": "Community Governance",
          "slug": "community-governance"
        },
        {
          "name": "Copilot",
          "slug": "copilot"
        },
        {
          "name": "Dependencies",
          "slug": "dependencies"
        },
        {
          "name": "Deterministic Verification",
          "slug": "deterministic-verification"
        },
        {
          "name": "Developer Experience",
          "slug": "developer-experience"
        },
        {
          "name": "Enterprise AI Adoption",
          "slug": "enterprise-ai-adoption"
        },
        {
          "name": "Frontier Models",
          "slug": "frontier-models"
        },
        {
          "name": "GitHub Actions",
          "slug": "github-actions"
        },
        {
          "name": "MCP Auth",
          "slug": "mcp-auth"
        },
        {
          "name": "OAuth",
          "slug": "oauth"
        },
        {
          "name": "Open Source",
          "slug": "open-source"
        },
        {
          "name": "Package Management",
          "slug": "package-management"
        },
        {
          "name": "Platform Engineering",
          "slug": "platform-engineering"
        },
        {
          "name": "RCA Agent",
          "slug": "rca-agent"
        },
        {
          "name": "Supply Chain Security",
          "slug": "supply-chain-security"
        }
      ],
      "wall_ms": 21819,
      "at": "2026-10-24T21:45:28.621Z",
      "verdict": "pass"
    },
    {
      "id": 143,
      "label": "R2 alerts final",
      "tool": "check_alerts",
      "args": {},
      "expected": "Only CONTACT records; inventory auto-created bridges",
      "follow": "Closure: record observed bridge ids",
      "result": {
        "content": [
          {
            "type": "text",
            "text": "[\n  {\n    \"bridge_uid\": \"bridge:d20f4fac-77b0-4482-8757-ed74dbf4208c\",\n    \"contact\": \"mcp-test-long-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Standardize shared agent memory & context infrastructure across local and cloud runtimes\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#4\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"Agent Memory\"\n  },\n  {\n    \"bridge_uid\": \"bridge:483f0dfa-7642-445f-a35e-538166eceef8\",\n    \"contact\": \"mcp-test-payload-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:e5ad65fd-72b5-46b0-abc5-80f3fa12f158\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"3\",\n    \"issue\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n    \"issue_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:34be0887-cf6b-4dfa-9c3f-c0f60f756d83\",\n    \"contact\": \"mcp-test-beta-private\",\n    \"hops\": \"3\",\n    \"issue\": \"mcp-test: नमस्ते 🧵 café é \\\"quote\\\" 'single' `MATCH (n) DETACH DELETE n`\",\n    \"issue_uid\": \"issue:notes/mcp-test-payload#2c4df3e7\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:9cf26a79-4ad7-470a-8cec-b5f2454dd6f5\",\n    \"contact\": \"mcp-test-beta-private\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:1a81cd1e-a24c-4723-8c2d-6b7bbacff29b\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"3\",\n    \"issue\": \"mcp-test: beta-only auth issue\",\n    \"issue_uid\": \"issue:notes/mcp-test-beta#f6cc6fcd\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  },\n  {\n    \"bridge_uid\": \"bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd\",\n    \"contact\": \"mcp-test-contact\",\n    \"hops\": \"4\",\n    \"issue\": \"Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)\",\n    \"issue_uid\": \"issue:manvendersingh21/deepcore-n1#1\",\n    \"state\": \"ACTION_READY\",\n    \"topic\": \"MCP Auth\"\n  }\n]"
          }
        ],
        "structuredContent": null,
        "isError": false
      },
      "data": [
        {
          "bridge_uid": "bridge:d20f4fac-77b0-4482-8757-ed74dbf4208c",
          "contact": "mcp-test-long-contact",
          "hops": "4",
          "issue": "Standardize shared agent memory & context infrastructure across local and cloud runtimes",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#4",
          "state": "ACTION_READY",
          "topic": "Agent Memory"
        },
        {
          "bridge_uid": "bridge:483f0dfa-7642-445f-a35e-538166eceef8",
          "contact": "mcp-test-payload-contact",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:e5ad65fd-72b5-46b0-abc5-80f3fa12f158",
          "contact": "mcp-test-contact",
          "hops": "3",
          "issue": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
          "issue_uid": "issue:notes/mcp-test-payload#2c4df3e7",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:34be0887-cf6b-4dfa-9c3f-c0f60f756d83",
          "contact": "mcp-test-beta-private",
          "hops": "3",
          "issue": "mcp-test: नमस्ते 🧵 café é \"quote\" 'single' `MATCH (n) DETACH DELETE n`",
          "issue_uid": "issue:notes/mcp-test-payload#2c4df3e7",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:9cf26a79-4ad7-470a-8cec-b5f2454dd6f5",
          "contact": "mcp-test-beta-private",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:1a81cd1e-a24c-4723-8c2d-6b7bbacff29b",
          "contact": "mcp-test-contact",
          "hops": "3",
          "issue": "mcp-test: beta-only auth issue",
          "issue_uid": "issue:notes/mcp-test-beta#f6cc6fcd",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        },
        {
          "bridge_uid": "bridge:dfaaeaac-cf9a-451e-9888-61780a7e95dd",
          "contact": "mcp-test-contact",
          "hops": "4",
          "issue": "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)",
          "issue_uid": "issue:manvendersingh21/deepcore-n1#1",
          "state": "ACTION_READY",
          "topic": "MCP Auth"
        }
      ],
      "wall_ms": 25670,
      "at": "2026-10-24T21:45:32.472Z",
      "verdict": "fail"
    }
  ],
  "tool_inventory": [
    {
      "name": "mcp__codex_apps__patchwork_check_alerts",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nBridges PatchWork found between your open issues and people you met (newest first). Draft any follow-up message with your own model. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_check_alerts(args: { [key: string]: unknown; }): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_list_topics",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nThe closed topic list (slug and name). Use these slugs for talks_by_topic and log_encounter. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_list_topics(args: { [key: string]: unknown; }): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_log_encounter",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nRecord a hallway conversation. Your own model extracts who was met and which topics (slugs from list_topics) were discussed; the server stores it and PatchWork's graph agents link it to open issues. No LLM runs on the server. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_log_encounter(args: {\n  // Their GitHub handle, if the user knows it. Leave empty otherwise; never guess.\n  met_handle?: string;\n  // Name of the person met.\n  met_name: string;\n  // The note as the user wrote it.\n  note: string;\n  // Person uid of the user logging the conversation, e.g. person:gh:<handle>.\n  person: string;\n  // Where it happened, e.g. \"snack table\".\n  place?: string;\n  // Topic slugs discussed, chosen from list_topics. Unknown slugs are rejected.\n  topics: Array<string>;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_log_issue",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nLog an open problem or question the user describes (verbally or from a photo of notes/a slide). It becomes an open issue on their personal notes repo in the graph, auto-classified under the given topics; the agents then find talks and people that address it. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_log_issue(args: {\n  // The full question or problem as the user described it.\n  body: string;\n  // Whose notes repo this lands on: GitHub handle or plain name.\n  person: string;\n  // One-line summary of the open problem or question.\n  title: string;\n  // Topic slugs from list_topics. Unknown slugs are rejected.\n  topics: Array<string>;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_log_learning",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nLog something the user learned at the conference. Works from a sentence the user says OR a photo they share (slide, whiteboard, poster, badge, notes): read the photo, extract the key lesson into `learned`, pick topic slugs with list_topics, and describe where it came from in `source`. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_log_learning(args: {\n  // The lesson itself, one or two sentences the user wants to keep.\n  learned: string;\n  // Who learned it: their GitHub handle (e.g. \"octocat\") or plain name (e.g. \"Ada Lovelace\").\n  person: string;\n  // Where it came from, e.g. \"photo of a slide\", \"hallway chat\".\n  source?: string;\n  // Topic slugs the lesson is about, chosen from list_topics. Unknown slugs are rejected.\n  topics: Array<string>;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_memory_recall",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nPeople the user met who discussed the topics of one issue (memory recall), newest first. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_memory_recall(args: {\n  // Issue uid, e.g. issue:<owner>/<repo>#<number>.\n  issue: string;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_people_to_meet",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nSpeakers relevant to the person's open issues whom they have not logged meeting yet. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_people_to_meet(args: { person: string; }): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_recall_learnings",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nRecall what the user logged as learned, newest first, optionally filtered to one topic slug from list_topics (empty = all). Summarize these lessons back to the user. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_recall_learnings(args: {\n  // Who to recall for: GitHub handle or plain name.\n  person: string;\n  // Optional topic slug from list_topics; empty for all topics.\n  topic?: string;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_talks_by_topic",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nTalks about given topics, filtered by day and earliest start time, with their path. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_talks_by_topic(args: {\n  after?: string;\n  before?: string;\n  day?: string;\n  // Comma-separated topic slugs, e.g. \"mcp-auth,agentic-security\".\n  topics: string;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_talks_for_issue",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nTalks that cover the topics of one issue, with the path that connects them. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_talks_for_issue(args: {\n  // Issue uid, e.g. issue:<owner>/<repo>#<number>.\n  issue: string;\n}): Promise<CallToolResult>; };\n```"
    },
    {
      "name": "mcp__codex_apps__patchwork_talks_for_my_issues",
      "description": "MCP server instructions: PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.\n\nTalks that cover topics of the person's open issues, as a 5-hop path (Person→Repo→Issue→Topic→Talk→Speaker). Optional day and earliest start time. This tool is part of plugin `patchwork`.\n\nexec tool declaration:\n```ts\ndeclare const tools: { mcp__codex_apps__patchwork_talks_for_my_issues(args: {\n  // Earliest start time HH:MM (24h), or empty for any time.\n  after?: string;\n  // Latest start time HH:MM (24h), or empty for any time.\n  before?: string;\n  // Day filter as YYYY-MM-DD, or empty for any day.\n  day?: string;\n  // Person uid, e.g. person:gh:<handle>. Defaults to the presenter.\n  person: string;\n}): Promise<CallToolResult>; };\n```"
    }
  ],
  "state_timing": [
    {
      "write": 29,
      "poll": 30,
      "issue_uid": "issue:notes/mcp-test-alpha#7c136aa7",
      "response_to_read_ms": 3062,
      "write_start_to_read_ms": 6659,
      "state": "LINKED",
      "rows": 1
    },
    {
      "write": 118,
      "poll": 119,
      "issue_uid": "issue:notes/mcp-test-timing-1#c30e2013",
      "response_to_read_ms": 2968,
      "write_start_to_read_ms": 6111,
      "state": "LINKED",
      "rows": 1
    },
    {
      "write": 120,
      "poll": 121,
      "issue_uid": "issue:notes/mcp-test-timing-2#bc4fd6eb",
      "response_to_read_ms": 2719,
      "write_start_to_read_ms": 5730,
      "state": "LINKED",
      "rows": 1
    }
  ],
  "assertions": [
    [
      "All alpha learning text/source/topics byte-for-byte",
      true
    ],
    [
      "Alpha newest first",
      true
    ],
    [
      "Presenter baseline rows preserved",
      true
    ],
    [
      "Topic set preserved",
      true
    ],
    [
      "All template envelopes and edge endpoints valid",
      true
    ]
  ]
}
```

