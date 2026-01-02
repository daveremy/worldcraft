# Worldcraft Dev Plan

## Goals

- Establish project infrastructure for rapid iteration and reproducible demos.
- Provide a minimal, reliable developer experience for Rust + dashboard + data tooling.
- Keep the stack lean and aligned with the demo scope.

## Repo Structure

```
/ (repo)
  apps/
    dashboard/        # web UI
  crates/
    ingest/           # Kafka + normalization
    discovery/        # heuristics + scoring
    model/            # core types + storage
    api/              # HTTP/WebSocket API
  data/               # demo dataset + expected discoveries
  docs/
  infra/
    docker/           # compose files
```

## Tooling + Conventions

- Rust stable, workspace-managed crates
- Frontend: minimal React/Vite (or equivalent) with TypeScript
- Formatting: rustfmt, clippy, eslint/prettier
- Linting: clippy (deny warnings in CI), eslint
- Testing: unit tests for discovery + integration smoke tests for API
- Versioning: semantic versioning for tagged releases
- Tooling: all automation and harnesses implemented in Rust

## CI (GitHub Actions)

- Build + test (Rust) on push and PR
- Lint (clippy + rustfmt) on push and PR
- Frontend build + lint
- Optional: discovery harness run against `data/expected-discoveries.json`

## Local Dev Bootstrap

- `make dev` or `just dev` to start API + dashboard
- `make ingest` to run Kafka consumer + discovery
- `make demo` to replay `data/demo-events.json` into Kafka
- `make test` for unit + harness tests
- `make dd_poc` to run the Timely/DD proof-of-concept
- `make eval` to evaluate discovery output against expected discoveries
- `make sqlite_smoke` to verify SQLite snapshot read/write

## Infra Setup (Phase 0)

- Initialize git repo and GitHub project
- Add README, CONTRIBUTING, CODEOWNERS, LICENSE (if needed)
- Add `docs/architecture-decisions.md`
- Add `AGENTS.md` and `CLAUDE.md` for AI agent guidance
- Add `docs/dd-learning-guide.md`
- Add `.editorconfig`, `.gitignore`, and basic repo lint rules
- Add Docker Compose for Kafka + schema registry (optional)
- Add local seed/replay Rust tools

## Data + Evaluation

- `data/demo-events.json` (seed events)
- `data/expected-discoveries.json` (labels)
- Harness: report precision/recall + time-to-discovery
- Metrics: track in `docs/discovery-metrics.md`

## Observability (v1)

- Structured logging (JSON logs)
- Basic latency metrics: discovery time, UI update latency
- Optional Prometheus endpoints

## Milestone Integration

- Phase 0: infra scaffolding + dev scripts + CI green
- Phase 1: Kafka ingestion + DD discovery loop + dashboard stub (3 weeks)
- Phase 2: relationships + signal + evidence UI
- Phase 3: demo readiness + performance tuning
