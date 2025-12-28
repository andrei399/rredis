use papaya::{HashMap, HashMapRef, HashSet, LocalGuard};
use std::{collections::VecDeque, fmt, hash::RandomState, sync::Arc};

fn format_list(list: &VecDeque<String>) -> String {
    format!(
        "[{}]",
        list.into_iter()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn format_set(set: &HashSet<String>) -> String {
    format!(
        "{{{}}}",
        set.pin()
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn format_hashmap_line(count: i32, value: &String) -> String {
    let mut result = String::new();
    result.push_str(&count.to_string());
    result.push_str(") ");
    result.push_str(value);
    result.push_str("\r\n");
    result
}

pub fn format_hashmap(hash_map: &HashMap<String, String>) -> String {
    let mut hash_map_str = String::new();
    let mut count = 1;
    for item in hash_map.pin().iter() {
        hash_map_str += &format_hashmap_line(count, item.0);
        count += 1;
        hash_map_str += &format_hashmap_line(count, item.1);
        count += 1;
    }
    hash_map_str
}

#[derive(Clone)]
pub enum DbValue {
    String(String),
    List(VecDeque<String>),
    Set(HashSet<String>),
    HashMap(HashMap<String, String>),
}
impl fmt::Display for DbValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbValue::String(string) => write!(f, "{}", string),
            DbValue::List(list) => write!(f, "{}", format_list(list)),
            DbValue::Set(set) => write!(f, "+{}\r\n", format_set(set)),
            DbValue::HashMap(hash_map) => write!(f, "+{}", format_hashmap(hash_map)),
        }
    }
}

pub enum GetResult {
    FoundString(String),
    FoundList(VecDeque<String>),
    FoundSet(HashSet<String>),
    FoundHashMap(HashMap<String, String>),
    NotFound(String),
}
impl fmt::Display for GetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetResult::FoundString(string) => write!(f, "+{}\r\n", string),
            GetResult::FoundList(list) => write!(f, "+{}\r\n", format_list(list)),
            GetResult::FoundSet(set) => write!(f, "+{}\r\n", format_set(set)),
            GetResult::FoundHashMap(hash_map) => write!(f, "+{}", format_hashmap(hash_map)),
            GetResult::NotFound(key) => write!(f, "-ERROR: Key \"{}\" not found\r\n", key),
        }
    }
}

pub type Db = Arc<HashMap<String, DbValue>>;
pub type DbRef<'a> = HashMapRef<'a, String, DbValue, RandomState, LocalGuard<'a>>;
