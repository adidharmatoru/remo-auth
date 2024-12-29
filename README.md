# Remo-Auth

[![CI](https://github.com/adidharmatoru/remo-auth/actions/workflows/ci.yml/badge.svg)](https://github.com/adidharmatoru/remo-auth/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/adidharmatoru/remo-auth/branch/master/graph/badge.svg)](https://codecov.io/gh/adidharmatoru/remo-auth)

remo-auth is a Rust-based WebSocket service for authentication and signalling. It leverages the power of Axum to provide a robust and efficient solution for managing user authentication and real-time communication.

## Getting Started

To start using remo-auth, follow these simple steps:

### Prerequisites

1. Install Rust and Cargo on your system.
2. Clone this repository and navigate to it in your terminal or command prompt.

## Running the Service

To start the Remo-auth service, run the following command:

```bash
cargo run
```

This will start the service, making it ready for use.

## Running Tests

To run the test suite:

```bash
cargo test
```

For test coverage:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin
```
