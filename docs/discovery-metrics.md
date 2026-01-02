# Discovery Metrics

This file captures the latest prototype runs against `data/demo-events.json`.

## Rust Prototype (non-DD)

Run:
- `make prototype`
- `make eval`

Results (current):
- precision: 1.0
- recall: 1.0
- false_positive_rate: 0.0
- tp_avg_confidence: 0.8692
- tp_min_confidence: 0.827

Latency (ms):
- load_ms: 0
- compute_ms: 1
- write_ms: 0
- total_ms: 2

## Timely/DD POC

Run:
- `make dd_poc`
- `make dd_eval`

Results (current):
- precision: 1.0
- recall: 1.0
- false_positive_rate: 0.0
- tp_avg_confidence: 0.8975
- tp_min_confidence: 0.5385

Latency (ms):
- load_ms: 0
- dataflow_ms: 26
- write_ms: 0
- total_ms: 29

## Notes

- Confidence values differ because the DD POC uses count/total-events as a proxy and includes an incremental event.
- These metrics are for the small demo dataset and should be updated after each heuristic change.
