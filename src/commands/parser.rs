use std::str::SplitWhitespace;
use tokio::io::{self, Result, AsyncReadExt};
use tokio::net::tcp::OwnedReadHalf;
use crate::commands::core::Commands;

pub struct CommandParser<'a> {
    pub split: &'a mut SplitWhitespace<'a>,
}
impl CommandParser<'_> {
    fn base_parse(&mut self, param_name: &str) -> Result<&str> {
        let result = self.split.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("-ERROR: {param_name} parameter is required."),
            )
        })?;
        Ok(result)
    }
    pub fn parse_key(&mut self) -> Result<String> {
        Ok(self.base_parse("KEY")?.to_string())
    }
    pub fn parse_value(&mut self) -> Result<String> {
        Ok(self.base_parse("VALUE")?.to_string())
    }
    pub fn parse_seconds(&mut self) -> Result<u64> {
        let param_name = "SECONDS";
        let seconds = self.base_parse(param_name)?.parse::<u64>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: SECONDS parameter needs to be of type: u64",
            )
        })?;
        Ok(seconds)
    }
    pub fn parse_keys(&mut self) -> Result<Vec<String>> {
        let keys: Vec<String> = self.split.map(|s| s.to_string()).collect();
        if keys.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: MGET requires at least one KEY parameter.",
            ));
        }
        Ok(keys)
    }
    pub fn parse_key_value_pairs(&mut self) -> Result<(Vec<String>, Vec<String>)> {
        let args: Vec<String> = self.split.map(|s| s.to_string()).collect();
        if args.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: MSET requires at least one KEY VALUE pair parameter.",
            ));
        }
        if args.len() % 2 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: Uneven number of keys and values.",
            ));
        }
        let mut keys: Vec<String> = [].to_vec();
        let mut values: Vec<String> = [].to_vec();
        let mut i = 0;
        for arg in args {
            if i % 2 == 1 {
                values.push(arg.to_string());
            } else {
                keys.push(arg.to_string());
            }
            i += 1;
        }
        Ok((keys, values))
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
        let mut parser = CommandParser { split: &mut split };
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
}
