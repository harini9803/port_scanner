use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

#[derive(Debug, Clone)]
pub struct BannerInfo {
    pub protocol: String,
    pub banner: String,
    pub server_info: Option<String>,
    pub version: Option<String>,
    pub additional_info: Vec<String>,
}

pub struct BannerGrabber {
    timeout_ms: u64,
}

impl BannerGrabber {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    pub async fn grab_banner(&self, ip: &str, port: u16) -> Option<BannerInfo> {
        if let Some(banner) = self.grab_http_banner(ip, port).await {
            return Some(banner);
        }
        
        if let Some(banner) = self.grab_ftp_banner(ip, port).await {
            return Some(banner);
        }
        
        if let Some(banner) = self.grab_smtp_banner(ip, port).await {
            return Some(banner);
        }
        
        self.grab_generic_banner(ip, port).await
    }

    async fn grab_http_banner(&self, ip: &str, port: u16) -> Option<BannerInfo> {
        let addr = format!("{}:{}", ip, port);
        
        match timeout(
            Duration::from_millis(self.timeout_ms),
            TcpStream::connect(&addr)
        ).await {
            Ok(Ok(mut stream)) => {
                let request = format!(
                    "HEAD / HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: PortScanner/1.0\r\nConnection: close\r\n\r\n",
                    ip, port
                );
                
                if let Ok(_) = stream.write_all(request.as_bytes()).await {
                    let mut response = Vec::new();
                    let mut buffer = [0; 1024];
                    
                    match timeout(
                        Duration::from_millis(self.timeout_ms),
                        stream.read(&mut buffer)
                    ).await {
                        Ok(Ok(n)) if n > 0 => {
                            response.extend_from_slice(&buffer[..n]);
                            
                            let response_str = String::from_utf8_lossy(&response);
                            let lines: Vec<&str> = response_str.lines().collect();
                            
                            if lines.is_empty() {
                                return None;
                            }
                            
                            let mut server_info = None;
                            let mut additional_info = Vec::new();
                            
                            for line in lines.iter().take(10) { 
                                if line.starts_with("Server:") {
                                    server_info = Some(line[7..].trim().to_string());
                                } else if line.starts_with("X-") || line.starts_with("Via:") || line.starts_with("X-Powered-By:") {
                                    additional_info.push(line.trim().to_string());
                                }
                            }
                            
                            let banner = if lines.len() > 1 {
                                lines[1..].join("\n")
                            } else {
                                lines[0].to_string()
                            };
                            
                            Some(BannerInfo {
                                protocol: "HTTP".to_string(),
                                banner: banner.trim().to_string(),
                                server_info,
                                version: None,
                                additional_info,
                            })
                        }
                        _ => None
                    }
                } else {
                    None
                }
            }
            _ => None
        }
    }

    async fn grab_ftp_banner(&self, ip: &str, port: u16) -> Option<BannerInfo> {
        let addr = format!("{}:{}", ip, port);
        
        match timeout(
            Duration::from_millis(self.timeout_ms),
            TcpStream::connect(&addr)
        ).await {
            Ok(Ok(mut stream)) => {
                let mut response = Vec::new();
                let mut buffer = [0; 512];
                
                match timeout(
                    Duration::from_millis(self.timeout_ms),
                    stream.read(&mut buffer)
                ).await {
                    Ok(Ok(n)) if n > 0 => {
                        response.extend_from_slice(&buffer[..n]);
                        
                        let banner_str = String::from_utf8_lossy(&response);
                        let banner = banner_str.trim().to_string();
                        
                        let server_info = if banner.contains("FTP server") || banner.contains("vsFTPd") || banner.contains("ProFTPD") {
                            Some(banner.clone())
                        } else {
                            None
                        };
                        
                        let mut additional_info = Vec::new();
                        if let Ok(_) = stream.write_all(b"HELP\r\n").await {
                            let mut help_buffer = [0; 256];
                            if let Ok(Ok(n)) = timeout(
                                Duration::from_millis(self.timeout_ms),
                                stream.read(&mut help_buffer)
                            ).await {
                                if n > 0 {
                                    let help_str = String::from_utf8_lossy(&help_buffer[..n]);
                                    additional_info.push(format!("HELP: {}", help_str.trim()));
                                }
                            }
                        }
                        
                        Some(BannerInfo {
                            protocol: "FTP".to_string(),
                            banner,
                            server_info,
                            version: None,
                            additional_info,
                        })
                    }
                    _ => None
                }
            }
            _ => None
        }
    }

