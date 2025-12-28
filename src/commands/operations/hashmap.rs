use std::iter::zip;
use std::str::FromStr;

use crate::commands::operations::errors::get_missing_key_error_message;
use crate::{commands::operations::generic::get_dbvalue, storage::DbRef};
use crate::storage::{GetResult, format_hashmap};
use papaya::HashMap;

pub fn set_hashmap(db: &mut DbRef<'_>, key: &str, fields: &[String], values: &[String]) -> String {
    let mut count = 0;
    let hash_map = match get_dbvalue(db, key) {
        GetResult::FoundHashMap(hash_map) => hash_map,
        GetResult::NotFound(_) => HashMap::new(),
        _ => {
            return format!("-ERROR: Key '{key}' is already assigned to an incompatible type")
        }
    };
    {
        let pinned_hash_map = hash_map.pin();
        for (field, value) in zip(fields, values) {
            pinned_hash_map.insert(field.to_owned(), value.to_owned());
            count += 1;
        };
    }
    db.insert(key.to_string(), crate::storage::DbValue::HashMap(hash_map));
    format!("+{count}")
}

pub fn get_field_from_hashmap(db: &mut DbRef, key: &str, field: &str) -> String {
    let hash_map = match get_dbvalue(db, key) {
        GetResult::FoundHashMap(hash_map) => hash_map,
        GetResult::NotFound(_) => return get_missing_key_error_message(key.to_string()),
        _ => return String::from_str("-ERROR: incompatible type").unwrap(),
    };
    let pinned_hash_map = hash_map.pin();
    let value = match pinned_hash_map.get(field) {
        Some(value) => value,
        None => return format!("-ERROR: Field {field} was not found"),
    };
    format!("+{}", value)
}

pub fn get_all_fields_from_hashmap(db: &mut DbRef, key: &str) -> String {
    match get_dbvalue(db, key) {
        GetResult::FoundHashMap(hash_map) => format!("+{}", format_hashmap(&hash_map)),
        GetResult::NotFound(_) => return get_missing_key_error_message(key.to_string()),
        _ => return String::from_str("-ERROR: incompatible type").unwrap(),
    }
}
