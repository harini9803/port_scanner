mod nmap_services;
mod vapt;
mod banner_grabber;

use std::sync::Arc;
use std::time::Duration;
use tokio::{net::TcpStream, time::timeout};
use clap::Parser;
use crate::nmap_services::load_nmap_services;
use crate::vapt::{VaptScanner, print_vapt_report, print_vapt_report_json, print_vapt_report_markdown};
use crate::banner_grabber::{BannerGrabber, print_banner_info};
use std::process::Command;
use std::collections::HashSet;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    ip: String,
    #[arg(long)]
    ports: String, 
    #[arg(long, default_value = "200")]
    timeout: u64, 
    #[arg(long, default_value_t = false)]
    banner: bool,
    #[arg(long, default_value = "100")]
    concurrency: usize,
    #[arg(long, default_value = "text")]
    output: String, 
    #[arg(long, default_value_t = false)]
    verify: bool,
}

async fn grab_banner(ip: &str, port: u16, timeout_ms: u64) -> Option<String> {
    let addr = format!("{}:{}", ip, port);
    let stream = timeout(Duration::from_millis(timeout_ms), TcpStream::connect(&addr)).await.ok().and_then(|r| r.ok())?;
    let stream = stream;
    let _ = stream.writable().await;
    let _ = stream.try_write(b"\r\n");
    let mut buf = [0u8; 1024];
    let _n = timeout(Duration::from_millis(timeout_ms), stream.readable()).await.ok()?;
    let n = stream.try_read(&mut buf).ok()?;
    if n > 0 {
        Some(String::from_utf8_lossy(&buf[..n]).to_string())
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    let start_time = std::time::Instant::now();
    let args = Args::parse();
    let (start_port, end_port) = {
        let parts: Vec<&str> = args.ports.split('-').collect();
        if parts.len() != 2 {
            eprintln!("Invalid port range. Use START-END, e.g. 1-1024");
            return;
        }
        (parts[0].parse::<u16>().unwrap(), parts[1].parse::<u16>().unwrap())
    };
    
    let nmap_services = Arc::new(load_nmap_services("nmap-services"));
    let vapt_scanner = Arc::new(VaptScanner::new(Arc::clone(&nmap_services)));
    
    println!("Scanning {} ports {}-{} with timeout={}ms, concurrency={}...", 
             args.ip, start_port, end_port, args.timeout, args.concurrency);
    
    use futures::stream::{FuturesUnordered, StreamExt};
    let mut tasks = FuturesUnordered::new();
    let mut open_ports = Vec::new();
    
    for port in start_port..=end_port {
        let ip = args.ip.clone();
        let timeout_ms = args.timeout;
        
        tasks.push(tokio::spawn(async move {
            let addr = format!("{}:{}", ip, port);
            if let Ok(Ok(_)) = timeout(Duration::from_millis(timeout_ms), TcpStream::connect(&addr)).await {
                Some(port)
            } else {
                None
            }
        }));
        
        if tasks.len() >= args.concurrency {
            if let Some(res) = tasks.next().await {
                if let Ok(Some(port)) = res {
                    println!("Port {} is OPEN", port);
                    open_ports.push(port);
                }
            }
        }
    }
    
    while let Some(res) = tasks.next().await {
        if let Ok(Some(port)) = res {
            println!("Port {} is OPEN", port);
            open_ports.push(port);
        }
    }
    
    let banner_grabber = BannerGrabber::new(args.timeout);
    let mut vapt_results = Vec::new();
    for &port in &open_ports {
        let banner_info = if args.banner {
            banner_grabber.grab_banner(&args.ip, port).await
        } else {
            None
        };
        if let Some(ref info) = banner_info {
            print_banner_info(info);
        }
        let vapt_result = vapt_scanner.scan_port(&args.ip, port, banner_info.as_ref().map(|b| b.banner.clone())).await;
        vapt_results.push(vapt_result);
    }
    
    if args.verify {
        println!("\n[Verification] Running nmap for comparison...");
        let nmap_output = Command::new("nmap")
            .arg("-p")
            .arg(format!("{}-{}", start_port, end_port))
            .arg(&args.ip)
            .output();
        match nmap_output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut nmap_open_ports = Vec::new();
                for line in stdout.lines() {
                    if let Some(port) = line.split('/').next() {
                        if let Ok(port_num) = port.trim().parse::<u16>() {
                            if line.contains("open") {
                                nmap_open_ports.push(port_num);
                            }
                        }
                    }
                }

                let my_ports: HashSet<_> = open_ports.iter().copied().collect();
                let nmap_ports: HashSet<_> = nmap_open_ports.iter().copied().collect();
                let rust_port_scanner: Vec<_> = my_ports.difference(&nmap_ports).collect();
                let only_in_nmap: Vec<_> = nmap_ports.difference(&my_ports).collect();
                println!("\n[Verification] Comparison with nmap:");
                if rust_port_scanner.is_empty() && only_in_nmap.is_empty() {
                    println!(" Results match nmap output.");
                } else {
                    if !rust_port_scanner.is_empty() {
                        println!("  Ports found only by this scanner: {:?}", rust_port_scanner);
                    }
                    if !only_in_nmap.is_empty() {
                        println!("  Ports found only by nmap: {:?}", only_in_nmap);
                    }
                }
            }
            Err(e) => {
                println!("Failed to run nmap: {e}");
            }
        }
    }
    
    match args.output.to_lowercase().as_str() {
        "text" => {
            print_vapt_report(&vapt_results);
        }
        "json" => {
            print_vapt_report_json(&vapt_results);
        }
        "markdown" => {
            print_vapt_report_markdown(&vapt_results);
        }
        _ => {
            eprintln!("Invalid output mode: {}. Using 'text' instead.", args.output);
            print_vapt_report(&vapt_results);
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("\nScan complete in {:.2} seconds.", elapsed.as_secs_f64());
}