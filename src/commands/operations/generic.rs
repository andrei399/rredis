use crate::storage::{DbRef, DbValue, GetResult};

pub fn del_key_in_db(db: &mut DbRef<'_>, key: &str) -> String {
    db.remove(key);
    String::from("+OK\r\n")
}

pub fn exists_in_db(db: &mut DbRef<'_>, key: &str) -> String {
    match get_dbvalue(db, key) {
        GetResult::NotFound(_) => String::from("+false\r\n"),
        _ => String::from("+true\r\n"),
    }
}

pub fn get_dbvalue(db: &mut DbRef<'_>, key: &str) -> GetResult {
    let result = match db.get(key) {
        Some(db_value) => match db_value {
            DbValue::String(s) => GetResult::FoundString(s.clone()),
            DbValue::List(l) => GetResult::FoundList(l.clone()),
            DbValue::Set(s) => GetResult::FoundSet(s.clone()),
        },
        None => GetResult::NotFound(key.to_string()),
    };
    result
}
