use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use crate::nmap_services::get_service;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaptResult {
    pub port: u16,
    pub service: String,
    pub vulnerabilities: Vec<Vulnerability>,
    pub security_headers: Option<HashMap<String, String>>,
    pub ssl_info: Option<SslInfo>,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    pub name: String,
    pub description: String,
    pub severity: RiskLevel,
    pub cve: Option<String>,
    pub remediation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SslInfo {
    pub certificate_valid: bool,
    pub certificate_expiry: Option<String>,
    pub weak_ciphers: Vec<String>,
    pub tls_version: Option<String>,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "{}", "LOW".green()),
            RiskLevel::Medium => write!(f, "{}", "MEDIUM".yellow()),
            RiskLevel::High => write!(f, "{}", "HIGH".red()),
            RiskLevel::Critical => write!(f, "{}", "CRITICAL".bright_red()),
        }
    }
}

pub struct VaptScanner {
    client: Client,
    services: Arc<std::collections::HashMap<(u16, String), String>>,
}

impl VaptScanner {
    pub fn new(services: Arc<std::collections::HashMap<(u16, String), String>>) -> Self { 
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client, services }
    }

    pub async fn scan_port(&self, ip: &str, port: u16, banner: Option<String>) -> VaptResult {
        let service = self.identify_service(port, &banner);
        let mut vulnerabilities = Vec::new();
        let mut recommendations = Vec::new();

        match service.as_str() {
            "HTTP" | "HTTPS" => {
                let (web_vulns, headers, ssl_info) = self.scan_web_service(ip, port).await;
                vulnerabilities.extend(web_vulns);
                if let Some(ref headers) = headers {
                    self.check_web_security_headers(headers, &mut vulnerabilities, &mut recommendations);
                }
                let risk_level = self.calculate_risk_level(&vulnerabilities);
                return VaptResult {
                    port,
                    service,
                    vulnerabilities,
                    security_headers: headers,
                    ssl_info,
                    risk_level,
                    recommendations,
                };
            }
            "SSH" => {
                self.check_ssh_vulnerabilities(ip, port, &mut vulnerabilities, &mut recommendations).await;
            }
            "FTP" => {
                self.check_ftp_vulnerabilities(ip, port, &mut vulnerabilities, &mut recommendations).await;
            }
            "SMTP" => {
                self.check_smtp_vulnerabilities(ip, port, &mut vulnerabilities, &mut recommendations).await;
            }
            "TELNET" => {
                vulnerabilities.push(Vulnerability {
                    name: "Telnet Service Detected".to_string(),
                    description: "Telnet transmits data in plain text, making it vulnerable to sniffing attacks".to_string(),
                    severity: RiskLevel::Critical,
                    cve: None,
                    remediation: "Disable Telnet and use SSH instead".to_string(),
                });
                recommendations.push("Replace Telnet with SSH for secure remote access".to_string());
            }
            "RDP" => {
                self.check_rdp_vulnerabilities(ip, port, &mut vulnerabilities, &mut recommendations).await;
            }
            _ => {
                self.check_generic_vulnerabilities(ip, port, &service, &mut vulnerabilities, &mut recommendations).await;
            }
        }
        let risk_level = self.calculate_risk_level(&vulnerabilities);
        VaptResult {
            port,
            service,
            vulnerabilities,
            security_headers: None,
            ssl_info: None,
            risk_level,
            recommendations,
        }
    }

    fn identify_service(&self, port: u16, banner: &Option<String>) -> String {
        if let Some(banner_text) = banner {
            if banner_text.to_lowercase().contains("http") {
                return "HTTP".to_string();
            } else if banner_text.to_lowercase().contains("ssh") {
                return "SSH".to_string();
            } else if banner_text.to_lowercase().contains("ftp") {
                return "FTP".to_string();
            } else if banner_text.to_lowercase().contains("smtp") {
                return "SMTP".to_string();
            } else if banner_text.to_lowercase().contains("telnet") {
                return "TELNET".to_string();
            }
        }
        
        let service = get_service(port, "tcp", &self.services);
        if service == "Unknown" {
            if let Some(banner_text) = banner {
                if banner_text.to_lowercase().contains("http") {
                    "HTTP".to_string()
                } else if banner_text.to_lowercase().contains("ssh") {
                    "SSH".to_string()
                } else if banner_text.to_lowercase().contains("ftp") {
                    "FTP".to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            }
        } else {
            service
        }
    }

    async fn scan_web_service(&self, ip: &str, port: u16) -> (Vec<Vulnerability>, Option<HashMap<String, String>>, Option<SslInfo>) {
        let mut vulnerabilities = Vec::new();
        let mut headers = None;
        let mut ssl_info = None;

        let protocol = if port == 443 { "https" } else { "http" };
        let url = format!("{}://{}:{}", protocol, ip, port);

        match self.client.get(&url).send().await {
            Ok(response) => {
                headers = Some(response.headers().iter().map(|(k, v)| {
                    (k.as_str().to_ascii_lowercase(), v.to_str().unwrap_or("").to_string())
                }).collect());

                if response.status().as_u16() == 200 {
                    vulnerabilities.push(Vulnerability {
                        name: "Default Page Accessible".to_string(),
                        description: "Default web page is accessible, may reveal system information".to_string(),
                        severity: RiskLevel::Medium,
                        cve: None,
                        remediation: "Remove or secure default pages".to_string(),
                    });
                }
            }
            Err(_) => {
                vulnerabilities.push(Vulnerability {
                    name: "Web Service Unreachable".to_string(),
                    description: "Web service is not responding properly".to_string(),
                    severity: RiskLevel::Low,
                    cve: None,
                    remediation: "Check service configuration".to_string(),
                });
            }
        }

        if port == 443 {
            ssl_info = self.check_ssl_security(ip, port).await;
        }

        (vulnerabilities, headers, ssl_info)
    }

    async fn check_ssl_security(&self, ip: &str, port: u16) -> Option<SslInfo> {
        let mut issues = Vec::new();

        match timeout(Duration::from_secs(5), TcpStream::connect(format!("{}:{}", ip, port))).await {
            Ok(Ok(_)) => {
                issues.push("SSL certificate validation not implemented in this version".to_string());
            }
            _ => {
                issues.push("SSL connection failed".to_string());
            }
        }

        Some(SslInfo {
            certificate_valid: false, 
            certificate_expiry: None,
            weak_ciphers: Vec::new(),
            tls_version: None,
            issues,
        })
    }

    fn check_web_security_headers(&self, headers: &HashMap<String, String>, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        let security_headers = [
            ("x-frame-options", "Missing X-Frame-Options header", "Add X-Frame-Options: DENY"),
            ("x-content-type-options", "Missing X-Content-Type-Options header", "Add X-Content-Type-Options: nosniff"),
            ("x-xss-protection", "Missing X-XSS-Protection header", "Add X-XSS-Protection: 1; mode=block"),
            ("strict-transport-security", "Missing HSTS header", "Add Strict-Transport-Security header"),
            ("content-security-policy", "Missing CSP header", "Add Content-Security-Policy header"),
        ];

        for (header, description, recommendation) in security_headers.iter() {
            if !headers.contains_key(*header) {
                vulnerabilities.push(Vulnerability {
                    name: format!("Missing Security Header: {}", header),
                    description: description.to_string(),
                    severity: RiskLevel::Medium,
                    cve: None,
                    remediation: recommendation.to_string(),
                });
                recommendations.push(recommendation.to_string());
            }
        }
    }

    async fn check_ssh_vulnerabilities(&self, _ip: &str, _port: u16, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        vulnerabilities.push(Vulnerability {
            name: "SSH Service Detected".to_string(),
            description: "SSH service is running - check for weak configurations".to_string(),
            severity: RiskLevel::Low,
            cve: None,
            remediation: "Ensure SSH is properly configured with strong authentication".to_string(),
        });
        recommendations.push("Disable root login and use key-based authentication".to_string());
        recommendations.push("Change default SSH port to reduce automated attacks".to_string());
    }

    async fn check_ftp_vulnerabilities(&self, _ip: &str, _port: u16, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        vulnerabilities.push(Vulnerability {
            name: "FTP Service Detected".to_string(),
            description: "FTP transmits data in plain text, making it vulnerable to sniffing".to_string(),
            severity: RiskLevel::High,
            cve: None,
            remediation: "Use SFTP or FTPS instead of plain FTP".to_string(),
        });
        recommendations.push("Replace FTP with SFTP for secure file transfer".to_string());
    }

    async fn check_smtp_vulnerabilities(&self, _ip: &str, _port: u16, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        vulnerabilities.push(Vulnerability {
            name: "SMTP Service Detected".to_string(),
            description: "SMTP service may be vulnerable to relay attacks".to_string(),
            severity: RiskLevel::Medium,
            cve: None,
            remediation: "Configure SMTP to prevent open relay".to_string(),
        });
        recommendations.push("Configure SMTP authentication and relay restrictions".to_string());
    }

    async fn check_rdp_vulnerabilities(&self, _ip: &str, _port: u16, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        vulnerabilities.push(Vulnerability {
            name: "RDP Service Detected".to_string(),
            description: "RDP service may be vulnerable to brute force attacks".to_string(),
            severity: RiskLevel::High,
            cve: None,
            remediation: "Enable Network Level Authentication and strong passwords".to_string(),
        });
        recommendations.push("Enable NLA and use strong authentication for RDP".to_string());
        recommendations.push("Consider using a VPN for RDP access".to_string());
    }

    async fn check_generic_vulnerabilities(&self, _ip: &str, _port: u16, service: &str, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        vulnerabilities.push(Vulnerability {
            name: format!("{} Service Detected", service),
            description: format!("{} service running on port {}", service, _port),
            severity: RiskLevel::Medium,
            cve: None,
            remediation: format!("Identify and assess the {} service running on this port", service),
        });
        recommendations.push("Document and assess all running services".to_string());
    }

    fn calculate_risk_level(&self, vulnerabilities: &[Vulnerability]) -> RiskLevel {
        let mut max_severity = RiskLevel::Low;
        
        for vuln in vulnerabilities {
            match vuln.severity {
                RiskLevel::Critical => return RiskLevel::Critical,
                RiskLevel::High => max_severity = RiskLevel::High,
                RiskLevel::Medium => {
                    if max_severity == RiskLevel::Low {
                        max_severity = RiskLevel::Medium;
                    }
                }
                RiskLevel::Low => {}
            }
        }
        
        max_severity
    }
}

