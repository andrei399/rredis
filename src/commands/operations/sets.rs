use std::str::FromStr;
use papaya::{HashSet}; 
use crate::{commands::operations::generic::get_dbvalue, storage::{DbRef, DbValue, GetResult}};

pub fn add_elements(db: &mut DbRef, key: &str, items: &[String]) -> String {
    let set: HashSet<String> = match get_dbvalue(db, key) {
        GetResult::FoundSet(set_arc) => set_arc,
        GetResult::NotFound(_) => HashSet::new(),
        _ => return String::from_str("-ERROR: Incorrect Type.").unwrap(),
    };
    
    let pinned_set = set.pin();
    for item in items {
        pinned_set.insert(item.clone());
    };
    if let GetResult::NotFound(_) = get_dbvalue(db, key) {
        db.insert(key.to_string(), DbValue::Set(set.clone()));
    };
    format!("+{{{}}}\r\n", pinned_set.iter().map(|x| x.as_str()).collect::<Vec<_>>().join(", "))
}
