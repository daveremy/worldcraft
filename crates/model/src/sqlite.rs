use std::path::Path;

use rusqlite::{params, Connection, Result};

use crate::{Evidence, EvidenceStats, Entity, Relationship, Signal};

pub struct SnapshotStore {
    conn: Connection,
}

impl SnapshotStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS entities (
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

            CREATE TABLE IF NOT EXISTS relationships (
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

            CREATE TABLE IF NOT EXISTS signals (
              id TEXT PRIMARY KEY,
              type TEXT NOT NULL,
              target_ref TEXT NOT NULL,
              confidence REAL NOT NULL,
              evidence_id TEXT NOT NULL,
              model_version INTEGER NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS evidence (
              id TEXT PRIMARY KEY,
              heuristics_json TEXT NOT NULL,
              stats_json TEXT NOT NULL,
              sample_events_json TEXT NOT NULL,
              compacted INTEGER NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS entities_by_name ON entities(name);
            CREATE INDEX IF NOT EXISTS relationships_by_from ON relationships(from_entity_id);
            CREATE INDEX IF NOT EXISTS signals_by_type ON signals(type);
            ",
        )
    }

    pub fn write_snapshot(
        &mut self,
        entities: &[Entity],
        relationships: &[Relationship],
        signals: &[Signal],
        evidence: &[Evidence],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        for entity in entities {
            tx.execute(
                "
                INSERT OR REPLACE INTO entities (
                  id, name, field_name, confidence, status, attributes_json,
                  metrics_json, evidence_id, rule_version, model_version,
                  created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ",
                params![
                    entity.id,
                    entity.name,
                    entity.field_name,
                    entity.confidence,
                    entity.status,
                    serde_json::to_string(&entity.attributes).unwrap_or_else(|_| "{}".to_string()),
                    serde_json::to_string(&entity.metrics).unwrap_or_else(|_| "{}".to_string()),
                    entity.evidence_id,
                    entity.rule_version,
                    entity.model_version,
                    entity.created_at,
                    entity.updated_at,
                ],
            )?;
        }

        for relationship in relationships {
            tx.execute(
                "
                INSERT OR REPLACE INTO relationships (
                  id, from_entity_id, to_entity_id, confidence, status,
                  evidence_id, rule_version, model_version, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ",
                params![
                    relationship.id,
                    relationship.from_entity_id,
                    relationship.to_entity_id,
                    relationship.confidence,
                    relationship.status,
                    relationship.evidence_id,
                    relationship.rule_version,
                    relationship.model_version,
                    relationship.created_at,
                    relationship.updated_at,
                ],
            )?;
        }

        for signal in signals {
            tx.execute(
                "
                INSERT OR REPLACE INTO signals (
                  id, type, target_ref, confidence, evidence_id,
                  model_version, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                params![
                    signal.id,
                    signal.signal_type,
                    signal.target_ref,
                    signal.confidence,
                    signal.evidence_id,
                    signal.model_version,
                    signal.created_at,
                ],
            )?;
        }

        for ev in evidence {
            tx.execute(
                "
                INSERT OR REPLACE INTO evidence (
                  id, heuristics_json, stats_json, sample_events_json,
                  compacted, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ",
                params![
                    ev.id,
                    serde_json::to_string(&ev.heuristics).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&ev.stats).unwrap_or_else(|_| "{}".to_string()),
                    serde_json::to_string(&ev.sample_events).unwrap_or_else(|_| "[]".to_string()),
                    if ev.compacted { 1 } else { 0 },
                    ev.created_at,
                ],
            )?;
        }

        tx.commit()
    }

    pub fn load_entities(&self) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, name, field_name, confidence, status, attributes_json, metrics_json,
                   evidence_id, rule_version, model_version, created_at, updated_at
            FROM entities
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            let attributes_json: String = row.get(5)?;
            let metrics_json: String = row.get(6)?;
            Ok(Entity {
                id: row.get(0)?,
                name: row.get(1)?,
                field_name: row.get(2)?,
                confidence: row.get(3)?,
                status: row.get(4)?,
                attributes: serde_json::from_str(&attributes_json).unwrap_or(serde_json::json!({})),
                metrics: serde_json::from_str(&metrics_json).unwrap_or(serde_json::json!({})),
                evidence_id: row.get(7)?,
                rule_version: row.get(8)?,
                model_version: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
        }
        Ok(entities)
    }

    pub fn load_evidence(&self) -> Result<Vec<Evidence>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, heuristics_json, stats_json, sample_events_json, compacted, created_at
            FROM evidence
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            let heuristics_json: String = row.get(1)?;
            let stats_json: String = row.get(2)?;
            let samples_json: String = row.get(3)?;
            let compacted: i64 = row.get(4)?;
            Ok(Evidence {
                id: row.get(0)?,
                heuristics: serde_json::from_str(&heuristics_json).unwrap_or_default(),
                stats: serde_json::from_str(&stats_json).unwrap_or(EvidenceStats {
                    cardinality: 0,
                    entropy: 0.0,
                    recurrence: 0.0,
                }),
                sample_events: serde_json::from_str(&samples_json)
                    .unwrap_or_else(|_| Vec::new()),
                compacted: compacted != 0,
                created_at: row.get(5)?,
            })
        })?;

        let mut evidence = Vec::new();
        for row in rows {
            evidence.push(row?);
        }
        Ok(evidence)
    }

    pub fn load_relationships(&self) -> Result<Vec<Relationship>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, from_entity_id, to_entity_id, confidence, status,
                   evidence_id, rule_version, model_version, created_at, updated_at
            FROM relationships
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Relationship {
                id: row.get(0)?,
                from_entity_id: row.get(1)?,
                to_entity_id: row.get(2)?,
                confidence: row.get(3)?,
                status: row.get(4)?,
                evidence_id: row.get(5)?,
                rule_version: row.get(6)?,
                model_version: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        let mut relationships = Vec::new();
        for row in rows {
            relationships.push(row?);
        }
        Ok(relationships)
    }

    pub fn load_signals(&self) -> Result<Vec<Signal>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, type, target_ref, confidence, evidence_id, model_version, created_at
            FROM signals
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Signal {
                id: row.get(0)?,
                signal_type: row.get(1)?,
                target_ref: row.get(2)?,
                confidence: row.get(3)?,
                evidence_id: row.get(4)?,
                model_version: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;

        let mut signals = Vec::new();
        for row in rows {
            signals.push(row?);
        }
        Ok(signals)
    }
}
