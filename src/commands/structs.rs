use std::iter::zip;
use std::str::FromStr;
use tokio::io::Result;

use tokio::io::{self, AsyncReadExt};
use tokio::net::tcp::OwnedReadHalf;

use crate::commands::parser::Parser;
use crate::storage::{Db, DbRef, DbValue, GetResult};

pub enum Commands {
    Get {
        key: String,
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
    pub fn execute_get(db: &mut DbRef<'_>, key: &str) -> GetResult {
        match db.get(key) {
            Some(db_value) => match db_value {
                DbValue::String(s) => GetResult::FoundString(s.clone()),
                DbValue::List(l) => GetResult::FoundList(l.clone()),
            },
            None => GetResult::NotFound(key.to_string()),
        }
    }
    pub async fn parse_command(mut read_half: OwnedReadHalf) -> io::Result<Commands> {
        let mut buffer = [0u8; 1024];
        let n = read_half.read(&mut buffer).await?;

        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "-ERROR: Client sent no data",
            ));
        }

        let input = String::from_utf8_lossy(&buffer[..n]);
        let mut split = input.split_whitespace();
        let command_type = split.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "-ERROR: Missing command")
        })?;
        let mut parser = Parser { split: &mut split };
        match command_type.to_uppercase().as_str() {
            "GET" => Ok(Commands::Get {
                key: parser.parse_key()?,
            }),
            "SET" => Ok(Commands::Set {
                key: parser.parse_key()?,
                value: parser.parse_value()?,
            }),
            "SETEX" => Ok(Commands::Setex {
                key: parser.parse_key()?,
                seconds: parser.parse_seconds()?,
                value: parser.parse_value()?,
            }),
            "DEL" => Ok(Commands::Del {
                key: parser.parse_key()?,
            }),
            "EXISTS" => Ok(Commands::Exists {
                key: parser.parse_key()?,
            }),
            "INCR" => Ok(Commands::Incr {
                key: parser.parse_key()?,
            }),
            "DECR" => Ok(Commands::Decr {
                key: parser.parse_key()?,
            }),
            "MGET" => Ok(Commands::Mget {
                keys: parser.parse_keys()?,
            }),
            "MSET" => {
                let (keys, values) = parser.parse_key_value_pairs()?;
                Ok(Commands::Mset { keys, values })
            }
            "LPUSH" => Ok(Commands::Lpush {
                key: parser.parse_key()?,
                value: parser.parse_value()?,
            }),
            "RPUSH" => Ok(Commands::Rpush {
                key: parser.parse_key()?,
                value: parser.parse_value()?,
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: Unknown command.",
            )),
        }
    }

    fn modify_integer_value(
        db: &mut DbRef<'_>,
        key: &str,
        operation: impl Fn(i64) -> i64,
    ) -> Result<String> {
        match Commands::execute_get(db, key) {
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
            _ => Ok(String::from_str("-ERROR: Cannot convert this type to an integer").unwrap()),
        }
    }

    fn push_list_element(
        db: &mut DbRef<'_>,
        key: &str,
        value: &str,
        operation: impl FnOnce(&mut Vec<String>, String),
    ) -> String {
        let current_value = Commands::execute_get(db, key);
        match current_value {
            GetResult::FoundList(mut list) => {
                operation(&mut list, value.to_string());
                db.insert(key.to_string(), DbValue::List(list.clone()));
                format!("+{:?}\r\n", list)
            }
            GetResult::NotFound(_) => {
                let mut list = Vec::new();
                operation(&mut list, value.to_string());
                let new_value = DbValue::List(list.clone());
                db.insert(key.to_string(), new_value);
                format!("+{:?}\r\n", list)
            }
            _ => format!("-ERROR: Key '{key}' is already assigned to an incompatible type.\r\n"),
        }
    }

    pub async fn execute(&mut self, storage: &Db) -> Result<String> {
        let mut db = storage.pin();
        let result = match self {
            Commands::Get { key } => db
                .get(key)
                .map(|val| format!("+{}\r\n", val))
                .unwrap_or_else(|| format!("-ERROR: Key \"{key}\" not found").to_string()),
            Commands::Set { key, value } => {
                db.insert(key.clone(), DbValue::String(value.clone()));
                format!("+{value}\r\n")
            }
            Commands::Setex {
                key,
                seconds,
                value,
            } => {
                db.insert(key.clone(), DbValue::String(value.clone()));
                drop(db);
                let key_clone = key.clone();
                let duration_seconds = *seconds;
                let storage_clone = storage.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(duration_seconds)).await;
                    let db = storage_clone.pin();
                    db.remove(key_clone.as_str());
                    drop(db);
                });
                String::from_str("+OK\r\n").unwrap()
            }
            Commands::Del { key } => {
                db.remove(key);
                String::from_str("+OK\r\n").unwrap()
            }
            Commands::Exists { key } => match Commands::execute_get(&mut db, key) {
                GetResult::NotFound(_) => String::from_str("+false\r\n").unwrap(),
                _ => String::from_str("+true\r\n").unwrap(),
            },
            Commands::Incr { key } => Commands::modify_integer_value(&mut db, &key, |x| x + 1)
                .unwrap_or_else(|e| format!("-ERROR: {}", e.kind())),
            Commands::Decr { key } => Commands::modify_integer_value(&mut db, &key, |x| x - 1)
                .unwrap_or_else(|e| format!("-ERROR: {}", e.kind())),
            Commands::Mget { keys } => {
                let mut message: Vec<String> = [].to_vec();
                for (i, key) in keys.iter().enumerate() {
                    message.push(
                        db.get(&key.clone())
                            .map(|v| format!("{}) {}", i + 1, v).to_string())
                            .unwrap_or_else(|| format!("{}) (nil)", i + 1).to_string()),
                    );
                }
                format!("+{}", message.join("\r\n")).to_string()
            }
            Commands::Mset { keys, values } => {
                let mut message: Vec<String> = [].to_vec();
                for (i, (key, value)) in zip(keys, values).enumerate() {
                    db.insert(key.clone(), DbValue::String(value.clone()));
                    message.push(format!("{}) {}", i + 1, value));
                }
                format!("+{}", message.join("\r\n")).to_string()
            }
            Commands::Lpush { key, value } => {
                Commands::push_list_element(&mut db, key, value, |list, value| {
                    list.insert(0, value.to_string())
                })
            }
            Commands::Rpush { key, value } => {
                Commands::push_list_element(&mut db, key, value, |list, value| {
                    list.push(value.to_string())
                })
            }
        };
        Ok(result)
    }
}
