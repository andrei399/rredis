use std::collections::VecDeque;
use std::str::FromStr;

use crate::commands::operations::generic::get_dbvalue;
use crate::storage::{DbRef, DbValue, GetResult};

pub fn modify_list_in_db(
    db: &mut DbRef<'_>,
    key: &str,
    value: &str,
    operation: impl FnOnce(&mut VecDeque<String>, String),
) -> String {
    let current_value = get_dbvalue(db, key);
    match current_value {
        GetResult::FoundList(mut list) => {
            operation(&mut list, value.to_string());
            db.insert(key.to_string(), DbValue::List(list.clone().into()));
            format!("+{:?}\r\n", list)
        }
        GetResult::NotFound(_) => {
            let mut list = VecDeque::new();
            operation(&mut list, value.to_string());
            let new_value = DbValue::List(list.clone().into());
            db.insert(key.to_string(), new_value);
            format!("+{:?}\r\n", list)
        }
        _ => format!("-ERROR: Key '{key}' is already assigned to an incompatible type.\r\n"),
    }
}

pub fn len_of_list_in_db(db: &mut DbRef<'_>, key: &str) -> String {
    match get_dbvalue(db, key) {
        GetResult::FoundList(list) => format!("+{:?}\r\n", list.len()),
        _ => String::from_str("-ERROR: Key not found or value is of another incompatible type")
            .unwrap(),
    }
}

pub fn get_list_with_range(db: &mut DbRef<'_>, key: &str, start: usize, stop: usize) -> String {
    match get_dbvalue(db, key) {
        GetResult::FoundList(list) => format!(
            "+{:?}\r\n",
            list.range(start..stop).cloned().collect::<Vec<String>>()
        ),
        _ => String::from_str("-ERROR: Key not found or value is of another incompatible type")
            .unwrap(),
    }
}