pub fn print_vapt_report(results: &[VaptResult]) {
    println!("\n{}", "=".repeat(80));
    println!("{}", "VULNERABILITY ASSESSMENT AND PENETRATION TESTING REPORT".bold());
    println!("{}", "=".repeat(80));
    
    let total_ports = results.len();
    let critical_vulns = results.iter().filter(|r| r.risk_level == RiskLevel::Critical).count();
    let high_vulns = results.iter().filter(|r| r.risk_level == RiskLevel::High).count();
    let medium_vulns = results.iter().filter(|r| r.risk_level == RiskLevel::Medium).count();
    let low_vulns = results.iter().filter(|r| r.risk_level == RiskLevel::Low).count();

    println!("\n {} SUMMARY:", "SCAN".bold());
    println!("   Total ports scanned: {}", total_ports);
    println!("   Critical vulnerabilities: {}", critical_vulns.to_string().bright_red());
    println!("   High vulnerabilities: {}", high_vulns.to_string().red());
    println!("   Medium vulnerabilities: {}", medium_vulns.to_string().yellow());
    println!("   Low vulnerabilities: {}", low_vulns.to_string().green());

    for result in results {
        if !result.vulnerabilities.is_empty() {
            println!("\n {} (Port {}) - Risk Level: {}", 
                result.service.bold(), 
                result.port, 
                result.risk_level
            );
            
            for vuln in &result.vulnerabilities {
                println!("   {}: {}", vuln.name.bold(), vuln.description);
                println!("     Severity: {}", vuln.severity);
                if let Some(ref cve) = vuln.cve {
                    println!("     CVE: {}", cve);
                }
                println!("     Remediation: {}", vuln.remediation);
            }

            if !result.recommendations.is_empty() {
                println!("     Recommendations:");
                for rec in &result.recommendations {
                    println!("     • {}", rec);
                }
            }
        }
    }

    let mut all_recommendations: std::collections::HashSet<String> = std::collections::HashSet::new();
    for result in results {
        all_recommendations.extend(result.recommendations.iter().cloned());
    }

    if !all_recommendations.is_empty() {
        println!("\n {} RECOMMENDATIONS:", "GENERAL".bold());
        for rec in all_recommendations {
            println!("   • {}", rec);
        }
    }

    println!("\n{}", "=".repeat(80));
}

