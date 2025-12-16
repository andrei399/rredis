use crate::commands::operations::generic::get_dbvalue;
use crate::storage::{DbRef, DbValue, GetResult};

pub fn modify_list_in_db(
    db: &mut DbRef<'_>,
    key: &str,
    value: &str,
    operation: impl FnOnce(&mut Vec<String>, String),
) -> String {
    let current_value = get_dbvalue(db, key);
    match current_value {
        GetResult::FoundList(mut list) => {
            operation(&mut list, value.to_string());
            db.insert(key.to_string(), DbValue::List(list.clone()));
            format!("+{:?}\r\n", list)
        }
        GetResult::NotFound(_) => {
            let mut list = Vec::new();
            operation(&mut list, value.to_string());
            let new_value = DbValue::List(list.clone());
            db.insert(key.to_string(), new_value);
            format!("+{:?}\r\n", list)
        }
        _ => format!("-ERROR: Key '{key}' is already assigned to an incompatible type.\r\n"),
    }
}
