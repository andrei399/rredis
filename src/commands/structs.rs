use papaya::HashMap;
use std::str::{FromStr, SplitWhitespace};
use std::sync::Arc;
use tokio::io::Result;

use tokio::io::{self, AsyncReadExt};
use tokio::net::tcp::OwnedReadHalf;

struct Parser<'a> {
    split: &'a mut SplitWhitespace<'a>,
}
impl Parser<'_> {
    fn base_parse(&mut self, param_name: &str) -> Result<&str> {
        let result = self.split.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("-ERROR: {param_name} parameter is required."),
            )
        })?;
        Ok(result)
    }
    fn parse_key(&mut self) -> Result<String> {
        Ok(self.base_parse("KEY")?.to_string())
    }
    fn parse_value(&mut self) -> Result<String> {
        Ok(self.base_parse("VALUE")?.to_string())
    }
    fn parse_seconds(&mut self) -> Result<u64> {
        let param_name = "SECONDS";
        let seconds = self.base_parse(param_name)?.parse::<u64>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: SECONDS parameter needs to be of type: u64",
            )
        })?;
        Ok(seconds)
    }
}

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
    Incr {
        key: String,
    },
    Decr {
        key: String,
    },
}

impl Commands {
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
            "INCR" => Ok(Commands::Incr {
                key: parser.parse_key()?,
            }),
            "DECR" => Ok(Commands::Decr {
                key: parser.parse_key()?,
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: Unknown command.",
            )),
        }
    }

    pub async fn execute(&mut self, storage: &Arc<HashMap<String, String>>) -> Result<String> {
        let db = storage.pin();
        let result = match self {
            Commands::Get { key } => db
                .get(key)
                .map(|val| format!("+{}\r\n", val))
                .unwrap_or_else(|| format!("-ERROR: Key \"{key}\" not found").to_string()),
            Commands::Set { key, value } => {
                db.insert(key.clone(), value.clone());
                format!("+{value}\r\n")
            }
            Commands::Setex {
                key,
                seconds,
                value,
            } => {
                db.insert(key.clone(), value.clone());
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
            Commands::Incr { key } => match db.get(key) {
                Some(val) => {
                    let parsed_value = val.parse::<i64>().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: Value is not a valid integer",
                        )
                    })?;
                    let new_value = parsed_value + 1;
                    db.update(key.to_string(), |_| new_value.to_string());
                    format!("+{new_value}\r\n")
                }
                None => format!("-ERROR: Key \"{}\" not found", &key),
            },
            Commands::Decr { key } => match db.get(key) {
                Some(val) => {
                    let parsed_value = val.parse::<i64>().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: Value is not a valid integer",
                        )
                    })?;
                    let new_value = parsed_value - 1;
                    db.update(key.to_string(), |_| new_value.to_string());
                    format!("+{new_value}\r\n")
                }
                None => format!("-ERROR: Key \"{}\" not found", &key),
            },
        };
        Ok(result)
    }
}
