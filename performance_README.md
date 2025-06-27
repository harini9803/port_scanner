# Performance & Benchmarking

This document shows how to set up and run performance comparisons between the Rust-based port scanner and the Python reference implementation (`python_scanner.py`). It walks you through configuring the environment, executing both scanners on the same target, and comparing their scan durations.

---

## Prerequisites

* **Rust** (with `cargo` in your `PATH`)
* **Python 3.x**
* **Git** (to clone the repo)

Ensure you have both the Rust project and the Python script in the same directory:

```bash
cd path/to/port_scanner
# you should see:
# ├── src/
# ├── tests/
# ├── python_scanner.py
# ├── Cargo.toml
# └── performance_README.md
```

---

## 1. Configure Python Scanner

The `python_scanner.py` script accepts the same arguments as the Rust binary:

* `--ip <TARGET_IP>` (e.g., `127.0.0.1`)
* `--ports <RANGE>` (e.g., `1-1024`)
* `--timeout <SECONDS>` (default `0.2`)

Example invocation:

```bash
python3 python_scanner.py --ip 127.0.0.1 --ports 1-500 --timeout 0.1
```

This prints each open port and the total scan time.

---

## 2. Build Rust Scanner

Compile the Rust scanner in release mode for optimized performance:

```bash
cargo build --release
```

Your optimized binary will be at `./target/release/port_scanner`.

---

## 3. Run Benchmarks

Execute both scanners back-to-back on the same target and port range. Capture timestamps to compare durations.

### 3.1 Bash Example

```bash
# Set common variables
target_ip=127.0.0.1
port_range="1-500"
timeout=0.1

# Rust scan
echo "Running Rust scanner..."
start_rust=$(date +%s.%N)
./target/release/port_scanner --ip $target_ip --ports $port_range --timeout $(echo "$timeout*1000" | bc) --no-verbose
end_rust=$(date +%s.%N)
rust_duration=$(echo "$end_rust - $start_rust" | bc)
echo "Rust scan complete in ${rust_duration}s"

# Python scan
echo "Running Python scanner..."
start_py=$(date +%s.%N)
python3 python_scanner.py --ip $target_ip --ports $port_range --timeout $timeout > /dev/null
end_py=$(date +%s.%N)
py_duration=$(echo "$end_py - $start_py" | bc)
echo "Python scan complete in ${py_duration}s"

# Summary
echo "\nSummary:"
echo "  Rust:   ${rust_duration}s"
echo "  Python: ${py_duration}s"
```

> **Note**: In the Rust call, we convert timeout from seconds to milliseconds if your CLI flag expects ms.

---

## 4. Interpreting Results

* Average of time taken by Rust port scanner for 5 tests: 212.468 seconds.
* Average of time taken by Python port scanner for 5 tests: 213.114 seconds.
* After asynchronous scanning was included using the `tokio` crate, the Rust port scanner achieved approximately **100x speedup**:
  - Without extra features: **2.15 seconds**
  - With banner grabbing: **2.16 seconds**
  - With verbose flagging: **2.52 seconds**


---