    async fn grab_smtp_banner(&self, ip: &str, port: u16) -> Option<BannerInfo> {
        let addr = format!("{}:{}", ip, port);
        
        match timeout(
            Duration::from_millis(self.timeout_ms),
            TcpStream::connect(&addr)
        ).await {
            Ok(Ok(mut stream)) => {
                let mut response = Vec::new();
                let mut buffer = [0; 512];
                
                match timeout(
                    Duration::from_millis(self.timeout_ms),
                    stream.read(&mut buffer)
                ).await {
                    Ok(Ok(n)) if n > 0 => {
                        response.extend_from_slice(&buffer[..n]);
                        
                        let banner_str = String::from_utf8_lossy(&response);
                        let banner = banner_str.trim().to_string();
                        
                        let server_info = if banner.contains("SMTP") || banner.contains("ESMTP") {
                            Some(banner.clone())
                        } else {
                            None
                        };
                        
                        let mut additional_info = Vec::new();
                        if let Ok(_) = stream.write_all(b"EHLO example.com\r\n").await {
                            let mut ehlo_buffer = [0; 512];
                            if let Ok(Ok(n)) = timeout(
                                Duration::from_millis(self.timeout_ms),
                                stream.read(&mut ehlo_buffer)
                            ).await {
                                if n > 0 {
                                    let ehlo_str = String::from_utf8_lossy(&ehlo_buffer[..n]);
                                    let lines: Vec<&str> = ehlo_str.lines().collect();
                                    for line in lines.iter().take(5) { 
                                        if line.contains("250") && (line.contains("SIZE") || line.contains("AUTH") || line.contains("STARTTLS")) {
                                            additional_info.push(line.trim().to_string());
                                        }
                                    }
                                }
                            }
                        }
                        
                        Some(BannerInfo {
                            protocol: "SMTP".to_string(),
                            banner,
                            server_info,
                            version: None,
                            additional_info,
                        })
                    }
                    _ => None
                }
            }
            _ => None
        }
    }

    async fn grab_generic_banner(&self, ip: &str, port: u16) -> Option<BannerInfo> {
        let addr = format!("{}:{}", ip, port);
        
        match timeout(
            Duration::from_millis(self.timeout_ms),
            TcpStream::connect(&addr)
        ).await {
            Ok(Ok(mut stream)) => {
                if let Ok(_) = stream.write_all(b"\r\n").await {
                    let mut buffer = [0; 512];
                    match timeout(
                        Duration::from_millis(self.timeout_ms),
                        stream.read(&mut buffer)
                    ).await {
                        Ok(Ok(n)) if n > 0 => {
                            let banner_str = String::from_utf8_lossy(&buffer[..n]);
                            let banner = banner_str.trim().to_string();
                            
                            let protocol = if banner.to_lowercase().contains("http") {
                                "HTTP".to_string()
                            } else if banner.to_lowercase().contains("ssh") {
                                "SSH".to_string()
                            } else if banner.to_lowercase().contains("ftp") {
                                "FTP".to_string()
                            } else if banner.to_lowercase().contains("smtp") {
                                "SMTP".to_string()
                            } else if banner.to_lowercase().contains("telnet") {
                                "TELNET".to_string()
                            } else {
                                "Unknown".to_string()
                            };
                            
                            Some(BannerInfo {
                                protocol,
                                banner,
                                server_info: None,
                                version: None,
                                additional_info: Vec::new(),
                            })
                        }
                        _ => None
                    }
                } else {
                    None
                }
            }
            _ => None
        }
    }
}

pub fn print_banner_info(banner_info: &BannerInfo) {
    println!("  Protocol: {}", banner_info.protocol);
    println!("  Banner: {}", banner_info.banner);
    
    if let Some(ref server_info) = banner_info.server_info {
        println!("  Server: {}", server_info);
    }
    
    if let Some(ref version) = banner_info.version {
        println!("  Version: {}", version);
    }
    
    if !banner_info.additional_info.is_empty() {
        println!("  Additional Info:");
        for info in &banner_info.additional_info {
            println!("    {}", info);
        }
    }
} 