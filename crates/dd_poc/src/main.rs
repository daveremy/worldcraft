use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use differential_dataflow::input::InputSession;
use differential_dataflow::operators::{Count, Join, Reduce, Threshold};
use serde::Serialize;
use timely::Config;

const IGNORE_FIELDS: [&str; 4] = ["event_id", "event_type", "event_time", "source"];

#[derive(Debug, Clone)]
struct PreparedEvent {
    event_id: String,
    field_values: Vec<(String, String)>,
}

#[derive(Debug, Serialize, Clone)]
struct Update {
    stream: String,
    time: usize,
    diff: isize,
    data: serde_json::Value,
}

fn arg_value(args: &[String], flag: &str, default: &str) -> String {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .map(String::from)
        .unwrap_or_else(|| default.to_string())
}

fn load_events(path: &Path) -> Vec<serde_json::Value> {
    let data = std::fs::read_to_string(path).expect("failed to read demo events");
    serde_json::from_str(&data).expect("failed to parse demo events")
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(inner) => inner.clone(),
        _ => value.to_string(),
    }
}

fn prepare_events(events: &[serde_json::Value]) -> Vec<PreparedEvent> {
    let mut prepared = Vec::new();

    for (index, event) in events.iter().enumerate() {
        let mut field_values = Vec::new();
        let mut event_id = format!("evt-{:04}", index + 1);

        if let Some(map) = event.as_object() {
            if let Some(raw_id) = map.get("event_id").and_then(|value| value.as_str()) {
                event_id = raw_id.to_string();
            }
            for (field, value) in map {
                if IGNORE_FIELDS.iter().any(|ignored| ignored == field) {
                    continue;
                }
                let value_str = value_to_string(value);
                field_values.push((field.clone(), value_str));
            }
        }

        prepared.push(PreparedEvent {
            event_id,
            field_values,
        });
    }

    prepared
}

fn relationship_candidate(left: &str, right: &str) -> bool {
    if left == "payment_id" {
        return right == "order_id";
    }
    if left == "order_id" {
        return matches!(right, "customer_id" | "product_id" | "merchant_id");
    }
    false
}

