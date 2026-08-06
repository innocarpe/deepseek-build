//! Light non-blocking plan checklist (spec 110).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanError {
    #[error("invalid action: {0}")]
    InvalidAction(String),
    #[error("invalid index: {0}")]
    InvalidIndex(String),
    #[error("invalid arguments: {0}")]
    Args(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanItem {
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanStore {
    pub items: Vec<PlanItem>,
}

impl PlanStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, args: &Value) -> Result<Value, PlanError> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PlanError::Args("missing action".into()))?;
        match action {
            "get" => Ok(json!({ "items": self.items })),
            "clear" => {
                self.items.clear();
                Ok(json!({ "ok": true, "items": [] }))
            }
            "set" => {
                let items = parse_items(args)?;
                self.items = items
                    .into_iter()
                    .map(|text| PlanItem { text, done: false })
                    .collect();
                Ok(json!({ "ok": true, "items": self.items }))
            }
            "add" => {
                let items = parse_items(args)?;
                for text in items {
                    self.items.push(PlanItem { text, done: false });
                }
                Ok(json!({ "ok": true, "items": self.items }))
            }
            "complete" => {
                let idx = args
                    .get("index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| PlanError::Args("complete requires index".into()))?
                    as usize;
                let item = self
                    .items
                    .get_mut(idx)
                    .ok_or_else(|| PlanError::InvalidIndex(format!("{idx}")))?;
                item.done = true;
                Ok(json!({ "ok": true, "items": self.items }))
            }
            other => Err(PlanError::InvalidAction(other.into())),
        }
    }
}

fn parse_items(args: &Value) -> Result<Vec<String>, PlanError> {
    let arr = args
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| PlanError::Args("items array required".into()))?;
    let mut out = Vec::new();
    for v in arr {
        let s = v
            .as_str()
            .ok_or_else(|| PlanError::Args("items must be strings".into()))?;
        if !s.trim().is_empty() {
            out.push(s.to_string());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_complete_clear() {
        let mut p = PlanStore::new();
        p.apply(&json!({"action":"set","items":["a","b"]})).unwrap();
        let g = p.apply(&json!({"action":"get"})).unwrap();
        assert_eq!(g["items"].as_array().unwrap().len(), 2);
        p.apply(&json!({"action":"complete","index":0})).unwrap();
        assert!(p.items[0].done);
        assert!(!p.items[1].done);
        p.apply(&json!({"action":"clear"})).unwrap();
        assert!(p.items.is_empty());
    }

    #[test]
    fn invalid_index_errors() {
        let mut p = PlanStore::new();
        p.apply(&json!({"action":"set","items":["a"]})).unwrap();
        let err = p.apply(&json!({"action":"complete","index":9})).unwrap_err();
        assert!(matches!(err, PlanError::InvalidIndex(_)));
    }
}
