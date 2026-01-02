# Worldcraft: The Living World Model

![Worldcraft Comprehensive Flow](../nanobanana-output/a_comprehensive_wideformat_isome.png)

## The Problem: Data Amnesia

In the modern data stack, we have perfect memory of history but no understanding of the present. We answer "What happened at 10:02 AM?" by querying petabytes of logs. But when an AI agent or a human operator asks, **"What is true right now?"**, our systems fall silent. 

We rely on fragile, manually maintained ETL pipelines to stitch together a fragmented view of reality. When the data changes, the pipelines break. When the schema evolves, the model lags behind.

## The Worldcraft Vision

**Worldcraft** is the platform that solves this by building a **World Model**—a continuously evolving, graph-based representation of your business's reality. 

It doesn't just store events; it *learns* from them. It walks up to a raw, chaotic Kafka stream and autonomously constructs a stateful understanding of entities, their relationships, and their behaviors.

### How It Works: The Identity Stack in Action

The Worldcraft platform is composed of four integrated layers that turn raw streams into agent-ready intelligence.

#### 1. It Starts with Discovery
Traditionally, you have to tell a computer what a "Customer" or an "Order" looks like. **Worldcraft Discovery** flips this. It acts as an always-on intelligence layer that profiles your data in real-time. It uses statistical signatures and heuristics to propose hypotheses: *"I see a high-entropy field `user_id` that appears in 90% of events. This is likely an Entity."* or *"I see `order_id` always appearing with `user_id`. There is likely a Relationship here."*

#### 2. Building the World Model
These hypotheses harden into the **World Model**. This is not a static database table; it is a living graph. As new events flow in, the model updates incrementally. If a human approves a discovery, the model solidifies. If the data drifts, the model adapts. This model is the single source of truth for the state of the world.

#### 3. detecting the Pulse
A static model is useful, but a dynamic one is game-changing. **Worldcraft Signals** monitors the World Model for meaningful shifts. It doesn't just alert on raw metric spikes; it detects semantic changes—like a "Customer" entity suddenly changing its behavioral pattern or a "Server" entity breaking a long-standing relationship with a "Cluster". These signals are the heartbeat of the system.

#### 4. Delivering Context
Finally, intelligence is useless if it's trapped. **Worldcraft Context** is the interface for humans and AI agents. When an agent needs to make a decision, it doesn't scrape logs. It queries the Context layer to get a high-fidelity, instantaneous snapshot of an entity: its history, its neighbors, its current state, and the evidence backing it up.

---

## Why This Is Game-Changing

### Beyond "What Happened"
Traditional streaming systems (Flink, Spark Streaming) are verbs—they *process* data. Worldcraft is a noun—it *maintains* state. It shifts the paradigm from **Event-Centric** (processing logs) to **Entity-Centric** (understanding things).

### Zero-Config Intelligence
The biggest bottleneck in data engineering is mapping schemas. Worldcraft's autonomous discovery removes this barrier. You connect a stream, and minutes later, you have a working graph of your data.

### Incremental Everything
Powered by **Rust and Differential Dataflow**, Worldcraft creates a system where "recomputing the world" is obsolete. Every new event, every human approval, and every schema change ripples through the graph incrementally, updating only what needs to change.

---

## Use Cases

### 1. The AI Agent's Memory
Autonomous agents act on the present. Worldcraft provides the ground truth they need. Instead of an agent trying to parse raw SQL logs to understand a user's status, it queries Worldcraft Context: *"Who is User 123?"* and receives a structured, evidence-backed profile instantly.

### 2. Adaptive Security & Fraud
Attackers change tactics faster than rules can be written. Worldcraft Discovery identifies new, anomalous relationships (e.g., a "Device" connecting to a "Server" it never has before) and emits Signals before a static rule would ever catch it.

### 3. Dynamic Operations
In complex microservices or logistics networks, the topology is always changing. Worldcraft automatically maps the dependency graph of services or supply chains simply by observing the traffic between them, giving operators a map that never goes out of date.

---

## Architecture Overview

To achieve this, Worldcraft employs a unique plane-based architecture:

![Worldcraft Architecture](../nanobanana-output/a_technical_architecture_diagram.png)

*   **Data Plane:** Ingests and normalizes streams (Rust/Kafka).
*   **Discovery Plane:** Generates hypotheses using Differential Dataflow.
*   **Commitment Plane:** Manages the versioned state of the World Model.
*   **Presentation Plane:** Serves the Dashboard and Context API.

> **Worldcraft turns raw streams into a living, queryable reality.**
