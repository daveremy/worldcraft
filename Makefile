.PHONY: fmt lint test dev ingest demo prototype eval dd_poc dd_eval sqlite_smoke

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all

dev:
	@echo "TODO: start api + dashboard"

ingest:
	@echo "TODO: run ingestion + discovery"

demo:
	@echo "Emit demo events to stdout (pipe to kcat)."
	cargo run -p worldcraft-ingest --bin emit_demo

prototype:
	cargo run -p worldcraft-discovery --bin prototype -- --output out/discovery.json --metrics-out out/discovery_metrics.json

eval:
	cargo run -p worldcraft-discovery --bin eval -- --expected data/expected-discoveries.json --actual out/discovery.json

dd_poc:
	cargo run -p worldcraft-dd-poc -- --output out/dd_discovery.json --updates-out out/dd_updates.json --metrics-out out/dd_metrics.json

dd_eval:
	cargo run -p worldcraft-discovery --bin eval -- --expected data/expected-discoveries.json --actual out/dd_discovery.json

sqlite_smoke:
	cargo run -p worldcraft-model --bin sqlite_smoke
