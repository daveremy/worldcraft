# Worldcraft - World-Model-First Streaming Intelligence Platform

## Identity Stack

- **Worldcraft** - the platform
- **World Model** - continuously evolving entity/relationship/state graph
- **Worldcraft Discovery** - entity, relationship, and signal discovery layer
- **Worldcraft Signals** - stream of detected "interesting changes"
- **Worldcraft Context** - agent- and human-ready entity snapshots

---

## 1. Vision

Build a system that can **walk up to any event stream**, begin consuming it, and **autonomously construct a continuously updated world model** composed of:
- discovered entities
- relationships between them
- attributes and metrics
- detected signals and changes

The world model is **incremental, explainable, queryable, and agent-ready**.

Prediction is *downstream* - the primary product is **stateful understanding**.

---

## 2. Core Value Proposition

Most streaming systems answer:
> "What happened?"

Worldcraft answers:
> **"What is true right now - and why?"**

Key differentiators:
- Zero / low configuration onboarding
- Continuous discovery, not static schemas
- Entity-centric, not event-centric
- Designed for agents *and* humans
- Incremental by construction (Timely/Differential Dataflow)

---

## 3. Non-Goals (v1)

To maintain focus:
- No end-user ML training pipelines
- No bespoke domain ontologies
- No full natural language querying (initially)
- No heavy visualization polish
- No relationship cardinality detection (v1 only shows relationships exist)
- No multiple signal types (v1 has one change-point detector)
- No embeddings in the entity schema
- No complex approval workflows (single approve action)
- No event-time watermarking (use ingestion time)

---

## 4. v1 Requirements & Constraints

To de-risk the demo and keep scope tight, v1 will include:
- **Event normalization contract** with required fields (event_time, source, raw payload) and optional fields (event_id, tenant/namespace, partition key)
- **Confidence scoring formula** defined in Phase 0 and applied everywhere
- **Evidence schema** for every hypothesis (heuristics, stats, sample events)
- **Evidence compaction strategy**: retain samples for discovery, compact after approval
- **Discovery evaluation harness** with labeled test data in Phase 1
- **Minimum confidence threshold for UI**: default 0.65 (configurable 0.60-0.70)
- **Demo dataset** locked in Phase 0 (e-commerce orders stream) with success criteria
- **Demo data files**: `data/demo-events.json` and `data/expected-discoveries.json`
- **Success criteria** defined in `docs/demo-plan.md`
- **Storage decision**: HashMap for hot state + SQLite for durable snapshots
- **Incrementality is mandatory**: approvals, new events, and rename operations must update the model without full recomputation
- **Latency budgets**:
  - Entity discovery <5s
  - UI updates <1s
  - Relationship inference <10s
- **Model versioning** when discovery rules change
- **Unknown category** for ambiguous fields (kept out of entity model)

---

## 5. High-Level Architecture

### 5.1 Data Plane (Rust)
- Kafka ingestion
- Event normalization + validation
- Statistical sketches
- Timely/Differential Dataflow core (arrangements for incremental updates)
- Ingestion-time ordering (no watermarking)

### 5.2 Discovery Plane
Produces **hypotheses**, not truth.

- Field profiling (cardinality, entropy, recurrence)
- Entity candidate detection
- Relationship inference
- Single signal type (change-point)
- Confidence scoring + evidence payloads
- Discovery quality metrics

### 5.3 Commitment Plane
Turns hypotheses into durable model elements.

- Entity definitions
- Relationship schemas
- Derived signals
- Versioned model state
- Provenance tracking
- Simple approve action

### 5.4 Control Plane
- Human-in-the-loop approvals (single approve)
- Replay / reinterpretation triggers (manual)

### 5.5 Presentation Plane (Demo-first)
- Dashboard (incremental, starting Phase 1)
- Query endpoints
- Change feeds

### 5.6 Model Store + Serving
- Hot state: in-memory HashMap keyed by entity_id/relationship_id
- Durable state: SQLite with WAL enabled
- Tables: entities, relationships, signals, evidence
- Snapshot format: versioned rows with JSON payloads for attributes/metrics
- Snapshot triggers: on approval + periodic interval
- Retention: keep last N versions (configurable)
- Evidence compaction: drop sample events after approval, keep summary evidence
- Restart behavior:
  1. Load latest snapshot into memory
  2. Resume Kafka from last committed offset
  3. Replay events since snapshot
  4. Reconcile discoveries with loaded state
- Historical queries: limited to last N versions
- Scale limits (v1): demo data up to ~100k entities / 200k relationships
- Checkpoint/restore: DD checkpoints written to SQLite snapshots

---

## 6. Discovery Mechanics (Detailed)

### 6.1 Entity Discovery
Heuristics:
- Field names (*_id, uuid, guid, host, email)
- Value signatures (UUIDs, ULIDs, IPs)
- High-cardinality + recurrence

Statistical evidence:
- Functional dependencies
- Co-occurrence frequency
- Stability across time

### 6.2 Relationship Discovery
- Shared presence in events
- Directional dependency (weak, v1)
- No cardinality inference in v1

### 6.3 Signal Discovery (v1)
Signals are *interesting changes*, not predictions.

Initial signal:
- Change-point detection

Signals attach to entities.

### 6.4 Confidence + Evidence (Defined in Phase 0)

**Evidence schema**
```
{
  heuristics: [matched_rules],
  stats: {cardinality, entropy, recurrence},
  sample_events: [max 3]
}
```

