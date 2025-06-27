# Banner Grabbing Functionality

This document explains how to use the enhanced banner grabbing functionality in the port scanner for HTTP, FTP, and SMTP protocols.

## Overview

The banner grabbing feature allows you to extract detailed information about services running on open ports, including:
- Protocol identification
- Server information
- Version details
- Additional service capabilities

## Supported Protocols

### 1. HTTP/HTTPS Banner Grabbing
- Sends HTTP HEAD requests to extract server information
- Parses HTTP headers for server details
- Extracts additional headers like `X-Powered-By`, `Via`, etc.
- Identifies web server type and version

### 2. FTP Banner Grabbing
- Reads initial FTP server banner
- Sends HELP command to get available commands
- Identifies FTP server type (vsFTPd, ProFTPD, etc.)
- Extracts server capabilities

### 3. SMTP Banner Grabbing
- Reads initial SMTP server banner
- Sends EHLO command to get server capabilities
- Identifies SMTP server type and features
- Extracts authentication and security features

## Usage

### Basic Banner Grabbing
```bash
# Scan with banner grabbing enabled
cargo run -- --ip 127.0.0.1 --ports 80-100 --banner

# Scan specific ports with banner grabbing
cargo run -- --ip 127.0.0.1 --ports 21,22,25,80,443 --banner --verbose
```

### Example Output
```
Port    80 is OPEN
  Protocol: HTTP
  Banner: HTTP/1.1 200 OK
  Server: Apache/2.4.41 (Ubuntu)
  Additional Info:
    X-Powered-By: PHP/7.4.3

Port    21 is OPEN
  Protocol: FTP
  Banner: 220 Test FTP Server (vsFTPd 3.0.3) ready.
  Server: Test FTP Server (vsFTPd 3.0.3) ready.
  Additional Info:
    HELP: 214-The following commands are recognized:
```

## Testing Banner Grabbing

### 1. Start Test Servers
Use the provided test script to start servers for testing:

```bash
# Start all test servers
python test_banner_grabber.py all

# Or start individual servers
python test_banner_grabber.py http   
python test_banner_grabber.py ftp    
python test_banner_grabber.py smtp   
```

### 2. Test Banner Grabbing
```bash
# Test HTTP banner grabbing
cargo run -- --ip 127.0.0.1 --ports 8080 --banner

# Test FTP banner grabbing
cargo run -- --ip 127.0.0.1 --ports 2121 --banner

# Test SMTP banner grabbing
cargo run -- --ip 127.0.0.1 --ports 2525 --banner

# Test all protocols
cargo run -- --ip 127.0.0.1 --ports 8080,2121,2525 --banner
```

## Implementation Details

### BannerGrabber Structure
```rust
pub struct BannerGrabber {
    timeout_ms: u64,
}

pub struct BannerInfo {
    pub protocol: String,
    pub banner: String,
    pub server_info: Option<String>,
    pub version: Option<String>,
    pub additional_info: Vec<String>,
}
```

### Protocol-Specific Methods
- `grab_http_banner()` - HTTP/HTTPS banner grabbing
- `grab_ftp_banner()` - FTP banner grabbing
- `grab_smtp_banner()` - SMTP banner grabbing
- `grab_generic_banner()` - Fallback for other protocols

### Integration with VAPT Scanner
The banner information is automatically passed to the VAPT scanner for enhanced vulnerability assessment:
- Service identification based on banner content
- Protocol-specific vulnerability checks
- Enhanced security recommendations

## Security Considerations

1. **Timeout Configuration**: Banner grabbing uses configurable timeouts to prevent hanging
2. **Error Handling**: Graceful handling of connection failures and malformed responses
3. **Protocol Detection**: Automatic fallback to generic banner grabbing if protocol-specific methods fail
4. **Resource Management**: Proper cleanup of network connections

## Troubleshooting

### Common Issues

1. **No Banner Retrieved**
   - Check if the service is actually running
   - Verify firewall settings
   - Increase timeout value if needed

2. **Incomplete Banner Information**
   - Some services may not respond to banner requests
   - Try different ports or services
   - Check service configuration

3. **Connection Timeouts**
   - Increase the `--timeout` parameter
   - Check network connectivity
   - Verify target service is accessible

### Debug Mode
Use the `--verbose` flag to see detailed connection attempts:
```bash
cargo run -- --ip 127.0.0.1 --ports 80 --banner --verbose
```

## Advanced Usage

### Custom Port Ranges
```bash
# Scan common web ports
cargo run -- --ip 192.168.1.1 --ports 80,443,8080,8443 --banner

# Scan mail server ports
cargo run -- --ip 192.168.1.1 --ports 25,465,587 --banner

# Scan file transfer ports
cargo run -- --ip 192.168.1.1 --ports 21,22,990 --banner
```

### High-Concurrency Scanning
```bash
# Use higher concurrency for faster scanning
cargo run -- --ip 192.168.1.1 --ports 1-1000 --banner --concurrency 200
```

## Integration Examples

### With VAPT Scanner
The banner grabbing automatically integrates with the VAPT scanner:
```bash
# Full scan with banner grabbing and vulnerability assessment
cargo run -- --ip 192.168.1.1 --ports 80,443,21,22,25 --banner
```

This will:
1. Scan the specified ports
2. Grab banners for open ports
3. Perform vulnerability assessment based on banner information
4. Generate a comprehensive security report

## Future Enhancements

Planned improvements for banner grabbing:
- SSL/TLS certificate information extraction
- More protocol support (SSH, Telnet, etc.)
- Banner fingerprinting for service identification
- Custom banner templates for different services
- Banner comparison and analysis features 