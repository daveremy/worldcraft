use serde::{Deserialize, Serialize};

pub mod sqlite;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStats {
    pub cardinality: u64,
    pub entropy: f64,
    pub recurrence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub heuristics: Vec<String>,
    pub stats: EvidenceStats,
    pub sample_events: Vec<serde_json::Value>,
    pub compacted: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub field_name: String,
    pub confidence: f64,
    pub status: String,
    pub attributes: serde_json::Value,
    pub metrics: serde_json::Value,
    pub evidence_id: String,
    pub rule_version: String,
    pub model_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub from_entity_id: String,
    pub to_entity_id: String,
    pub confidence: f64,
    pub status: String,
    pub evidence_id: String,
    pub rule_version: String,
    pub model_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub signal_type: String,
    pub target_ref: String,
    pub confidence: f64,
    pub evidence_id: String,
    pub model_version: i64,
    pub created_at: String,
}
