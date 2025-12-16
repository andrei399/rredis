use tokio::io::Result;
use crate::commands::operations::generic::{del_key_in_db, exists_in_db, get_dbvalue};
use crate::commands::operations::lists::modify_list_in_db;
use crate::commands::operations::strings::{
    append, modify_integer, multiple_get, multiple_set, set_expiration, set_key
};
use crate::storage::Db;

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
        value: String
    },
    Lpush {
        key: String,
        value: String,
    },
    Rpush {
        key: String,
        value: String,
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
            },
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
                    list.insert(0, value.to_string())
                })
            }
            Commands::Rpush { key, value } => {
                modify_list_in_db(&mut db, key, value, |list, value| {
                    list.push(value.to_string())
                })
            }
        };
        Ok(result)
    }
}
