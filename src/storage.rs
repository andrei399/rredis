use papaya::{HashMap, HashMapRef, LocalGuard};
use std::{collections::VecDeque, fmt, hash::RandomState, sync::Arc};

#[derive(Clone)]
pub enum DbValue {
    String(String),
    List(VecDeque<String>),
}
impl fmt::Display for DbValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbValue::String(string) => write!(f, "{}", string),
            DbValue::List(list) => {
                // Format the List as a comma-separated string enclosed in brackets
                write!(f, "[{}]", list.into_iter().cloned().collect::<Vec<String>>().join(", "))
            }
        }
    }
}

pub enum GetResult {
    FoundString(String),
    FoundList(VecDeque<String>),
    NotFound(String),
}
impl fmt::Display for GetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetResult::FoundString(string) => {
                write!(f, "+{}\r\n", string)
            }
            GetResult::FoundList(list) => {
                let list_str = format!("[{}]", list.into_iter().cloned().collect::<Vec<String>>().join(", "));
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
