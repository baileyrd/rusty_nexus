//! Bases database engine for structured tabular note databases (.bases directories).

use rusty_json::{from_str, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Field definition in a Base schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseFieldSchema {
    pub name: String,
    pub field_type: String, // "string", "number", "boolean", "date", "list", "reference"
    pub required: bool,
}

/// Base schema contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseSchema {
    pub name: String,
    pub description: String,
    pub fields: Vec<BaseFieldSchema>,
}

/// Record item in a Base database.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseRecord {
    pub id: String,
    pub fields: HashMap<String, String>,
}

/// A parsed Base database instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    pub rel_path: String,
    pub schema: BaseSchema,
    pub records: Vec<BaseRecord>,
}

impl Base {
    /// Validate a record against the base schema.
    pub fn validate_record(&self, record: &BaseRecord) -> Result<(), String> {
        for field in &self.schema.fields {
            if field.required && !record.fields.contains_key(&field.name) {
                return Err(format!("Missing required field '{}' in base record", field.name));
            }
        }
        Ok(())
    }
}

/// Create a new Base database directory with metadata schema.
pub fn create_base(forge_root: &Path, rel_path: &str, schema: BaseSchema) -> Result<Base, String> {
    let abs_dir = forge_root.join(rel_path);
    if abs_dir.exists() {
        return Err(format!("Base directory '{}' already exists", rel_path));
    }

    fs::create_dir_all(&abs_dir).map_err(|e| e.to_string())?;

    let schema_json = format!(
        "{{\"name\":\"{}\",\"description\":\"{}\",\"fields\":[{}]}}",
        schema.name,
        schema.description,
        schema
            .fields
            .iter()
            .map(|f| format!(
                "{{\"name\":\"{}\",\"type\":\"{}\",\"required\":{}}}",
                f.name, f.field_type, f.required
            ))
            .collect::<Vec<_>>()
            .join(",")
    );

    let schema_file = abs_dir.join("schema.json");
    fs::write(&schema_file, schema_json).map_err(|e| e.to_string())?;

    let records_file = abs_dir.join("records.json");
    fs::write(&records_file, "[]").map_err(|e| e.to_string())?;

    Ok(Base {
        rel_path: rel_path.to_string(),
        schema,
        records: Vec::new(),
    })
}

/// Load an existing Base database from disk.
pub fn load_base(forge_root: &Path, rel_path: &str) -> Result<Base, String> {
    let abs_dir = forge_root.join(rel_path);
    let schema_file = abs_dir.join("schema.json");
    let records_file = abs_dir.join("records.json");

    if !schema_file.exists() {
        return Err(format!("schema.json missing in base directory '{}'", rel_path));
    }

    let schema_str = fs::read_to_string(&schema_file).map_err(|e| e.to_string())?;
    let schema_val: Value = from_str(&schema_str).map_err(|e| e.to_string())?;

    let mut schema = BaseSchema {
        name: "Untitled Base".to_string(),
        description: String::new(),
        fields: Vec::new(),
    };

    if let Value::Object(obj) = schema_val {
        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            schema.name = name.to_string();
        }
        if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
            schema.description = desc.to_string();
        }
        if let Some(Value::Array(fields_arr)) = obj.get("fields") {
            for f_val in fields_arr {
                if let Value::Object(f_obj) = f_val {
                    let name = f_obj.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let field_type = f_obj.get("type").and_then(|v| v.as_str()).unwrap_or("string").to_string();
                    let required = f_obj.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
                    schema.fields.push(BaseFieldSchema {
                        name,
                        field_type,
                        required,
                    });
                }
            }
        }
    }

    let mut records = Vec::new();
    if records_file.exists() {
        let records_str = fs::read_to_string(&records_file).map_err(|e| e.to_string())?;
        if let Ok(Value::Array(arr)) = from_str(&records_str) {
            for r_val in arr {
                if let Value::Object(r_obj) = r_val {
                    let id = r_obj.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let mut fields = HashMap::new();
                    for (k, v) in r_obj.iter() {
                        if k != "id" {
                            if let Some(s) = v.as_str() {
                                fields.insert(k.clone(), s.to_string());
                            } else if let Ok(s) = rusty_json::to_string(v) {
                                fields.insert(k.clone(), s);
                            }
                        }
                    }
                    records.push(BaseRecord { id, fields });
                }
            }
        }
    }

    Ok(Base {
        rel_path: rel_path.to_string(),
        schema,
        records,
    })
}

/// Append a record to a Base database and save to disk.
pub fn insert_base_record(forge_root: &Path, rel_path: &str, mut record: BaseRecord) -> Result<BaseRecord, String> {
    let mut base = load_base(forge_root, rel_path)?;
    if record.id.is_empty() {
        record.id = format!("rec_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos());
    }

    base.validate_record(&record)?;
    base.records.push(record.clone());

    let abs_dir = forge_root.join(rel_path);
    let records_file = abs_dir.join("records.json");

    let recs_json = base
        .records
        .iter()
        .map(|r| {
            let mut fields_str = vec![format!("\"id\":\"{}\"", r.id)];
            for (k, v) in &r.fields {
                fields_str.push(format!("\"{}\":\"{}\"", k, v.replace('"', "\\\"")));
            }
            format!("{{{}}}", fields_str.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");

    fs::write(&records_file, format!("[{}]", recs_json)).map_err(|e| e.to_string())?;

    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("base_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let schema = BaseSchema {
            name: "Projects".to_string(),
            description: "Active projects base".to_string(),
            fields: vec![
                BaseFieldSchema { name: "title".to_string(), field_type: "string".to_string(), required: true },
                BaseFieldSchema { name: "status".to_string(), field_type: "string".to_string(), required: false },
            ],
        };

        let base = create_base(&temp_dir, "projects.bases", schema).expect("create base");
        assert_eq!(base.schema.name, "Projects");

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "rusty_nexus".to_string());
        fields.insert("status".to_string(), "active".to_string());

        let rec = insert_base_record(&temp_dir, "projects.bases", BaseRecord { id: "p1".to_string(), fields }).expect("insert record");
        assert_eq!(rec.id, "p1");

        let loaded = load_base(&temp_dir, "projects.bases").expect("load base");
        assert_eq!(loaded.records.len(), 1);
        assert_eq!(loaded.records[0].fields.get("title").unwrap(), "rusty_nexus");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
