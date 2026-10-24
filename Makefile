# PatchWork — build & demo shortcuts.
# Snapshots live in data/snapshots/. The demo reset is: make restore SNAP=baseline

SNAP ?= baseline
HANDLE ?= $(shell grep -E '^PRESENTER_HANDLE=' .env | cut -d= -f2 | tr -d '"')

.PHONY: api ui build test seed smoke export restore synced

api:
	cargo run --bin patchwork

ui:
	cd frontend && npm run dev

build:
	cargo build

test:
	cargo test

seed:
	cargo run --bin seed

smoke:
	cargo run --bin smoke

export:
	cargo run --bin snapshot -- export data/snapshots/$(SNAP).json

restore:
	cargo run --bin snapshot -- restore data/snapshots/$(SNAP).json

# Rehearse Act 1 and save the post-sync snapshot (data/snapshots/synced.json).
synced:
	$(MAKE) restore SNAP=baseline
	curl -s -X POST http://127.0.0.1:8000/api/sync -H 'Content-Type: application/json' -d '{"handle":"$(HANDLE)","kind":"self"}'
	@ok=0; \
	for i in $$(seq 1 30); do \
		if curl -s 'http://127.0.0.1:8000/api/graph?scope=around&uid=person:gh:$(HANDLE)' | grep -q '"state":"SYNCED"'; then \
			echo "presenter node state=SYNCED (poll $$i)"; ok=1; break; \
		fi; \
		sleep 2; \
	done; \
	test $$ok -eq 1 || { echo 'presenter node never reached SYNCED after ~60s' >&2; exit 1; }
	$(MAKE) export SNAP=synced
	@echo 'Rehearsal snapshot saved. Demo reset is: make restore SNAP=baseline'
