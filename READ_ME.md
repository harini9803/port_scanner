#  Rust Port Scanner – Setup 

This project is a fast and simple CLI-based port scanner written in Rust. It includes optional banner grabbing, configurable timeouts, and full unit test coverage for the port range parser.

Includes:
- CLI interface using `clap`
- Synchronous port scanning using `TcpStream`
- Optional service banner grabbing
- Unit-tested port range parsing
- Performance comparison with a basic Python port scanner

---

## Project Structure

port_scanner
├── src/
│   ├── main.rs              # Main application entry point
│   └── lib.rs               # Port range parsing library
├── tests/
│   └── unit_tests.rs        # Unit tests for port parser
├── Cargo.toml               # Rust project configuration
├── python_scanner.py        # Python comparison implementation                  


---

## Prerequisites

Ensure the following are installed:

- [Rust](https://www.rust-lang.org/tools/install) (`rustc`, `cargo`)
- [VS Code](https://code.visualstudio.com/)
- Rust Analyzer extension for IntelliSense and diagnostics
- Python 3.x (for benchmarking comparison)
- Git (optional)

Verify Rust setup:

rustc --version
cargo --version

---

## Setup and Running

### 1. Initialize Rust Project

# Create new Rust project in your folder
cargo new port_scanner 
cd port_scanner

### 2. Add below dependencies to cargo.toml file

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

### 3.I. Working in main.rs

The `main.rs` file implements a command-line port scanner using the `clap` crate for argument parsing. It defines a struct `Args` to handle parameters like target IP, port range, connection timeout, verbose mode, and banner grabbing. The program parses the input port range, attempts TCP connections with specified timeouts, and reports open ports. If banner grabbing is enabled, it reads service banners from open ports. Verbose mode displays status for all scanned ports. The scanner measures and outputs the total time taken to complete the scan.


### II. Working of lib.rs

The `parse_port_range` function validates and parses a port range string (e.g., "20-100") into a tuple of start and end ports. It checks if the input contains exactly two parts separated by a hyphen, converts these parts to valid u16 integers, ensures the start port is less than or equal to the end port, and returns appropriate error messages for any invalid cases.

### III. Working of unit_tests.rs

The provided tests for the `parse_port_range` function verify its functionality across different scenarios: checking if a valid port range like "10-20" is correctly parsed into a tuple (10, 20), ensuring an invalid format without a hyphen (e.g., "1020") returns an error, confirming non-numeric values (e.g., "abc-def") result in an error, and validating that a range where the start port is greater than the end port (e.g., "20-10") also returns an error.

### 4. Build the project

# Build in debug mode
cargo build

# Build optimized release version (recommended for performance)
cargo build --release

### 5. Running the port scanner

# Run in debug mode via cargo
cargo run -- --ip 127.0.0.1 --ports 20-100

# Or run the optimized release binary directly
./target/release/port_scanner --ip 127.0.0.1 --ports 20-100

### 5. For running the unit tests

# Run all tests
cargo test

# Run only the custom integration tests
cargo test --test unit_tests



















