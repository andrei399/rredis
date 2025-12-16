use crate::{
    commands::operations::generic::get_dbvalue,
    storage::{Db, DbRef, DbValue, GetResult},
};
use std::{io, iter::zip, str::FromStr};
use tokio;

pub fn set_key(db: &mut DbRef<'_>, key: String, value: String) -> String {
    db.insert(key, DbValue::String(value.clone()));
    format!("+{value}\r\n")
}

pub fn set_expiration(
    storage: &Db,
    db: &mut DbRef<'_>,
    key: String,
    value: String,
    timeout: u64,
) -> String {
    db.insert(key.clone(), DbValue::String(value.clone()));
    let key_clone = key.clone();
    let duration_seconds = timeout.clone();
    let storage_clone = storage.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(duration_seconds)).await;
        let db = storage_clone.pin();
        db.remove(key_clone.as_str());
    });
    String::from_str("+OK\r\n").unwrap()
}

pub fn modify_integer(
    db: &mut DbRef<'_>,
    key: &str,
    operation: impl Fn(i64) -> i64,
) -> io::Result<String> {
    match get_dbvalue(db, key) {
        GetResult::FoundString(val) => {
            let parsed_value = val.parse::<i64>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "-ERROR: Value is not a valid integer",
                )
            })?;

            let new_value = operation(parsed_value);
            db.update(key.to_string(), |_| DbValue::String(new_value.to_string()));
            Ok(format!("+{new_value}\r\n"))
        }
        GetResult::NotFound(_) => Ok(format!("-ERROR: Key \"{}\" not found", key)),
        _ => Ok(String::from(
            "-ERROR: Cannot convert this type to an integer",
        )),
    }
}

pub fn multiple_get(db: &mut DbRef<'_>, keys: &[String]) -> String {
    let mut message = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        let val = db
            .get(key)
            .map(|v| format!("{}) {}", i + 1, v))
            .unwrap_or_else(|| format!("{}) (nil)", i + 1));
        message.push(val);
    }
    format!("+{}", message.join("\r\n"))
}

pub fn multiple_set(db: &mut DbRef<'_>, keys: &[String], values: &[String]) -> String {
    let mut message = Vec::new();
    for (i, (key, value)) in zip(keys, values).enumerate() {
        db.insert(key.clone(), DbValue::String(value.clone()));
        message.push(format!("{}) {}", i + 1, value));
    }
    format!("+{}", message.join("\r\n")).to_string()
}
