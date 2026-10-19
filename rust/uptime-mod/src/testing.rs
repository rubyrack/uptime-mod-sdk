//! Host-side helpers for testing a mod without a host: build a snapshot from a
//! partial JSON value, and encode it the way the host does.
//!
//! The schema is the snapshot schema the SDK was generated from, so a test
//! names only the fields it cares about and every required field the schema
//! lists is filled with an empty value.

use serde_json::{json, Map, Value};

const SCHEMA: &str = include_str!("generated/snapshot.schema.json");

fn schema() -> Value {
    serde_json::from_str(SCHEMA).expect("the committed schema is JSON")
}

/// A snapshot value with the envelope filled, `overrides` merged over it
/// (section by section), and every required field beneath `overrides` filled.
pub fn snapshot(overrides: Value) -> Value {
    let schema = schema();
    let root = &schema["components"]["ModSnapshot"];
    let mut value = json!({
        "mod_api": { "generation": crate::snapshot::API_GENERATION, "minor": crate::snapshot::API_MINOR },
        "quantum": 0,
        "sim_time_ns": 0,
    });
    if let (Value::Object(base), Value::Object(extra)) = (&mut value, overrides) {
        base.extend(extra);
    }
    complete(&mut value, root, &schema);
    value
}

/// A snapshot with every section present and empty-valued: decoding it
/// exercises every type reachable from `ModSnapshot`.
pub fn full_snapshot() -> Value {
    let schema = schema();
    let root = &schema["components"]["ModSnapshot"];
    let mut sections = Map::new();
    for (name, prop) in root["properties"].as_object().into_iter().flatten() {
        if !root["required"]
            .as_array()
            .is_some_and(|r| r.iter().any(|n| n == name))
        {
            sections.insert(name.clone(), minimal(strip_null(prop), &schema));
        }
    }
    snapshot(Value::Object(sections))
}

/// Encodes `value` as named msgpack, the host's snapshot encoding.
pub fn encode(value: &Value) -> Vec<u8> {
    rmp_serde::to_vec_named(value).expect("a JSON value encodes")
}

/// The schema of a nullable property with its null alternative removed.
fn strip_null(prop: &Value) -> &Value {
    prop["oneOf"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["type"] != "null"))
        .unwrap_or(prop)
}

fn resolve<'a>(schema: &'a Value, root: &'a Value) -> &'a Value {
    match schema["$ref"].as_str() {
        Some(r) => &root["components"][r.rsplit('/').next().unwrap_or_default()],
        None => schema,
    }
}

fn base_type(schema: &Value) -> Option<&str> {
    match &schema["type"] {
        Value::String(s) => Some(s.as_str()),
        Value::Array(a) => a.iter().filter_map(Value::as_str).find(|t| *t != "null"),
        _ => None,
    }
}

fn minimal(schema: &Value, root: &Value) -> Value {
    let schema = resolve(strip_null(schema), root);
    if let Some(first) = schema["enum"].as_array().and_then(|e| e.first()) {
        return first.clone();
    }
    if let Some(first) = schema["oneOf"].as_array().and_then(|v| v.first()) {
        if let Some(unit) = first["enum"].as_array().and_then(|e| e.first()) {
            return unit.clone();
        }
        let (tag, body) = first["properties"]
            .as_object()
            .and_then(|p| p.iter().next())
            .expect("a tagged variant has one property");
        return json!({ tag: minimal(body, root) });
    }
    match base_type(schema) {
        Some("string") => json!(""),
        Some("integer") => json!(0),
        Some("number") => json!(0.0),
        Some("boolean") => json!(false),
        Some("array") => json!([]),
        Some("object") => {
            let mut v = json!({});
            complete(&mut v, schema, root);
            v
        }
        other => panic!("no minimal value for {other:?}"),
    }
}

/// Adds every missing required field of `schema` to `value`, recursively.
fn complete(value: &mut Value, schema: &Value, root: &Value) {
    let schema = resolve(strip_null(schema), root);
    match value {
        Value::Object(map) => {
            if let Some(props) = schema["properties"].as_object() {
                for name in schema["required"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    if !map.contains_key(name) {
                        map.insert(name.to_owned(), minimal(&props[name], root));
                    }
                }
                for (name, v) in map.iter_mut() {
                    if let Some(prop) = props.get(name) {
                        complete(v, prop, root);
                    }
                }
            } else if let Some(variants) = schema["oneOf"].as_array() {
                for (tag, v) in map.iter_mut() {
                    let body = variants.iter().find_map(|var| var["properties"].get(tag));
                    if let Some(body) = body {
                        complete(v, body, root);
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                complete(item, &schema["items"], root);
            }
        }
        _ => {}
    }
}
