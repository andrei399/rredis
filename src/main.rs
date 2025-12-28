use papaya::HashMap;
use ini::Ini;

use crate::commands::parser::CommandParser;
use crate::storage::Db;
use std::sync::Arc;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
pub mod commands;
pub mod storage;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Ini::load_from_file("config.ini").unwrap();
    let server_section = config.section(Some("server")).unwrap();
    let port = server_section.get("PORT").and_then(|p| p.parse().ok()).unwrap_or(6379);
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;
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
    let mut command = match CommandParser::parse_command(read_half).await {
        Ok(cmd) => cmd,
        Err(e) => {
            write_half.write_all(format!("{e}\r\n").as_bytes()).await?;
            return Ok(());
        }
    };
    let result = match command.execute(&storage).await {
        Ok(result) => result,
        Err(e) => {
            write_half.write_all(format!("{e}\r\n").as_bytes()).await?;
            return Ok(());
        }
    };

    write_half.write_all(result.as_bytes()).await?;
    write_half.flush().await?;
    Ok(())
}
