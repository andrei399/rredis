use crate::commands::operations::generic::{del_key_in_db, exists_in_db, get_dbvalue};
use crate::commands::operations::lists::{
    get_list_with_range, len_of_list_in_db, modify_list_in_db,
};
use crate::commands::operations::strings::{
    append, modify_integer, multiple_get, multiple_set, set_expiration, set_key,
};
use crate::storage::Db;
use tokio::io::Result;

pub enum Commands {
    Get {
        key: String,
    },
    Getset {
        key: String,
        value: String,
    },
    Set {
        key: String,
        value: String,
    },
    Setex {
        key: String,
        seconds: u64,
        value: String,
    },
    Del {
        key: String,
    },
    Exists {
        key: String,
    },
    Incr {
        key: String,
    },
    Decr {
        key: String,
    },
    Mget {
        keys: Vec<String>,
    },
    Mset {
        keys: Vec<String>,
        values: Vec<String>,
    },
    Append {
        key: String,
        value: String,
    },
    Lpush {
        key: String,
        value: String,
    },
    Rpush {
        key: String,
        value: String,
    },
    Lpop {
        key: String,
    },
    Rpop {
        key: String,
    },
    Llen {
        key: String,
    },
    Lrange {
        key: String,
        start: usize,
        stop: usize,
    },
}

impl Commands {
    pub async fn execute(&mut self, storage: &Db) -> Result<String> {
        let mut db = storage.pin();
        let result = match self {
            Commands::Get { key } => format!("{}", get_dbvalue(&mut db, &key)),
            Commands::Getset { key, value } => {
                let result = format!("{}", get_dbvalue(&mut db, &key));
                set_key(&mut db, key.clone(), value.clone());
                result
            }
            Commands::Set { key, value } => set_key(&mut db, key.clone(), value.clone()),
            Commands::Setex {
                key,
                seconds,
                value,
            } => set_expiration(
                storage,
                &mut db,
                key.clone(),
                value.clone(),
                seconds.clone(),
            ),
            Commands::Del { key } => del_key_in_db(&mut db, &key),
            Commands::Exists { key } => exists_in_db(&mut db, &key),
            Commands::Incr { key } => modify_integer(&mut db, &key, |x| x + 1)
                .unwrap_or_else(|e| format!("-ERROR: {}", e.kind())),
            Commands::Decr { key } => modify_integer(&mut db, &key, |x| x - 1)
                .unwrap_or_else(|e| format!("-ERROR: {}", e.kind())),
            Commands::Mget { keys } => multiple_get(&mut db, keys),
            Commands::Mset { keys, values } => multiple_set(&mut db, keys, values),
            Commands::Append { key, value } => append(&mut db, key.clone(), value.clone()),
            Commands::Lpush { key, value } => {
                modify_list_in_db(&mut db, key, value, |list, value| {
                    list.push_front(value.to_string());
                })
            }
            Commands::Rpush { key, value } => {
                modify_list_in_db(&mut db, key, value, |list, value| {
                    list.push_back(value.to_string());
                })
            }
            Commands::Lpop { key } => modify_list_in_db(&mut db, key, "", |list, _| {
                list.pop_front();
            }),
            Commands::Rpop { key } => modify_list_in_db(&mut db, key, "", |list, _| {
                list.pop_back();
            }),
            Commands::Llen { key } => len_of_list_in_db(&mut db, key),
            Commands::Lrange { key, start, stop } => {
                get_list_with_range(&mut db, key, start.clone(), stop.clone())
            }
        };
        Ok(result)
    }
}
