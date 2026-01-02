use std::path::Path;

use worldcraft_model::sqlite::SnapshotStore;
use worldcraft_model::{Entity, Evidence, EvidenceStats, Relationship, Signal};

fn main() {
    let db_path = Path::new("out/worldcraft.db");
    if db_path.exists() {
        std::fs::remove_file(db_path).expect("failed to remove existing db");
    }

    let mut store = SnapshotStore::open(db_path).expect("failed to open snapshot store");

    let evidence = Evidence {
        id: "ev-1".to_string(),
        heuristics: vec!["suffix_id".to_string()],
        stats: EvidenceStats {
            cardinality: 10,
            entropy: 1.0,
            recurrence: 0.5,
        },
        sample_events: Vec::new(),
        compacted: false,
        created_at: "2025-01-01T00:00:00Z".to_string(),
    };

    let entity = Entity {
        id: "ent-1".to_string(),
        name: "Customer".to_string(),
        field_name: "customer_id".to_string(),
        confidence: 0.9,
        status: "candidate".to_string(),
        attributes: serde_json::json!({}),
        metrics: serde_json::json!({}),
        evidence_id: evidence.id.clone(),
        rule_version: "v1".to_string(),
        model_version: 1,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        updated_at: "2025-01-01T00:00:00Z".to_string(),
    };

    let relationship = Relationship {
        id: "rel-1".to_string(),
        from_entity_id: "ent-1".to_string(),
        to_entity_id: "ent-2".to_string(),
        confidence: 0.8,
        status: "candidate".to_string(),
        evidence_id: evidence.id.clone(),
        rule_version: "v1".to_string(),
        model_version: 1,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        updated_at: "2025-01-01T00:00:00Z".to_string(),
    };

    let signal = Signal {
        id: "sig-1".to_string(),
        signal_type: "change_point".to_string(),
        target_ref: "Customer.status".to_string(),
        confidence: 0.7,
        evidence_id: evidence.id.clone(),
        model_version: 1,
        created_at: "2025-01-01T00:00:00Z".to_string(),
    };

    store
        .write_snapshot(&[entity], &[relationship], &[signal], &[evidence])
        .expect("failed to write snapshot");

    let entities = store.load_entities().expect("failed to load entities");
    let relationships = store.load_relationships().expect("failed to load relationships");
    let signals = store.load_signals().expect("failed to load signals");
    let evidence = store.load_evidence().expect("failed to load evidence");

    println!(
        "entities={} relationships={} signals={} evidence={}",
        entities.len(),
        relationships.len(),
        signals.len(),
        evidence.len()
    );
}