**Confidence formula (v1)**
```
confidence = clamp01(
  0.45 * heuristic_score +
  0.35 * stats_score +
  0.20 * recurrence_score
)
```
- `heuristic_score`: fraction of matched entity rules
- `stats_score`: normalized signal from entropy + functional dependency tests
- `recurrence_score`: log-scaled recurrence over the last N events

### 6.5 Unknown + Low-Confidence Handling
- Fields with ambiguous signals are tagged as **unknown**
- UI includes a low-confidence section, excluded from primary entity list

### 6.6 Evidence Compaction Strategy
- Discovery: full evidence (heuristics + stats + up to 3 sample events)
- Approval: compact to summary (heuristics + stats only)
- Retention: keep summary evidence for last N versions

### 6.7 Discovery Evaluation Harness
- Labeled test data for expected entities and relationships (`data/expected-discoveries.json`)
- Metrics: precision@k, recall@k, false positive rate, time-to-discovery
- Gated CI check for discovery regressions (Phase 1)

---

## 7. World Model Representation

### Entities
- ID
- Attributes (latest + history)
- Metrics (rolling, decayed)
- Provenance + evidence (source fields, confidence, compacted samples)

### Relationships
- Directed / undirected
- Temporal validity
- Provenance + evidence (co-occurrence windows, dependency strength)

### Signals
- Type
- Confidence
- Triggering evidence
- Timestamp
- Affected entity/relationship references

All maintained incrementally.

---

## 8. Human + Agent-in-the-Loop

Humans do **review**, not configuration.

Capabilities (v1):
- Approve entity or relationship proposals

---

## 9. Demo Dashboard (Critical Path)

### Phase 1 Panels
1. Stream Overview
2. Discovered Entities list (includes low-confidence section)

### Phase 2 Panels
3. Entity Inspector (evidence view)
4. Relationship Graph
5. Signals Feed

---

## 10. The 2-Minute Demo (North-Star Experience)

**Goal:** Show that Worldcraft can walk up to an unknown stream and rapidly build a meaningful world model.

1. **Connect a Kafka topic**
   - No schema supplied
   - Events immediately visible in Stream Overview

2. **Watch discovery happen**
   - Entity candidates appear (e.g., `customer_id`, `order_id`)
   - Relationships auto-inferred (Order -> Customer)
   - Confidence scores update in real time

3. **Human-in-the-loop moment**
   - Approve one entity ("customer_id -> Customer")
   - Confidence scores update for related entities incrementally
   - Relationships to Customer strengthen without recomputation

4. **World model updates live**
   - Entity Inspector shows current state + evidence
   - Relationship graph materializes

5. **Incremental update moment**
   - New event arrives with a new `customer_id`
   - Customer entity stats update incrementally (no full recompute)
   - Relationship graph updates live

6. **Rename cascade**
   - Rename entity ("order_id -> Order")
   - Downstream references update automatically

7. **Signal fires**
   - Change-point detected (e.g., status change burst)
   - Signal ties back to specific entities with evidence

**Takeaway:** Worldcraft updates the world model incrementally - approvals, new events, and signals update in real time without recomputation.

---

## 11. APIs (v1)

```
GET /entities
GET /entity/{id}
GET /entity/{id}/neighbors
SUBSCRIBE /signals
SUBSCRIBE /model_changes
```

---

## 12. Revised Phase Plan (4 Phases)

### Phase 0: Foundation + De-Risking (Week 1-2)
- Set up project infra (git, GitHub, CI, repo scaffold, dev scripts)
- Create `docs/dev-plan.md`
- Create `AGENTS.md` and `CLAUDE.md` for AI agent guidance
- Define demo dataset schema (e-commerce orders) and success criteria
- Create `data/demo-events.json` and `data/expected-discoveries.json`
- Prototype discovery on sample data before full pipeline
- Define schemas: confidence formula, evidence payload, storage tables (SQLite)
- Write `docs/schemas.md` (SQLite + Rust structs)
- Define evidence compaction + restart behavior
- Create discovery evaluation harness skeleton
- Validate confidence weights on sample data
- Create empty dashboard stub with navigation
- Timely/Differential Dataflow POC with incremental updates (timeboxed)
- Add DD complexity test: relationship inference + approval-triggered recompute
- Implement checkpoint/restore on demo data
- Create `docs/dd-learning-guide.md`
- Document DD decision + fallback in `docs/architecture-decisions.md`
- Deliverable: `docs/demo-plan.md`

### Phase 1: Core Loop (Week 3-5)
- Kafka ingestion (Timely/Differential Dataflow)
- Event normalization with validation
- Field profiling + entity discovery with confidence scoring
- Model storage: HashMap + SQLite snapshots
- Query API: `GET /entities`, `GET /entity/{id}`
- Dashboard: Stream Overview + Discovered Entities list
- Discovery quality metrics wired to harness
- Acceptance:
  - Discovers 5/5 expected entities within 10s
  - Confidence: 4/5 >0.70, 1/5 >0.60
  - False positive rate <20% (max 1 junk entity shown)
  - Relationships: 3/4 expected within 15s
  - UI updates within 1s

### Phase 2: Graph + Signals (Week 6-7)
- Relationship inference (existence only)
- Single signal type (change-point)
- Evidence UI showing why entities/relationships discovered
- Graph visualization
- Acceptance:
  - Signal fires within 30s of relevant events

### Phase 3: Demo-Ready (Week 8-9)
- Approve button (simple)
- Stability + latency tuning
- Full 2-minute demo script + rehearsal

---

## 13. Design Principles

- Hypotheses before truth
- Incremental over batch
- Explainability over cleverness
- State is the product
- Prediction is optional

---

## 14. North Star

> Worldcraft continuously turns raw streams into a living, queryable world model - so humans and agents always know what is true right now.
