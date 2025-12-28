use std::iter::zip;

use crate::{commands::operations::generic::get_dbvalue, storage::DbRef};
use crate::storage::GetResult;
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
    let pinned_hash_map = hash_map.pin();
    for (field, value) in zip(fields, values) {
        pinned_hash_map.insert(field.to_owned(), value.to_owned());
        count += 1;
    };
    format!("+{count}")
}
