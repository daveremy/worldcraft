# Architecture Decisions

## ADR-0001: Differential Dataflow for Incremental Computation

- Status: Accepted (strategic)
- Date: 2025-01-01

### Context
Worldcraft's core differentiator is "incremental by construction" - maintaining a continuously updated world model without batch recomputation. Stakeholders must see incremental updates in the demo.

### Decision
Use Timely/Differential Dataflow (DD) as the core data plane for v1.

### Rationale
- Demonstrates incremental updates for approvals, new events, and renames.
- Aligns directly with the value prop: "what is true right now" without recomputation.
- Enables efficient downstream updates when entity definitions change.

### Implementation Notes
- POC scaffold lives in `crates/dd_poc` and should be extended to parse demo events.
- Run via `make dd_poc` (writes `out/dd_discovery.json` and `out/dd_updates.json`).
- POC demonstrates approval, incremental event, and rename updates via `out/dd_updates.json`.

### Consequences
- Phase 1 extended to 3 weeks to account for DD learning curve.
- Checkpoint/restore is mandatory for restart behavior.
- Requires team DD fluency and debugging readiness.

### Risks & Mitigations
- Risk: DD learning curve blocks progress.
  Mitigation: Phase 0 complexity test (relationship inference + approval recompute).
- Risk: Checkpoint/restore bugs.
  Mitigation: Test restart behavior in Phase 0 with demo data.
- Risk: Performance issues.
  Mitigation: Validate 1000 events/sec on demo data before Phase 1.

### Timebox + Fallback
- DD POC must demonstrate incremental updates on the demo dataset within 3 days.
- If checkpoint/restore or incremental updates are not working by end of Phase 0, use a plain Rust fallback for the demo and defer DD to v2.
- Record the outcome at the end of Phase 0.
