# Brief: demo script and data check — cursor-agent

Read `CLAUDE.md` and the "Stage demo script", "Storyline texts" and "Seed data plan" sections of `PLAN.md`.

## You own
- `demo/script.md` (create)
- `data/` — read-only. Report any problem; do not edit.

## Do not touch
- anything else. Do NOT run `git commit`. Do not print `.env` values.

## Do
1. Write `demo/script.md` with:
   - the three storyline texts (Act 1 issue, Act 2 hallway note, Act 3 issue) — use the real repo's issue #1 text for Act 1; the Act 2 note must mention a real teammate or be marked as a placeholder to fill in
   - the three acts with timings from PLAN.md, and each act's fallback
   - the Act 1 chip question: "Which talks on day 2 morning help with my open issues?" (day 2 = 2026-10-29, 10:00 onward)
   - the list of real vs staged items (what is real: sessions, repo, issues, people; what is staged: the restart)
2. Check `data/sessions.json` against the rules in PLAN.md ("never invent a day, time or room"). List any session with a missing field.
3. Keep it short: under 150 lines in `demo/script.md`.

## Acceptance (paste the output in your final message)
- `python3 -c "import json;json.load(open('data/sessions.json'))"` exits 0
- `grep -c "^## " demo/script.md` shows at least 4 sections