pub fn print_vapt_report_json(results: &[VaptResult]) {
    match serde_json::to_string_pretty(results) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to serialize results to JSON: {}", e),
    }
}

pub fn print_vapt_report_markdown(results: &[VaptResult]) {
    println!("# Vulnerability Assessment and Penetration Testing Report\n");
    println!("| Port | Service | Risk Level | Vulnerabilities | Recommendations |");
    println!("|------|---------|------------|-----------------|-----------------|");
    for result in results {
        let vulns = if result.vulnerabilities.is_empty() {
            "None".to_string()
        } else {
            result.vulnerabilities.iter().map(|v| format!("**{}**: {} ({}). Remediation: {}", v.name, v.description, v.severity, v.remediation)).collect::<Vec<_>>().join("<br>")
        };
        let recs = if result.recommendations.is_empty() {
            "None".to_string()
        } else {
            result.recommendations.join("<br>")
        };
        println!("| {} | {} | {} | {} | {} |", result.port, result.service, result.risk_level, vulns, recs);
    }
    println!("");
    println!("## General Recommendations\n");
    let mut all_recommendations: std::collections::HashSet<String> = std::collections::HashSet::new();
    for result in results {
        all_recommendations.extend(result.recommendations.iter().cloned());
    }
    for rec in all_recommendations {
        println!("- {}", rec);
    }
}