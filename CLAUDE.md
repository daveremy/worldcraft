# Claude Guide

Use this file when working in this repo with Claude or Claude-based agents.

## What to Optimize For

- Ship the 2-minute demo with explainable discovery.
- Minimize risk by validating heuristics on the demo dataset early.
- Keep the system simple and inspectable.

## How to Work

- Read `AGENTS.md` first, then the product plan and demo plan.
- Prefer small, verifiable changes that keep the demo loop working.
- If you change discovery heuristics, update evidence and confidence tests.
- If a task impacts the demo script, update `docs/demo-plan.md`.

## Key Constraints

- Timely/Differential Dataflow is a required assumption for the demo.
- Ingestion-time ordering only (no watermarking).
- SQLite snapshots + in-memory hot state.
- Evidence compaction after approval.

## When in Doubt

- Ask for clarification on demo success criteria and dataset specifics.
- Propose the smallest change that reduces risk.
- Prefer explicit metrics over subjective judgments.

## Dependency Policy

- Use the latest stable versions of languages, tools, and libraries unless explicitly constrained.
- All tooling and automation should be written in Rust unless explicitly approved otherwise.
