use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use serde_json::Value;
use worldcraft_discovery::score_confidence;

const IGNORE_FIELDS: [&str; 4] = ["event_id", "event_type", "event_time", "source"];

fn arg_value(args: &[String], flag: &str, default: &str) -> String {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .map(String::from)
        .unwrap_or_else(|| default.to_string())
}

fn arg_f64(args: &[String], flag: &str, default: f64) -> f64 {
    arg_value(args, flag, "")
        .parse::<f64>()
        .unwrap_or(default)
}

fn clamp01(value: f64) -> f64 {
    value.max(0.0).min(1.0)
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(inner) => inner.clone(),
        _ => value.to_string(),
    }
}

fn entropy(values: &[String]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for value in values {
        *counts.entry(value.as_str()).or_insert(0) += 1;
    }
    let total = values.len() as f64;
    counts
        .values()
        .map(|count| {
            let p = *count as f64 / total;
            if p == 0.0 {
                0.0
            } else {
                -p * p.log2()
            }
        })
        .sum()
}

fn heuristic_score(field: &str) -> (f64, Vec<String>) {
    let mut score = 0.0;
    let mut rules = Vec::new();

    if field.ends_with("_id") {
        score += 1.0;
        rules.push("suffix_id".to_string());
    }
    if matches!(field, "uuid" | "guid") {
        score += 0.6;
        rules.push("explicit_uuid_guid".to_string());
    }
    if field.ends_with("_key") {
        score += 0.3;
        rules.push("suffix_key".to_string());
    }
    if field.contains("id") && !field.ends_with("_id") {
        score += 0.2;
        rules.push("contains_id".to_string());
    }

    (clamp01(score), rules)
}

fn stats_score(values: &[String]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let total = values.len() as f64;
    let unique = values.iter().collect::<HashSet<_>>().len() as f64;
    if total == 0.0 {
        return 0.0;
    }
    let cardinality_ratio = unique / total;
    let ent = entropy(values);
    let ent_norm = if unique > 1.0 {
        ent / unique.log2()
    } else {
        0.0
    };

    let score = 0.5 * (cardinality_ratio * 1.2).min(1.0) + 0.5 * ent_norm;
    clamp01(score)
}

fn recurrence_score(count: usize, total_events: usize) -> f64 {
    if total_events == 0 {
        return 0.0;
    }
    let count = count as f64;
    let total = total_events as f64;
    clamp01((count + 1.0).ln() / (total + 1.0).ln())
}

fn load_events(path: &Path) -> Vec<Value> {
    let data = std::fs::read_to_string(path).expect("failed to read demo events");
    serde_json::from_str(&data).expect("failed to parse demo events")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_path = arg_value(&args, "--input", "data/demo-events.json");
    let output_path = arg_value(&args, "--output", "out/discovery.json");
    let metrics_path = arg_value(&args, "--metrics-out", "");
    let threshold = arg_f64(&args, "--threshold", 0.65);

    let total_start = Instant::now();
    let load_start = Instant::now();
    let events = load_events(Path::new(&input_path));
    let load_ms = load_start.elapsed().as_millis();
    let total_events = events.len();
    let mut values_by_field: HashMap<String, Vec<String>> = HashMap::new();
    let mut presence_by_field: HashMap<String, usize> = HashMap::new();

    let compute_start = Instant::now();
    for event in events {
        if let Value::Object(map) = event {
            for (field, value) in map {
                if IGNORE_FIELDS.iter().any(|ignored| ignored == &field) {
                    continue;
                }
                *presence_by_field.entry(field.clone()).or_insert(0) += 1;
                values_by_field
                    .entry(field)
                    .or_default()
                    .push(value_to_string(&value));
            }
        }
    }

    let mut entities = Vec::new();
    let mut unknown = Vec::new();

    for (field, values) in values_by_field {
        let (heuristic_s, rules) = heuristic_score(&field);
        let stats_s = stats_score(&values);
        let rec_s = recurrence_score(
            *presence_by_field.get(&field).unwrap_or(&0),
            total_events,
        );
        let conf = score_confidence(heuristic_s, stats_s, rec_s);

        let entry = serde_json::json!({
            "field": field,
            "confidence": round4(conf),
            "heuristics": rules,
            "stats": {
                "cardinality": values.iter().collect::<HashSet<_>>().len(),
                "entropy": round4(entropy(&values)),
                "recurrence": round4(rec_s),
            }
        });

        if conf >= threshold {
            entities.push(entry);
        } else {
            unknown.push(entry);
        }
    }
    let compute_ms = compute_start.elapsed().as_millis();

    entities.sort_by(|a, b| {
        let b_conf = b.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let a_conf = a.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b_conf
            .partial_cmp(&a_conf)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    unknown.sort_by(|a, b| {
        let b_conf = b.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let a_conf = a.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b_conf
            .partial_cmp(&a_conf)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let output = serde_json::json!({
        "entities": entities,
        "unknown": unknown,
    });

    let output_path = Path::new(&output_path);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create output directory");
    }
    let write_start = Instant::now();
    std::fs::write(output_path, serde_json::to_string_pretty(&output).unwrap())
        .expect("failed to write output");
    let write_ms = write_start.elapsed().as_millis();

    if !metrics_path.is_empty() {
        let metrics = serde_json::json!({
            "load_ms": load_ms,
            "compute_ms": compute_ms,
            "write_ms": write_ms,
            "total_ms": total_start.elapsed().as_millis()
        });
        let metrics_path = Path::new(&metrics_path);
        if let Some(parent) = metrics_path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create metrics directory");
        }
        std::fs::write(metrics_path, serde_json::to_string_pretty(&metrics).unwrap())
            .expect("failed to write metrics");
    }

    println!(
        "events={} entities={} unknown={}",
        total_events,
        output["entities"].as_array().map(|v| v.len()).unwrap_or(0),
        output["unknown"].as_array().map(|v| v.len()).unwrap_or(0)
    );
}
