use crate::commands::operations::generic::{del_key_in_db, exists_in_db, get_dbvalue};
use crate::commands::operations::hashmap::{
    delete_fields_from_hashmap, get_all_fields_from_hashmap, get_field_from_hashmap, set_hashmap
};
use crate::commands::operations::lists::{
    get_list_with_range, len_of_list_in_db, modify_list_in_db,
};
use crate::commands::operations::sets::{
    add_elements_to_set, is_member, remove_elements_from_set, show_set_memebers,
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
    Sadd {
        key: String,
        items: Vec<String>,
    },
    Srem {
        key: String,
        items: Vec<String>,
    },
    Smembers {
        key: String,
    },
    Sismember {
        key: String,
        value: String,
    },
    Hset {
        key: String,
        fields: Vec<String>,
        values: Vec<String>,
    },
    Hget {
        key: String,
        field: String,
    },
    Hgetall {
        key: String,
    },
    Hdel {
        key: String,
        fields: Vec<String>,
    },
}

impl Commands {
    pub async fn execute(&mut self, storage: &Db) -> Result<String> {
        let mut db = storage.pin();
        let result = match self {
            Commands::Get { key } => format!("{}", get_dbvalue(&mut db, &key)),
            Commands::Getset { key, value } => {
                let result = format!("{}", get_dbvalue(&mut db, &key));
                set_key(&mut db, key.to_owned(), value.to_owned());
                result
            }
            Commands::Set { key, value } => set_key(&mut db, key.to_owned(), value.to_owned()),
            Commands::Setex {
                key,
                seconds,
                value,
            } => set_expiration(
                storage,
                &mut db,
                key.to_owned(),
                value.to_owned(),
                seconds.to_owned(),
            ),
            Commands::Del { key } => del_key_in_db(&mut db, &key),
            Commands::Exists { key } => exists_in_db(&mut db, &key),
            Commands::Incr { key } => modify_integer(&mut db, &key, |x| x + 1)
                .unwrap_or_else(|e| format!("-ERROR: {}", e.kind())),
            Commands::Decr { key } => modify_integer(&mut db, &key, |x| x - 1)
                .unwrap_or_else(|e| format!("-ERROR: {}", e.kind())),
            Commands::Mget { keys } => multiple_get(&mut db, keys),
            Commands::Mset { keys, values } => multiple_set(&mut db, keys, values),
            Commands::Append { key, value } => append(&mut db, key.to_owned(), value.to_owned()),
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
                get_list_with_range(&mut db, key, start.to_owned(), stop.to_owned())
            }
            Commands::Sadd { key, items } => add_elements_to_set(&mut db, key, items),
            Commands::Srem { key, items } => remove_elements_from_set(&mut db, key, items),
            Commands::Smembers { key } => show_set_memebers(&mut db, key),
            Commands::Sismember { key, value } => is_member(&mut db, key, value),
            Commands::Hset {
                key,
                fields,
                values,
            } => set_hashmap(&mut db, key, fields, values),
            Commands::Hget { key, field } => get_field_from_hashmap(&mut db, key, field),
            Commands::Hgetall { key } => get_all_fields_from_hashmap(&mut db, key),
            Commands::Hdel { key, fields, } => delete_fields_from_hashmap(&mut db, key, fields),
        };
        Ok(result)
    }
}
