use std::collections::{HashMap, HashSet};
use std::path::Path;

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

fn arg_optional_f64(args: &[String], flag: &str) -> Option<f64> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .and_then(|value| value.parse::<f64>().ok())
}

fn load_json(path: &Path) -> serde_json::Value {
    let data = std::fs::read_to_string(path).expect("failed to read json");
    serde_json::from_str(&data).expect("failed to parse json")
}

fn fields_from_entities(value: &serde_json::Value) -> HashSet<String> {
    value
        .get("entities")
        .and_then(|entities| entities.as_array())
        .map(|entities| {
            entities
                .iter()
                .filter_map(|entity| {
                    entity
                        .get("field")
                        .and_then(|field| field.as_str())
                        .map(|field| field.to_string())
                })
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
}

fn confidence_by_field(value: &serde_json::Value) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    if let Some(entities) = value.get("entities").and_then(|v| v.as_array()) {
        for entity in entities {
            let field = entity
                .get("field")
                .and_then(|v| v.as_str())
                .map(|field| field.to_string());
            let confidence = entity.get("confidence").and_then(|v| v.as_f64());
            if let (Some(field), Some(confidence)) = (field, confidence) {
                map.entry(field)
                    .and_modify(|existing: &mut f64| *existing = (*existing).max(confidence))
                    .or_insert(confidence);
            }
        }
    }
    map
}

fn avg(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let expected_path = arg_value(&args, "--expected", "data/expected-discoveries.json");
    let actual_path = arg_value(&args, "--actual", "out/discovery.json");
    let min_recall = arg_f64(&args, "--min-recall", 0.7);
    let max_fpr = arg_f64(&args, "--max-fpr", 0.2);
    let min_tp_conf = arg_optional_f64(&args, "--min-tp-confidence");
    let max_fp_conf = arg_optional_f64(&args, "--max-fp-confidence");

    let expected = load_json(Path::new(&expected_path));
    let actual = load_json(Path::new(&actual_path));

    let expected_fields = fields_from_entities(&expected);
    let actual_fields = fields_from_entities(&actual);
    let confidence_map = confidence_by_field(&actual);

    let true_pos = expected_fields
        .intersection(&actual_fields)
        .count() as f64;
    let false_pos = actual_fields
        .difference(&expected_fields)
        .count() as f64;
    let false_neg = expected_fields
        .difference(&actual_fields)
        .count() as f64;

    let precision = if actual_fields.is_empty() {
        0.0
    } else {
        true_pos / actual_fields.len() as f64
    };
    let recall = if expected_fields.is_empty() {
        0.0
    } else {
        true_pos / expected_fields.len() as f64
    };
    let fpr = if true_pos + false_pos == 0.0 {
        0.0
    } else {
        false_pos / (true_pos + false_pos)
    };

    let tp_conf: Vec<f64> = expected_fields
        .intersection(&actual_fields)
        .filter_map(|field| confidence_map.get(field).copied())
        .collect();
    let fp_conf: Vec<f64> = actual_fields
        .difference(&expected_fields)
        .filter_map(|field| confidence_map.get(field).copied())
        .collect();

    let result = serde_json::json!({
        "expected": expected_fields.len(),
        "actual": actual_fields.len(),
        "true_pos": true_pos as usize,
        "false_pos": false_pos as usize,
        "false_neg": false_neg as usize,
        "precision": (precision * 10_000.0).round() / 10_000.0,
        "recall": (recall * 10_000.0).round() / 10_000.0,
        "false_positive_rate": (fpr * 10_000.0).round() / 10_000.0,
        "confidence_stats": {
            "tp_avg": avg(&tp_conf).map(|v| (v * 10_000.0).round() / 10_000.0),
            "fp_avg": avg(&fp_conf).map(|v| (v * 10_000.0).round() / 10_000.0),
            "tp_min": tp_conf.iter().copied().reduce(f64::min).map(|v| (v * 10_000.0).round() / 10_000.0),
            "fp_max": fp_conf.iter().copied().reduce(f64::max).map(|v| (v * 10_000.0).round() / 10_000.0)
        }
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());

    if recall < min_recall {
        eprintln!("recall below threshold");
        std::process::exit(1);
    }
    if fpr > max_fpr {
        eprintln!("false positive rate above threshold");
        std::process::exit(1);
    }
    if let Some(min_tp_conf) = min_tp_conf {
        if let Some(tp_avg) = avg(&tp_conf) {
            if tp_avg < min_tp_conf {
                eprintln!("tp confidence below threshold");
                std::process::exit(1);
            }
        }
    }
    if let Some(max_fp_conf) = max_fp_conf {
        if let Some(fp_avg) = avg(&fp_conf) {
            if fp_avg > max_fp_conf {
                eprintln!("fp confidence above threshold");
                std::process::exit(1);
            }
        }
    }
}
