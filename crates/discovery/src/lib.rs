use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStats {
    pub cardinality: u64,
    pub entropy: f64,
    pub recurrence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub heuristics: Vec<String>,
    pub stats: EvidenceStats,
    pub sample_events: Vec<serde_json::Value>,
}

pub fn score_confidence(heuristic_score: f64, stats_score: f64, recurrence_score: f64) -> f64 {
    let raw = 0.45 * heuristic_score + 0.35 * stats_score + 0.20 * recurrence_score;
    raw.max(0.0).min(1.0)
}
