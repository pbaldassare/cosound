//! Minimal JSON Schema validator for operation inputs (subset: type, enum, required, properties,
//! items, minimum/maximum, minItems/maxItems, minLength/maxLength). Unknown keywords are ignored,
//! extra properties are allowed so callers can send newer optional fields.
use serde_json::Value;

fn type_ok(t: &str, v: &Value) -> bool {
    match t {
        "string" => v.is_string(),
        "number" => v.is_number(),
        "integer" => v.as_f64().is_some_and(|x| x.fract() == 0.0),
        "boolean" => v.is_boolean(),
        "array" => v.is_array(),
        "object" => v.is_object(),
        "null" => v.is_null(),
        _ => true,
    }
}

pub fn validate(schema: &Value, v: &Value, path: &str) -> Result<(), String> {
    let at = if path.is_empty() { "input".to_string() } else { path.to_string() };
    match &schema["type"] {
        Value::String(t) if !type_ok(t, v) => return Err(format!("{at} must be {t}")),
        Value::Array(ts) if !ts.iter().any(|t| type_ok(t.as_str().unwrap_or(""), v)) => {
            return Err(format!("{at} must be one of {}", ts.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>().join(", ")))
        }
        _ => {}
    }
    if let Some(e) = schema["enum"].as_array() {
        if !e.contains(v) {
            return Err(format!("{at} must be one of {}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")));
        }
    }
    if let Some(x) = v.as_f64() {
        if schema["minimum"].as_f64().is_some_and(|m| x < m) || schema["maximum"].as_f64().is_some_and(|m| x > m) {
            return Err(format!("{at} must be between {} and {}", schema["minimum"], schema["maximum"]));
        }
    }
    if let Some(s) = v.as_str() {
        let n = s.chars().count() as u64;
        if schema["minLength"].as_u64().is_some_and(|m| n < m) || schema["maxLength"].as_u64().is_some_and(|m| n > m) {
            return Err(format!("{at} has an invalid length"));
        }
    }
    if let Some(a) = v.as_array() {
        let n = a.len() as u64;
        if schema["minItems"].as_u64().is_some_and(|m| n < m) || schema["maxItems"].as_u64().is_some_and(|m| n > m) {
            return Err(format!("{at} must have between {} and {} items", schema["minItems"].as_u64().unwrap_or(0), schema["maxItems"]));
        }
        if schema["items"].is_object() {
            for (i, item) in a.iter().enumerate() {
                validate(&schema["items"], item, &format!("{at}[{i}]"))?;
            }
        }
    }
    if let Some(o) = v.as_object() {
        if let Some(req) = schema["required"].as_array() {
            for r in req.iter().filter_map(|r| r.as_str()) {
                if o.get(r).is_none_or(|x| x.is_null()) {
                    return Err(format!("{at}.{r} is required"));
                }
            }
        }
        if let Some(props) = schema["properties"].as_object() {
            for (k, s) in props {
                if let Some(x) = o.get(k).filter(|x| !x.is_null()) {
                    validate(s, x, &format!("{at}.{k}"))?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn subset_of_json_schema() {
        let s = json!({"type": "object", "required": ["role"], "properties": {
            "role": {"type": "string", "enum": ["bass", "lead"]},
            "bpm": {"type": "number", "minimum": 40, "maximum": 250},
            "progression": {"type": ["string", "array"], "items": {"type": "string"}},
            "parts": {"type": "array", "maxItems": 2, "items": {"type": "object", "required": ["name"]}}
        }});
        assert!(validate(&s, &json!({"role": "bass", "bpm": 124, "extra": true}), "").is_ok());
        assert!(validate(&s, &json!({"role": "bass", "progression": ["i", "VI"]}), "").is_ok());
        assert_eq!(validate(&s, &json!({}), "").unwrap_err(), "input.role is required");
        assert!(validate(&s, &json!({"role": "kazoo"}), "").unwrap_err().contains("must be one of"));
        assert!(validate(&s, &json!({"role": "bass", "bpm": 900}), "").is_err());
        assert!(validate(&s, &json!({"role": "bass", "progression": 4}), "").is_err());
        assert_eq!(validate(&s, &json!({"role": "bass", "parts": [{"x": 1}]}), "").unwrap_err(), "input.parts[0].name is required");
        assert!(validate(&s, &json!({"role": "bass", "parts": [{"name": 1}, {"name": 2}, {"name": 3}]}), "").is_err());
    }
}
