use std::collections::VecDeque;
use std::str::FromStr;

use crate::commands::operations::errors::get_missing_key_error_message;
use crate::commands::operations::generic::get_dbvalue;
use crate::storage::{DbRef, DbValue, GetResult};

pub fn modify_list_in_db(
    db: &mut DbRef<'_>,
    key: &str,
    value: &str,
    operation: impl FnOnce(&mut VecDeque<String>, String),
) -> String {
    let current_value = get_dbvalue(db, key);
    let mut list = match current_value {
        GetResult::FoundList(list) => list,
        GetResult::NotFound(_) => VecDeque::new(),
        _ => {
            return format!("-ERROR: Key '{key}' is already assigned to an incompatible type.\r\n");
        }
    };
    operation(&mut list, value.to_string());
    db.insert(key.to_string(), DbValue::List(list.to_owned().into()));
    format!("+{:?}\r\n", list)
}

pub fn len_of_list_in_db(db: &mut DbRef<'_>, key: &str) -> String {
    match get_dbvalue(db, key) {
        GetResult::FoundList(list) => format!("+{:?}\r\n", list.len()),
        GetResult::NotFound(key) => get_missing_key_error_message(key),
        _ => String::from_str("-ERROR: Incompatible type").unwrap(),
    }
}

pub fn get_list_with_range(db: &mut DbRef<'_>, key: &str, start: usize, stop: usize) -> String {
    match get_dbvalue(db, key) {
        GetResult::FoundList(list) => format!(
            "+{:?}\r\n",
            list.range(start..stop).cloned().collect::<Vec<String>>()
        ),
        GetResult::NotFound(key) => get_missing_key_error_message(key),
        _ => String::from_str("-ERROR: Incompatible type").unwrap(),
    }
}
