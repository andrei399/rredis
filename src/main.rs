use papaya::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, TcpStream};
use std::thread;

type Db = Arc<HashMap<String, String>>;

enum Commands {
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
}

impl Commands {
    async fn parse_command(mut read_half: OwnedReadHalf) -> io::Result<Commands> {
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
        let command_type = split
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "-ERROR: Missing command"))?;
        match command_type.to_uppercase().as_str() {
            "GET" => {
                let key = split
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "-ERROR: GET requires a key")
                    })?
                    .to_string();
                Ok(Commands::Get { key })
            }
            "SET" => {
                let key = split
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: SET requires a key as the first parameter",
                        )
                    })?
                    .to_string();
                let value = split
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: SET requires a value as the second parameter",
                        )
                    })?
                    .to_string();
                Ok(Commands::Set { key, value })
            }
            "SETEX" => {
                let key = split
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: SET requires a key as the first parameter",
                        )
                    })?
                    .to_string();
                let seconds = split
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: SETEX requires 'seconds' as the second parameter",
                        )
                    })?
                    .parse::<u64>()
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: SETEX 'seconds' parameter needs to be of type: u64",
                        )
                    })?;
                let value = split
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "-ERROR: SET requires a value as the third parameter",
                        )
                    })?
                    .to_string();
                Ok(Commands::Setex {
                    key,
                    seconds,
                    value,
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: Unknown command.",
            )),
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6969").await?;
    let storage: Db = Arc::new(HashMap::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        let storage_clone = storage.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_request(socket, &storage_clone).await {
                eprintln!("Error handling request from {}: {}", addr, e)
            }
        });
    }
}

async fn handle_request(socket: TcpStream, storage: &Db) -> io::Result<()> {
    let (read_half, mut write_half) = socket.into_split();
    let command = match Commands::parse_command(read_half).await {
        Ok(cmd) => cmd,
        Err(e) => {
            let error_msg = format!("Error: {}\r\n", e);
            write_half.write_all(error_msg.as_bytes()).await?;
            return Ok(());
        }
    };

    let response = {
        let db = storage.pin();
        match command {
            Commands::Get { key } => db
                .get(&key)
                .map(|val| format!("+{}\r\n", val))
                .unwrap_or_else(|| "-ERROR: Key \"{key}\" not found".to_string()),
            Commands::Set { key, value } => {
                db.insert(key, value);
                String::from_str("+OK\r\n").unwrap()
            },
            Commands::Setex { key, seconds, value } => {
                db.insert(key.clone(), value);
                drop(db);
                let storage_clone = storage.clone();
                tokio::spawn(async move {
                    thread::sleep(Duration::from_secs(seconds));
                    let db = storage_clone.pin();
                    db.remove(key.as_str());
                    drop(db);
                });
                String::from_str("+OK\r\n").unwrap()
            }
        }
    };
    write_half.write_all(response.as_bytes()).await?;
    write_half.flush().await?;
    Ok(())
}
