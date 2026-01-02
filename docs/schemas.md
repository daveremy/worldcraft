# Worldcraft Schemas

## Evidence Schema

```json
{
  "heuristics": ["matched_rule"],
  "stats": {"cardinality": 0, "entropy": 0.0, "recurrence": 0.0},
  "sample_events": [{"event_id": "evt-0001"}]
}
```

## Confidence Formula (v1)

```
confidence = clamp01(
  0.45 * heuristic_score +
  0.35 * stats_score +
  0.20 * recurrence_score
)
```

## SQLite Tables (v1)

```sql
CREATE TABLE entities (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  field_name TEXT NOT NULL,
  confidence REAL NOT NULL,
  status TEXT NOT NULL,
  attributes_json TEXT NOT NULL,
  metrics_json TEXT NOT NULL,
  evidence_id TEXT NOT NULL,
  rule_version TEXT NOT NULL,
  model_version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE relationships (
  id TEXT PRIMARY KEY,
  from_entity_id TEXT NOT NULL,
  to_entity_id TEXT NOT NULL,
  confidence REAL NOT NULL,
  status TEXT NOT NULL,
  evidence_id TEXT NOT NULL,
  rule_version TEXT NOT NULL,
  model_version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE signals (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  target_ref TEXT NOT NULL,
  confidence REAL NOT NULL,
  evidence_id TEXT NOT NULL,
  model_version INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE evidence (
  id TEXT PRIMARY KEY,
  heuristics_json TEXT NOT NULL,
  stats_json TEXT NOT NULL,
  sample_events_json TEXT NOT NULL,
  compacted INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX entities_by_name ON entities(name);
CREATE INDEX relationships_by_from ON relationships(from_entity_id);
CREATE INDEX signals_by_type ON signals(type);
```

## Rust Structs (v1)

```rust
struct Evidence {
    id: String,
    heuristics: Vec<String>,
    stats: EvidenceStats,
    sample_events: Vec<serde_json::Value>,
    compacted: bool,
}

struct EvidenceStats {
    cardinality: u64,
    entropy: f64,
    recurrence: f64,
}

struct Entity {
    id: String,
    name: String,
    field_name: String,
    confidence: f64,
    status: String,
    attributes: serde_json::Value,
    metrics: serde_json::Value,
    evidence_id: String,
    rule_version: String,
    model_version: i64,
}

struct Relationship {
    id: String,
    from_entity_id: String,
    to_entity_id: String,
    confidence: f64,
    status: String,
    evidence_id: String,
    rule_version: String,
    model_version: i64,
}

struct Signal {
    id: String,
    signal_type: String,
    target_ref: String,
    confidence: f64,
    evidence_id: String,
    model_version: i64,
}
```

## Evidence Compaction

- Discovery: store full evidence with up to 3 sample events.
- Approval: compact evidence (drop sample events, keep heuristics + stats).
- Retention: keep compacted evidence for last N model versions.

## Unknown Handling

- Fields with ambiguous signals are tagged as `unknown` and excluded from entity output.
- Low-confidence candidates are shown separately in the UI.
