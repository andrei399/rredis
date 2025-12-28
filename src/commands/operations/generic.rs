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
            DbValue::String(s) => GetResult::FoundString(s.to_owned()),
            DbValue::List(l) => GetResult::FoundList(l.to_owned()),
            DbValue::Set(s) => GetResult::FoundSet(s.to_owned()),
            DbValue::HashMap(h) => GetResult::FoundHashMap(h.to_owned())
        },
        None => GetResult::NotFound(key.to_string()),
    };
    result
}
