# Rust Port Scanner & VAPT Tool

A fast, modern, and extensible CLI-based port scanner and vulnerability assessment tool written in Rust. It supports advanced features like banner grabbing, service detection, vulnerability checks, security header analysis, SSL/TLS checks, and multiple output formats (text, JSON, Markdown). It also includes nmap-based verification and a Python scanner for performance comparison.

---

## Features

- **Fast concurrent port scanning** with configurable concurrency and timeouts
- **Service detection** using nmap-services and banner analysis
- **Banner grabbing** for HTTP, FTP, SMTP, and generic services
- **Vulnerability Assessment (VAPT):**
  - Service-specific vulnerability checks for HTTP(S), SSH, FTP, SMTP, RDP, TELNET, and more
  - Web security header analysis (CSP, HSTS, X-Frame-Options, etc.)
  - SSL/TLS checks for HTTPS (basic validation, placeholder for future improvements)
  - Risk level calculation (Low, Medium, High, Critical)
  - Actionable recommendations for each finding
- **Multiple output formats:** human-readable text, JSON, and Markdown reports
- **Nmap verification:** compare results with nmap for accuracy
- **Unit-tested port range parsing**
- **Performance comparison** with a basic Python port scanner

---

## Project Structure

```
port_scanner/
├── src/
│   ├── main.rs            # Main application entry point (CLI, scanning logic)
│   ├── lib.rs             # Port range parsing and shared utilities
│   ├── vapt.rs            # Vulnerability assessment logic (VAPT)
│   ├── banner_grabber.rs  # Banner grabbing for various protocols
│   └── nmap_services.rs   # nmap-services file parsing and service lookup
├── tests/
│   └── unit_tests.rs      # Unit tests for port range parsing
├── python_scanner.py      # Python implementation for benchmarking
├── nmap-services          # nmap service definitions (required for service detection)
├── README.md              # This file
└── ... (other docs, reports, etc.)
```

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (`rustc`, `cargo`)
- [Tokio](https://crates.io/crates/tokio) (async runtime)
- [clap](https://crates.io/crates/clap) (CLI argument parsing)
- [colored](https://crates.io/crates/colored) (colored terminal output)
- [serde](https://crates.io/crates/serde), [serde_json](https://crates.io/crates/serde_json) (serialization)
- [reqwest](https://crates.io/crates/reqwest) (HTTP client)
- Python 3.x (for benchmarking)
- nmap (optional, for verification)
- Git (optional)

---

## Setup

1. **Clone the repository:**
   ```bash
   git clone <repo-url>
   cd port_scanner
   ```

2. **Ensure dependencies are in `Cargo.toml`:**
   ```toml
   [dependencies]
   tokio = { version = "1", features = ["full"] }
   clap = { version = "4.0", features = ["derive"] }
   colored = "2"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   reqwest = { version = "0.11", features = ["json", "blocking", "rustls-tls"] }
   ```

3. **Download or symlink the `nmap-services` file** (required for service detection).

4. **Build the project:**
   ```bash
   cargo build --release
   ```

---

## Usage

### Basic Scan

```bash
cargo run --release -- --ip 127.0.0.1 --ports 20-100
```

### Options

- `--ip <IP_ADDRESS>`: Target IP address (required)
- `--ports <START-END>`: Port range to scan (e.g., `1-1024`)
- `--timeout <MILLIS>`: Connection timeout in milliseconds (default: 200)
- `--concurrency <N>`: Number of concurrent scans (default: 100)
- `--banner`: Enable banner grabbing (default: off)
- `--output <text|json|markdown>`: Output format (default: text)
- `--verify`: Compare results with nmap (if installed)

### Example: Full VAPT Scan with Banner Grabbing and Markdown Output

```bash
cargo run --release -- --ip 192.168.1.1 --ports 1-1024 --banner --output markdown --verify
```

---

## Output Formats

- **Text:** Human-readable, color-coded report with risk levels and recommendations.
- **JSON:** Machine-readable, structured output for integration or further analysis.
- **Markdown:** Table-based report for easy sharing or documentation.

---

## Vulnerability Assessment (VAPT) Details

- **Service Detection:** Uses both nmap-services and banner analysis for accurate protocol identification.
- **Web Services:** Checks for missing security headers (CSP, HSTS, X-Frame-Options, etc.), default pages, and basic SSL/TLS issues.
- **SSH, FTP, SMTP, RDP, TELNET:** Checks for common misconfigurations and insecure defaults.
- **Risk Levels:** Each finding is classified as Low, Medium, High, or Critical.
- **Recommendations:** Actionable remediation steps for each vulnerability and a summary of general recommendations.

---

## Example Output

### Text

```
================================================================================
VULNERABILITY ASSESSMENT AND PENETRATION TESTING REPORT
================================================================================

 SCAN SUMMARY:
   Total ports scanned: 3
   Critical vulnerabilities: 1
   High vulnerabilities: 1
   Medium vulnerabilities: 2
   Low vulnerabilities: 0

 HTTP (Port 80) - Risk Level: MEDIUM
   Missing Security Header: x-frame-options: Missing X-Frame-Options header
     Severity: MEDIUM
     Remediation: Add X-Frame-Options: DENY
   ...

 GENERAL RECOMMENDATIONS:
   • Add X-Frame-Options: DENY
   • Replace Telnet with SSH for secure remote access

================================================================================
```

### JSON

```json
[
  {
    "port": 80,
    "service": "HTTP",
    "vulnerabilities": [
      {
        "name": "Missing Security Header: x-frame-options",
        "description": "Missing X-Frame-Options header",
        "severity": "Medium",
        "cve": null,
        "remediation": "Add X-Frame-Options: DENY"
      }
    ],
    "security_headers": {
      "server": "nginx/1.18.0"
    },
    "ssl_info": null,
    "risk_level": "Medium",
    "recommendations": ["Add X-Frame-Options: DENY"]
  }
]
```

### Markdown

| Port | Service | Risk Level | Vulnerabilities | Recommendations |
|------|---------|------------|-----------------|-----------------|
| 80   | HTTP    | MEDIUM     | **Missing Security Header: x-frame-options**: Missing X-Frame-Options header (Medium). Remediation: Add X-Frame-Options: DENY | Add X-Frame-Options: DENY |

---

## Running Tests

```bash
cargo test
```

---

## Python Scanner (for Benchmarking)

A simple Python port scanner is included for performance comparison:

```bash
python python_scanner.py --ip 127.0.0.1 --ports 1-1024
```

---

## Notes

- The SSL/TLS checks are basic and can be extended for full certificate validation and weak cipher detection.
- The tool is extensible: add more vulnerability checks in `src/vapt.rs` as needed.
- For best results, run as root/administrator to avoid permission issues on some ports.

---


