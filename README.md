# Yjs Collaboration Server

English | [中文](README_zh.md)

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-performance, real-time collaborative document editing server built with Rust, Yrs (Yjs Rust implementation), and Volo HTTP. This server implements the Yjs protocol to enable seamless real-time collaboration between multiple clients.

## ✨ Features

- 🚀 **Real-time Collaboration**: Multiple users can edit documents simultaneously
- 🔄 **CRDT-based**: Conflict-free Replicated Data Types ensure consistency
- ⚡ **High Performance**: Built with Rust for maximum efficiency
- 🌐 **WebSocket Support**: Real-time bidirectional communication
- 🏗️ **Clean Architecture**: Well-structured and maintainable codebase
- 🔒 **Type Safety**: Leveraging Rust's powerful type system

## 📦 Dependencies

- **volo**: Core HTTP/RPC framework powering the server.
- **volo-http**: Full-featured HTTP utilities for routing and middleware.
- **tokio**: Asynchronous runtime for concurrency.
- **futures-util**: Utilities and combinators for async futures.
- **yrs**: Rust implementation of the Yjs CRDT protocol.
- **serde**: Data serialization framework with derive support.
- **sonic-rs**: High-performance JSON serialization/deserialization.
- **once_cell**: Lazy initialization support for static data.
- **uuid**: Generation of unique document identifiers.
- **base64**: Encoding/decoding CRDT update payloads.
- **tracing**: Instrumentation and structured logging framework.
- **tracing-subscriber**: Subscriber for `tracing` with formatting and filtering support.

## 🚧 Roadmap

- [ ] Infrastructure

  - [ ] Caching
    - [ ] Support Redis
    - [ ] Support multi-level caching
  - [ ] Storage
    - [ ] Support MySQL
    - [ ] Support MongoDB

- [ ] Connectivity

  - [ ] Heartbeat
  - [ ] Rate limiting

- [ ] Logging & Monitoring

  - [ ] Dynamic log level adjustment
  - [ ] Integrate metrics collection (e.g. Prometheus)

- [ ] Deployment

  - [ ] Docker image
  - [ ] Kubernetes deployment

- [ ] Testing

  - [ ] Unit test coverage
  - [ ] Integration tests

- [ ] Performance Optimization
  - [ ] Algorithmic improvements
    - [ ] Delta-only updates
    - [ ] Compress small updates
  - [ ] Concurrency optimizations
    - [ ] Lock-free or fine-grained locks
    - [ ] Reduce context switches
  - [ ] Resource pooling

## 🏗️ Architecture

The project follows Clean Architecture principles with clear separation of concerns:

- **Domain Layer**: Core business logic and entities

  - `entities/`: Core domain models
  - `repositories/`: Repository interfaces
  - `services/`: Domain services

- **Application Layer**: Use cases and application services

  - `use_cases/`: Application-specific business rules

- **Infrastructure Layer**: External implementations

  - `adapters/`: Concrete implementations of repositories

- **Adapter Layer**: Web/API adapters
  - `http/`: HTTP server and routing
  - `websocket/`: WebSocket handlers

## 🚀 Getting Started

### Prerequisites

- Rust 1.60+ (install via [rustup](https://rustup.rs/))
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/yjs-collaboration-server.git
   cd yjs-collaboration-server
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Running the Server

```bash
cargo run --release
```

The server will start on `http://localhost:8080`

## 📚 API Documentation

### Endpoints

- `GET /` - Health check endpoint

  - Returns: `200 OK` with server status

- `GET /ws` - WebSocket endpoint for real-time collaboration
  - Protocol: Yjs synchronization protocol over WebSocket

### WebSocket Protocol

The server implements the Yjs synchronization protocol. Clients should connect to the WebSocket endpoint and follow the protocol for document synchronization.

#### Message Format

```json
{
  "type": "sync|update|sv",
  "data": "...",
  "update": "base64_encoded_update"
}
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

For test coverage:

```bash
cargo tarpaulin --ignore-tests
```

## 🛠️ Development

### Code Style

This project uses `rustfmt` for code formatting. Please run:

```bash
cargo fmt
```

### Linting

Run the linter:

```bash
cargo clippy -- -D warnings
```

### Documentation

Generate documentation:

```bash
cargo doc --open
```

## 🤝 Contributing

Contributions are welcome! Please follow these steps to contribute:

1. Fork the repository to your GitHub account.
2. Clone your fork and add the upstream repository:
   ```bash
   git clone https://github.com/<your-username>/yjs-collaboration-server.git
   cd volo-http-example
   git remote add upstream https://github.com/Wenrh2004/yjs-collaboration-server.git
   ```
3. Create a feature branch:
   ```bash
   git checkout -b feature/amazing-feature
   ```
4. Commit your changes:
   ```bash
   git add .
   git commit -m "feat: add some amazing feature"
   ```
5. Sync your branch with upstream:
   ```bash
   git fetch upstream
   git rebase upstream/master
   ```
6. Push to your fork and open a Pull Request on the upstream repository, explaining your changes.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Yjs](https://yjs.dev/) - CRDT framework for collaborative applications
- [Yrs](https://github.com/y-crdt/y-crdt) - Rust port of Yjs
- [Volo](https://www.cloudwego.io/volo/) - High-performance HTTP/RPC framework
