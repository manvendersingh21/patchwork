# Brief: React frontend against fixtures — opencode / GLM

Read `CLAUDE.md`, then the "Frontend spec" and "Backend API contract" sections of `PLAN.md`.

## You own (create/edit only these)
- everything under `frontend/` (Vite + React + TypeScript + Tailwind + react-force-graph-2d)

## Do not touch
- anything outside `frontend/`. Do NOT run `git commit`.
- Allowed npm deps: react, react-dom, vite, @vitejs/plugin-react, typescript, tailwindcss (+ its vite plugin), react-force-graph-2d. Ask before adding others.

## What to build
1. Vite app; `npm run dev` on :5173; `VITE_MOCK=1` serves `src/mocks/*.json` from `src/api.ts`; otherwise proxies `/api` to `http://localhost:8000`.
2. Create the fixtures first, one per endpoint in the API contract. Use the real seeded data:
   presenter `person:gh:manvendersingh21`; issue `issue:manvendersingh21/deepcore-n1#1` "Enforce fine-grained per-identity OAuth scopes on hosted MCP tool calls (block auto-merge)";
   topics `topic:mcp-auth` "MCP Auth" and `topic:agentic-security` "Agentic Security"; talk `talk:U26-09` "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers" (2026-10-29 10:30, Demo Stage);
   speaker `person:name:nick-taylor` "Nick Taylor". The `ask` fixture is the envelope shape:
   `{template, params, cypher, hops: 5, run_time_ms: 1.04, rows: [...], path: {nodes: [{uid,label,name,state}], edges: [{type,src_uid,dst_uid}]}, answer}`.
   The Act 1 path is Person→MAINTAINS→Repo→HAS_ISSUE→Issue→ABOUT→Topic←COVERS←Talk←SPEAKS_AT←Person.
3. Components per spec: GraphPane, AskPane (chips: "Which talks on day 2 morning help with my open issues?"), PathBadges, GraphProofDrawer, AgentTicker, AlertCard, EncounterBox. 40/60 split, dark, ≥16 px text, 1280×720 with no page scroll.
4. Graph: color by label, size by degree, highlight path (others 15% opacity, particles 4, zoom to fit), amber pulse for in-progress states, update nodes in place by uid every 2 s. Hotkeys Alt+I (inject issue) and Alt+R (replay highlight) only when no input has focus.
5. Never hard-code timings in components; render `run_time_ms` from the response.

## Acceptance (run; paste output in your final message)
- `cd frontend && npm install && npx tsc --noEmit && npm run build`
- `VITE_MOCK=1 npm run dev` serves the page and the ask fixture highlights its path (describe what you verified).
