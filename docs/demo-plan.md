# Worldcraft Demo Plan

## Demo Dataset

**Dataset:** E-commerce orders stream (Kafka topic or JSONL replay)

**Data files:**
- `data/demo-events.json` (seed events)
- `data/expected-discoveries.json` (labeled entities/relationships)

**Event types (example):**
- order_created
- order_paid
- order_shipped
- order_delivered
- order_refunded

**Core fields (expected to be discovered):**
- order_id
- customer_id
- product_id
- payment_id
- shipment_id
- merchant_id
- status
- amount
- currency
- event_time

**Volume target (demo):**
- 10k events total or 1k events/sec for 60s

**Expected discoveries:**
- Entities: Order, Customer, Product, Merchant, PaymentMethod, Shipment
- Relationships: Order -> Customer, Order -> Product, Order -> Merchant, Payment -> Order
- Signal: change-point on refund/failed status burst

**Why this dataset:**
- Clear entity boundaries with natural relationships
- Strong field-name heuristics (e.g., *_id)
- Easy to simulate change-points (status bursts, refund spikes)

## Success Criteria (Phase 0)

- Discovers 5/5 expected entities within 10s
- Confidence: 4/5 >0.70, 1/5 >0.60
- False positive rate <20% (max 1 junk entity shown)
- Relationship inference: 3/4 expected within 15s
- Signal fires within 30s of relevant events
- UI updates within 1s of model changes
- Approval and rename operations update downstream entities/relationships incrementally (no full recompute)

## Demo Script (2 minutes, Incremental Focus)

1. Connect to Kafka topic and show live Stream Overview.
2. Watch entity discovery populate the Discovered Entities list.
3. Click into an entity to view evidence and sample events.
4. Approve one entity (Customer) and show confidence updates for related entities.
5. Show relationship graph for Order -> Customer and Order -> Product updating live.
6. Send a new event with a new customer_id and show incremental stats update.
7. Rename entity (order_id -> Order) and show downstream references update automatically.
8. Trigger a change-point (e.g., refund spike) and show the signal with evidence.

## Incrementality Takeaway

Worldcraft updates the world model incrementally: approvals, new events, and renames update downstream state without recomputation.

## Discovery Quality Evaluation

- Labeled dataset with expected entities and relationships
- Metrics: precision@k, recall@k, false positive rate, time-to-discovery
- Regression check in CI for discovery metrics

## Plan B (If Discovery Underperforms)

- Fallback to curated entity mapping for demo dataset
- Reduce scope to fewer entities and stronger heuristics
- Increase confidence threshold and simplify UI to reduce false positives

## Open Questions (This Week)

1. Final source for demo data (Kafka topic vs. static replay)?
2. What is the "wow moment" to emphasize in the demo?
3. Confidence threshold for UI (default 0.65 vs 0.70)?
4. How to measure discovery quality in CI (target metrics)?
5. Is a single change-point signal sufficient for the demo story?