fn orient_relationship(left: String, right: String) -> (String, String) {
    if left == "payment_id" {
        return (left, right);
    }
    if right == "payment_id" {
        return (right, left);
    }
    if left == "order_id" {
        return (left, right);
    }
    if right == "order_id" {
        return (right, left);
    }
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn materialize_stream(updates: &[Update], stream: &str) -> Vec<serde_json::Value> {
    let mut counts: HashMap<String, isize> = HashMap::new();
    for update in updates.iter().filter(|u| u.stream == stream) {
        let key = serde_json::to_string(&update.data).unwrap_or_default();
        *counts.entry(key).or_insert(0) += update.diff;
    }

    counts
        .into_iter()
        .filter(|(_, diff)| *diff > 0)
        .filter_map(|(key, _)| serde_json::from_str(&key).ok())
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_path = arg_value(&args, "--input", "data/demo-events.json");
    let output_path = arg_value(&args, "--output", "out/dd_discovery.json");
    let updates_path = arg_value(&args, "--updates-out", "out/dd_updates.json");
    let metrics_path = arg_value(&args, "--metrics-out", "");

    let total_start = Instant::now();
    let load_start = Instant::now();
    let events = load_events(Path::new(&input_path));
    let load_ms = load_start.elapsed().as_millis();

    let mut events = prepare_events(&events);
    if let Some(mut template) = events.first().cloned() {
        for (field, value) in template.field_values.iter_mut() {
            if field == "customer_id" {
                *value = "cus-999".to_string();
            }
            if field == "status" {
                *value = "failed".to_string();
            }
        }
        template.event_id = format!("evt-inc-{}", events.len() + 1);
        events.push(template);
    }

    let split_index = events.len().saturating_sub(1);
    let (initial_events, incremental_events) = {
        let (initial, incremental) = events.split_at(split_index);
        (initial.to_vec(), incremental.to_vec())
    };
    let total_events = events.len().max(1) as f64;

    let updates: Arc<Mutex<Vec<Update>>> = Arc::new(Mutex::new(Vec::new()));
    let updates_for_dataflow = Arc::clone(&updates);

    let dataflow_start = Instant::now();
    timely::execute(Config::thread(), move |worker| {
        let mut field_input: InputSession<usize, (String, String, String), isize> =
            InputSession::new();
        let mut approval_input: InputSession<usize, String, isize> = InputSession::new();
        let mut rename_input: InputSession<usize, (String, String), isize> = InputSession::new();

        let is_leader = worker.index() == 0;
        let updates = Arc::clone(&updates_for_dataflow);
        let mut probe = timely::dataflow::operators::probe::Handle::new();

        worker.dataflow(|scope| {
            let field_values = field_input.to_collection(scope);
            let approvals = approval_input
                .to_collection(scope)
                .map(|field| (field, ()));
            let renames = rename_input
                .to_collection(scope)
                .map(|(field, name)| (field, (1i8, name)));

            let field_presence = field_values
                .map(|(_event_id, field, _)| field)
                .count();
            let field_cardinality = field_values
                .map(|(_event_id, field, value)| (field, value))
                .distinct()
                .map(|(field, _)| field)
                .count();

            let field_stats = field_presence
                .join_map(&field_cardinality, |field, count, card| {
                    (field.clone(), (*count, *card))
                });

            let identity_names = field_stats
                .map(|(field, _stats)| (field.clone(), (0i8, field.clone())))
                .distinct();

            let names = identity_names
                .concat(&renames)
                .reduce(|_field, vals, output| {
                    if let Some((val, _diff)) = vals.last() {
                        output.push(((*val).clone(), 1));
                    }
                })
                .map(|(field, (_priority, name))| (field, name));

            let approved_entities = field_stats
                .map(|(field, (count, card))| (field.clone(), (count, card)))
                .join_map(&approvals, |field, (count, card), _| {
                    (field.clone(), *count, *card)
                });

            let approved_named = approved_entities
                .map(|(field, count, card)| (field.clone(), (count, card)))
                .join_map(&names, |field, (count, card), name: &String| {
                    (field.clone(), name.clone(), *count, *card)
                });

            let event_fields = field_values
                .map(|(event_id, field, _)| (event_id, field))
                .filter(|(_event_id, field)| field.ends_with("_id"))
                .distinct();

            let rel_pairs = event_fields
                .join_map(&event_fields, |_event_id, left, right| (left.clone(), right.clone()))
                .filter(|(left, right)| left < right)
                .map(|(left, right)| orient_relationship(left, right))
                .filter(|(left, right)| relationship_candidate(left, right));

            let rel_counts = rel_pairs.count();

            let rel_approval_base = rel_counts.map(|(pair, _count)| (pair, 0i8));
            let rel_left_approval = rel_counts
                .map(|((left, right), _count)| (left.clone(), right.clone()))
                .join_map(&approvals, |left, right, _| ((left.clone(), right.clone()), 1i8));
            let rel_right_approval = rel_counts
                .map(|((left, right), _count)| (right.clone(), left.clone()))
                .join_map(&approvals, |right, left, _| ((left.clone(), right.clone()), 1i8));

            let rel_approval_score = rel_approval_base
                .concat(&rel_left_approval)
                .concat(&rel_right_approval)
                .reduce(|_pair, vals, output| {
                    let mut total = 0i8;
                    for (score, diff) in vals {
                        total += *score * (*diff as i8);
                    }
                    output.push((total, 1));
                });

            let rel_with_approval = rel_counts.join_map(&rel_approval_score, |pair, count, score| {
                (pair.clone(), (*count, *score))
            });

            let rel_named_left = rel_with_approval
                .map(|((left, right), (count, score))| (left.clone(), (right.clone(), count, score)))
                .join_map(&names, |_left, (right, count, score), name: &String| {
                    (right.clone(), (name.clone(), *count, *score))
                });

            let rel_named = rel_named_left.join_map(
                &names,
                |_right, (left_name, count, score), right_name: &String| {
                    ((left_name.clone(), right_name.clone()), (*count, *score))
                },
            );

            let bad_status = field_values
                .filter(|(_event_id, field, value)| {
                    field == "status" && (value == "refunded" || value == "failed")
                })
                .map(|(_event_id, _field, _value)| "refund_or_failed".to_string());

            let bad_status_counts = bad_status.count();

            let bad_status_signal = bad_status
                .threshold(|_key, count| if *count >= 4 { 1 } else { 0 })
                .map(|key| (key, ()));

            let bad_status_with_count = bad_status_signal.join_map(
                &bad_status_counts,
                |key, _unit, count| (key.clone(), *count),
            );

            if is_leader {
                let updates = Arc::clone(&updates);
                field_stats.inspect(move |(data, time, diff)| {
                    if let Ok(mut guard) = updates.lock() {
                        let (field, (count, card)) = data;
                        guard.push(Update {
                            stream: "field_stats".to_string(),
                            time: *time,
                            diff: *diff,
                            data: serde_json::json!({
                                "field": field,
                                "count": count,
                                "cardinality": card
                            }),
                        });
                    }
                });
            }

            if is_leader {
                let updates = Arc::clone(&updates);
                approved_named.inspect(move |(data, time, diff)| {
                    if let Ok(mut guard) = updates.lock() {
                        let (field, name, count, card) = data;
                        guard.push(Update {
                            stream: "approved_entities".to_string(),
                            time: *time,
                            diff: *diff,
                            data: serde_json::json!({
                                "field": field,
                                "name": name,
                                "count": count,
                                "cardinality": card
                            }),
                        });
                    }
                });
            }

            if is_leader {
                let updates = Arc::clone(&updates);
                rel_named.inspect(move |(data, time, diff)| {
                    if let Ok(mut guard) = updates.lock() {
                        let ((left, right), (count, score)) = data;
                        let base = (*count as f64 / (total_events * 1.5)).min(1.0);
                        let bonus = (*score as f64) * 0.1;
                        let confidence = (base + bonus).min(1.0);
                        guard.push(Update {
                            stream: "relationships".to_string(),
                            time: *time,
                            diff: *diff,
                            data: serde_json::json!({
                                "from": left,
                                "to": right,
                                "count": count,
                                "approval_score": score,
                                "confidence": (confidence * 10_000.0).round() / 10_000.0
                            }),
                        });
                    }
                });
            }

            if is_leader {
                let updates = Arc::clone(&updates);
                bad_status_with_count.inspect(move |(data, time, diff)| {
                    if let Ok(mut guard) = updates.lock() {
                        let (key, count) = data;
                        guard.push(Update {
                            stream: "signals".to_string(),
                            time: *time,
                            diff: *diff,
                            data: serde_json::json!({
                                "type": "change_point",
                                "target": "Order.status",
                                "key": key,
                                "count": count,
                                "threshold": 4
                            }),
                        });
                    }
                });
            }

            field_stats.probe_with(&mut probe);
            approved_named.probe_with(&mut probe);
            rel_named.probe_with(&mut probe);
            bad_status_with_count.probe_with(&mut probe);
        });

        if is_leader {
            let mut current_time = 0usize;

            for event in &initial_events {
                for (field, value) in &event.field_values {
                    field_input.insert((event.event_id.clone(), field.clone(), value.clone()));
                }
            }

            current_time += 1;
            field_input.advance_to(current_time);
            approval_input.advance_to(current_time);
            rename_input.advance_to(current_time);
            field_input.flush();
            approval_input.flush();
            rename_input.flush();

            while probe.less_than(&current_time) {
                worker.step();
            }

            approval_input.insert("customer_id".to_string());
            rename_input.insert(("customer_id".to_string(), "Customer".to_string()));
            current_time += 1;
            field_input.advance_to(current_time);
            approval_input.advance_to(current_time);
            rename_input.advance_to(current_time);
            field_input.flush();
            approval_input.flush();
            rename_input.flush();

            while probe.less_than(&current_time) {
                worker.step();
            }

            for event in &incremental_events {
                for (field, value) in &event.field_values {
                    field_input.insert((event.event_id.clone(), field.clone(), value.clone()));
                }
            }

            current_time += 1;
            field_input.advance_to(current_time);
            approval_input.advance_to(current_time);
            rename_input.advance_to(current_time);
            field_input.flush();
            approval_input.flush();
            rename_input.flush();

            while probe.less_than(&current_time) {
                worker.step();
            }

            rename_input.insert(("order_id".to_string(), "Order".to_string()));
            current_time += 1;
            field_input.advance_to(current_time);
            approval_input.advance_to(current_time);
            rename_input.advance_to(current_time);
            field_input.flush();
            approval_input.flush();
            rename_input.flush();

            while probe.less_than(&current_time) {
                worker.step();
            }

            field_input.close();
            approval_input.close();
            rename_input.close();
        }

        while worker.step() {}
    })
    .expect("timely execution failed");
    let dataflow_ms = dataflow_start.elapsed().as_millis();

    let updates = match updates.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };

    let mut entities = materialize_stream(&updates, "field_stats");
    entities.retain(|entity| {
        entity
            .get("field")
            .and_then(|v| v.as_str())
            .map(|field| field.ends_with("_id") && field != "event_id")
            .unwrap_or(false)
    });
    for entity in &mut entities {
        if let Some(obj) = entity.as_object_mut() {
            if let Some(count) = obj.get("count").and_then(|v| v.as_i64()) {
                let confidence = (count as f64 / total_events).min(1.0);
                obj.insert(
                    "confidence".to_string(),
                    serde_json::json!((confidence * 10_000.0).round() / 10_000.0),
                );
            }
        }
    }

    let approved_entities = materialize_stream(&updates, "approved_entities");
    let relationships = materialize_stream(&updates, "relationships");
    let signals = materialize_stream(&updates, "signals");

    let output = serde_json::json!({
        "entities": entities,
        "approved_entities": approved_entities,
        "relationships": relationships,
        "signals": signals
    });

    let output_path = Path::new(&output_path);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create output directory");
    }

    let write_start = Instant::now();
    std::fs::write(output_path, serde_json::to_string_pretty(&output).unwrap())
        .expect("failed to write output");
    let write_ms = write_start.elapsed().as_millis();

    if !updates_path.is_empty() {
        let updates_path = Path::new(&updates_path);
        if let Some(parent) = updates_path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create updates directory");
        }
        std::fs::write(updates_path, serde_json::to_string_pretty(&updates).unwrap())
            .expect("failed to write updates");
    }

    if !metrics_path.is_empty() {
        let metrics = serde_json::json!({
            "load_ms": load_ms,
            "dataflow_ms": dataflow_ms,
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
}
