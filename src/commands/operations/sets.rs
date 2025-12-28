use crate::{
    commands::operations::{errors::get_missing_key_error_message, generic::get_dbvalue},
    storage::{DbRef, DbValue, GetResult},
};
use papaya::HashSet;
use std::str::FromStr;

fn get_set_by_key(db: &mut DbRef, key: &str) -> Result<HashSet<String>, String> {
    match get_dbvalue(db, key) {
        GetResult::FoundSet(set_arc) => Ok(set_arc),
        GetResult::NotFound(_) => Ok(HashSet::new()),
        _ => Err(String::from_str("-ERROR: Incorrect Type.").unwrap()),
    }
}

pub fn add_elements_to_set(db: &mut DbRef, key: &str, items: &[String]) -> String {
    let set = match get_set_by_key(db, key) {
        Ok(set) => set,
        Err(e) => return e,
    };
    let mut count = 0;
    let pinned_set = set.pin();
    for item in items {
        if pinned_set.insert(item.to_owned()) {
            count += 1
        };
    }
    db.insert(key.to_string(), DbValue::Set(set.to_owned()));
    format!("+{}\r\n", count)
}

pub fn remove_elements_from_set(db: &mut DbRef, key: &str, items: &[String]) -> String {
    let set = match get_dbvalue(db, key) {
        GetResult::FoundSet(set) => set,
        GetResult::NotFound(k) => return get_missing_key_error_message(k),
        _ => return String::from_str("-ERROR: Incompatible type").unwrap(),
    };
    let mut count = 0;
    let pinned_set = set.pin();

    for item in items {
        if pinned_set.remove(item) {
            count += 1
        };
    }
    db.insert(key.to_string(), DbValue::Set(set.to_owned()));
    format!("+{}\r\n", count)
}

pub fn show_set_memebers(db: &mut DbRef, key: &str) -> String {
    let set = match get_set_by_key(db, key) {
        Ok(set) => set,
        Err(e) => return e,
    };
    format!(
        "+{{{}}}\r\n",
        set.pin()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn is_member(db: &mut DbRef, key: &str, value: &str) -> String {
    let set = match get_set_by_key(db, key) {
        Ok(set) => set,
        Err(e) => return e,
    };
    match set.pin().contains(value) {
        true => format!("+1"),
        false => format!("+0"),
    }
}
