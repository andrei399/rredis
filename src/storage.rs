use papaya::{HashMap, HashMapRef, LocalGuard};
use std::{fmt, hash::RandomState, sync::Arc};

#[derive(Clone)]
pub enum DbValue {
    String(String),
    List(Vec<String>),
}
impl fmt::Display for DbValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbValue::String(s) => write!(f, "{}", s),
            DbValue::List(l) => {
                // Format the List as a comma-separated string enclosed in brackets
                write!(f, "[{}]", l.join(", "))
            }
        }
    }
}

pub enum GetResult {
    FoundString(String),
    FoundList(Vec<String>),
    NotFound(String),
}
impl fmt::Display for GetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetResult::FoundString(string) => {
                write!(f, "+{}\r\n", string)
            }
            GetResult::FoundList(list) => {
                let list_str = format!("[{}]", list.join(", "));
                write!(f, "+{}\r\n", list_str)
            }

            GetResult::NotFound(key) => {
                write!(f, "-ERROR: Key \"{}\" not found\r\n", key)
            }
        }
    }
}

pub type Db = Arc<HashMap<String, DbValue>>;
pub type DbRef<'a> = HashMapRef<'a, String, DbValue, RandomState, LocalGuard<'a>>;
