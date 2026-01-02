# Differential Dataflow Learning Guide

This guide captures the mental model and working patterns for Timely/Differential Dataflow (DD) in Worldcraft.

## Mental Model

- **Timely** executes dataflow graphs across workers with progress tracking.
- **Differential Dataflow** adds incremental computation over collections.
- **Arrangements** index collections for efficient incremental updates.
- **Frontiers** represent progress in time and drive when results are complete.
- **Capabilities** allow operators to produce data at specific times.

## Core Patterns (Worldcraft)

1. **Entity discovery as incremental aggregation**
   - Stream events -> extract fields -> aggregate stats by field.
   - Maintain stats incrementally (cardinality, recurrence, entropy approximations).

2. **Relationship inference as incremental co-occurrence**
   - Extract pairs within events (order_id -> customer_id).
   - Maintain counts and confidence per pair as events arrive.

3. **Approval-triggered recomputation**
   - Treat approvals as updates to rule streams.
   - Join rules with evidence collections to update confidence.

## Checkpoint/Restore

- **Checkpoint**: snapshot DD state into SQLite (model + evidence + version).
- **Restore**: load latest snapshot, resume Kafka at last committed offset, replay events since snapshot.
- Keep checkpoints lightweight and tied to model versions.

## Debugging Tips

- Start with a single worker (`Config::thread`) for determinism.
- Inspect intermediate streams with `.inspect` or `.probe`.
- Validate frontiers advance to avoid stuck computations.
- Compare DD output with non-DD prototype on demo data.

## Worldcraft-Specific Targets

- Show incremental updates in the demo:
  - Approvals update confidence without recomputation.
  - New events update entity stats and relationships live.
  - Renames propagate across the model.

## References

- Timely Dataflow docs: https://github.com/TimelyDataflow/timely-dataflow
- Differential Dataflow docs: https://github.com/TimelyDataflow/differential-dataflow
