# 🦀 rredis: A Pure Rust Redis Clone

> [!CAUTION]
> This project is in early development, many features are missing and the project is littered with bugs

| Feature | Status |
| :--- | :--- |
| **Basic Commands** (`GET`, `SET`, etc.) | 🚧 In Progress |
| **TTL/Expiration** | ❌ Not Started |
| **Replication** | ❌ Not Started |
| `crates.io` | N/A (Server Application) |

-----

## 🌟 Overview

**rredis** is an open-source, from-scratch implementation of the **Redis** protocol and core data structures, written entirely in [Rust](https://www.rust-lang.org/). The goal is to create a performant, reliable, and educational implementation that can serve as a drop-in replacement for common use cases.

  * **Protocol:** Implements the [RESP (Redis Serialization Protocol)](https://redis.io/docs/latest/develop/reference/protocol-spec/) for full client compatibility.
  * **Performance:** Built on `tokio` for high-throughput, non-blocking I/O.
  * **Focus:** Safety, efficiency, and clear, modular code design.

## 🧱 Implemented Features & Data Structures

We aim to implement the most commonly used Redis features. The following is a summary of currently supported data types and commands:

### Data Structures

| Data Structure | Supported | Implemented Commands (Examples) |
| :--- | :--- | :--- |
| **Strings** | 🚧 | `GET`, `SET`, `DEL`, `INCR`, `DECR` |
| **Lists** | 🚧 | `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LRANGE` |
| **Hashes** | 🚧 | `HSET`, `HGET`, `HDEL` |
| **Sets** | ❌ | |

### Core Functionality

  * **Keyspace:** `DEL`, `EXISTS`, `KEYS`
  * **Connections:** `PING`, `ECHO`, `QUIT`

## 🚀 Getting Started

### Prerequisites

You must have a recent stable version of [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.

### Build and Run

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/your-github-username/rredis
    cd rredis
    ```

2.  **Build the server executable:**

    ```bash
    cargo build --release
    ```

3.  **Run the server:**

    ```bash
    ./target/release/rredis
    # Server running on 127.0.0.1:6379...
    ```

### Connecting with a Client

You can connect to your running `rredis` instance using any standard Redis client (like `redis-cli`) or any programming language client:

```bash
# In a new terminal window
redis-cli -p 6379

127.0.0.1:6379> PING
PONG
127.0.0.1:6379> SET rust_is_fast true
OK
127.0.0.1:6379> GET rust_is_fast
"true"
```

## 📐 Architecture

**rredis** is structured around the core components necessary for a high-performance network server:

1.  **Listener:** A single `tokio` task accepts incoming TCP connections.
2.  **Connection Handler:** A dedicated asynchronous task is spawned for each client connection to handle I/O and protocol parsing (RESP).
3.  **Data Store:** All client commands are forwarded to a shared, concurrently accessed main storage engine (protected by a mutex or similar synchronization primitive).

## 🤝 Contributing

This project is in active development, and contributions are highly valued\! We are currently focused on implementing the remaining core data structures and advanced features like transactions and persistence.

  * **[Issue Tracker](https://github.com/andrei399/rredis/issues)**

## 📜 License

This project is licensed under the **MIT License**. See the [LICENSE](./LICENSE.txt) file for details.
