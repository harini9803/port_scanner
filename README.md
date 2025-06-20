# Rust Port Scanner – Setup

This project is a fast and simple CLI-based port scanner written in Rust. It includes optional banner grabbing, configurable timeouts, and full unit test coverage for the port range parser.

## Features

* CLI interface using [`clap`](https://crates.io/crates/clap)
* Synchronous port scanning using `TcpStream`
* Optional service banner grabbing
* Unit-tested port range parsing
* Performance comparison with a basic Python port scanner

## Project Structure

```
port_scanner/
├── src/
│   ├── main.rs          # Main application entry point
│   └── lib.rs           # Port range parsing library
├── tests/
│   └── unit_tests.rs    # Unit tests for port range parsing
├── python_scanner.py    # Python implementation for comparison
├── Cargo.toml           # Rust project configuration
└── README.md            # This readme file
```

## Prerequisites

Ensure the following are installed:

* [Rust](https://www.rust-lang.org/tools/install) (`rustc`, `cargo`)
* [VS Code](https://code.visualstudio.com/) with the Rust Analyzer extension
* Python 3.x (for benchmarking comparison)
* Git (optional)

Verify Rust setup:

```bash
rustc --version
cargo --version
```

## Setup and Running

### 1. Initialize Rust Project

```bash
cargo new port_scanner
cd port_scanner
```

### 2. Update `Cargo.toml`

Add the following sections to your `Cargo.toml`:

```toml
[package]
name = "port_scanner"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }

[[bin]]
name = "port_scanner"
path = "src/main.rs"

[lib]
name = "port_scanner"
path = "src/lib.rs"
```

### 3. Code Overview

#### `src/main.rs`

* Uses `clap` for parsing command-line arguments.
* Defines an `Args` struct for parameters such as:

  * `--ip <IP_ADDRESS>`: Target IP address
  * `--ports <RANGE>`: Port range to scan (e.g., `20-100`)
  * `--timeout <MILLIS>`: Connection timeout in milliseconds
  * `--verbose`: Enable verbose output
  * `--grab-banner`: Enable banner grabbing
* Parses the port range and attempts TCP connections with the specified timeout.
* Reports open ports and optionally prints service banners.
* Measures and prints the total scan duration.

#### `src/lib.rs`

* Implements the `parse_port_range` function:

  1. Validates the format (must contain a single `-`).
  2. Parses start and end ports as `u16`.
  3. Ensures `start <= end`.
  4. Returns `Result<(u16, u16), String>` with error messages for invalid input.

#### `tests/unit_tests.rs`

* Contains unit tests for `parse_port_range` covering:

  * Valid range (e.g., `10-20`)
  * Missing hyphen (e.g., `1020`)
  * Non-numeric values (e.g., `abc-def`)
  * Start port greater than end port (e.g., `20-10`)

## Building the Project

* **Debug mode**:

  ```bash
  cargo build
  ```
* **Release mode** (optimized):

  ```bash
  cargo build --release
  ```

## Running the Port Scanner

* **Using Cargo**:

  ```bash
  cargo run -- --ip 127.0.0.1 --ports 20-100
  ```
* **Direct Binary (release)**:

  ```bash
  ./target/release/port_scanner --ip 127.0.0.1 --ports 20-100
  ```

## Running Tests

* Run all tests:

  ```bash
  cargo test
  ```
* Run only the port range parsing tests:

  ```bash
  cargo test -- --test-threads=1 unit_tests
  ```
