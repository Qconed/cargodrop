# CargoDrop TCP Network Layer

This repository now includes a TCP-based network layer for direct file transfer between two devices on the same LAN.

## Features

- Thread-safe receiver (`TcpServer`) with one thread per incoming connection
- Sender client (`TcpClient`) for streaming files in chunks over TCP
- JSON handshake with metadata before transfer starts
- Progress reporting through `mpsc::channel`
- Receiver saves files into `./received/`

## Dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Build

```bash
cargo build
```

## Local Network Test (2 PCs)

### On the RECEIVER PC (example IP: 192.168.1.10)

```bash
cargo run -- receive --port 5001
```

Expected behavior:
- Starts TCP server on port `5001`
- Waits for incoming connection
- Saves incoming file under `./received/`
- Prints progress logs such as:

```text
[1742895600] Receiving... 45% (2.30 MB / 5.10 MB)
```

### On the SENDER PC (example IP: 192.168.1.20)

```bash
cargo run -- send --ip 192.168.1.10 --port 5001 --file ./photo.jpg
```

Expected behavior:
- Connects to receiver at `192.168.1.10:5001`
- Sends handshake containing device name and file metadata
- Streams file bytes in chunks of 4096 bytes
- Prints progress logs such as:

```text
[1742895601] Sending... 72% (3.60 MB / 5.10 MB)
```

## CLI Usage

```bash
cargo run -- receive --port <PORT>
cargo run -- send --ip <IP> --port <PORT> --file <FILE_PATH>
```

- `receive`: run in receiver mode
- `send`: run in sender mode
- `--port`: optional, default is `5001`
- `--ip`: required for sender mode
- `--file`: required for sender mode

If invalid arguments are provided, the program prints a clear usage message and exits.