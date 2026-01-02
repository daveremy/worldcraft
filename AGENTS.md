# Worldcraft Agent Guide

This repo is optimized for AI-assisted development. Read this file first.

## Project Goal

Worldcraft builds a world-model-first streaming intelligence platform. The short-term goal is a reliable 2-minute demo that discovers entities/relationships/signals from a Kafka stream with explainable evidence and a lightweight UI.

## Critical Assumptions (v1 Demo)

- Timely/Differential Dataflow is required for the demo (timeboxed POC in Phase 0).
- Ingestion-time ordering (no event-time watermarking).
- Storage: in-memory HashMap for hot state + SQLite for durable snapshots.
- Confidence scoring and evidence schema are mandatory for every hypothesis.

## Demo Data

- Seed events: `data/demo-events.json`
- Expected labels: `data/expected-discoveries.json`
- Demo plan: `docs/demo-plan.md`

## Core Docs

- Product plan: `worldcraft_product_plan.md`
- Schemas: `docs/schemas.md`
- Architecture decisions: `docs/architecture-decisions.md`
- Dev plan: `docs/dev-plan.md`
- DD learning guide: `docs/dd-learning-guide.md`

## Repo Layout (planned)

```
apps/
  dashboard/
crates/
  ingest/
  discovery/
  model/
  api/
infra/
  docker/
data/
docs/
```

## Build & Run (planned commands)

- `make dev` or `just dev`: start API + dashboard
- `make ingest`: run ingestion + discovery
- `make demo`: replay demo events into Kafka
- `make test`: unit + discovery harness
- `make eval`: evaluate discovery output against expected discoveries
- `make dd_poc`: run the Timely/DD proof-of-concept
- `make dd_eval`: evaluate DD POC output against expected discoveries
- `make sqlite_smoke`: verify SQLite snapshot read/write

If these commands are missing, add minimal Rust tools aligned with the dev plan.

## Development Priorities

1. De-risk discovery quality with the demo dataset and evaluation harness.
2. Keep the demo loop fast: stream -> discovery -> UI update.
3. Prefer clarity and explainability over clever optimization.

## Dependency Policy

- Use the latest stable versions of languages, tools, and libraries unless explicitly constrained.
- All tooling and automation should be written in Rust unless explicitly approved otherwise.

## Evidence + Confidence

- Evidence schema and confidence formula are defined in `docs/schemas.md`.
- Evidence compaction: keep sample events for discovery, compact after approval.
- Unknown/low-confidence fields must not pollute the main entity list.

## Acceptance Targets (Phase 1)

- Discovers 5/5 expected entities within 10s.
- Confidence: 4/5 >0.70, 1/5 >0.60.
- False positives <20% (max 1 junk entity shown).
- UI updates within 1s.

## Testing Expectations

- Add unit tests for scoring, heuristics, and evidence compaction.
- Add discovery harness that reads `data/expected-discoveries.json`.

## Change Discipline

- Keep changes small and reviewable.
- Update docs when behavior or assumptions change.
- If a change conflicts with the demo plan, flag it explicitly.
