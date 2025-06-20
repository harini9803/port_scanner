use clap::Parser;
use std::time::Instant;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{timeout, Duration},
};
use port_scanner::parse_port_range;

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
    verbose: bool,
    #[arg(long, default_value_t = false)]
    banner: bool,
    #[arg(long, default_value = "100")]
    concurrency: usize,
}

async fn scan_port(ip: &str, port: u16, timeout_ms: u64, verbose: bool, banner: bool) -> Option<(u16, Option<String>)> {
    let addr = format!("{}:{}", ip, port);
    
    if verbose {
        println!("Trying port {}...", port);
    }

    match timeout(
        Duration::from_millis(timeout_ms),
        TcpStream::connect(&addr)
    ).await {
        Ok(Ok(mut stream)) => {
            let mut banner_text = None;
            
            if banner {
                // Try to get banner information
                if let Ok(_) = stream.write_all(b"\r\n").await {
                    let mut buffer = [0; 512];
                    match timeout(
                        Duration::from_millis(timeout_ms),
                        stream.read(&mut buffer)
                    ).await {
                        Ok(Ok(n)) if n > 0 => {
                            let banner_str = String::from_utf8_lossy(&buffer[..n]);
                            banner_text = Some(banner_str.trim().to_string());
                        }
                        Ok(Ok(_)) => {
                            banner_text = Some("<no response>".to_string());
                        }
                        Ok(Err(_)) | Err(_) => {
                            banner_text = Some("<banner read failed>".to_string());
                        }
                    }
                }
            }
            
            Some((port, banner_text))
        }
        Ok(Err(_)) | Err(_) => {
            if verbose {
                println!("Port {:5} is CLOSED", port);
            }
            None
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    let (start_port, end_port) = match parse_port_range(&args.ports) {
        Ok(range) => range,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    println!(
        "Scanning {} ports {}–{} with timeout={}ms, concurrency={}…\n",
        args.ip, start_port, end_port, args.timeout, args.concurrency
    );

    let start_time = Instant::now();
    let mut open_ports = Vec::new();

    // Create semaphore to limit concurrency
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(args.concurrency));
    let mut tasks = Vec::new();

    // Create tasks for all ports
    for port in start_port..=end_port {
        let ip = args.ip.clone();
        let sem = semaphore.clone();
        let timeout_ms = args.timeout;
        let verbose = args.verbose;
        let banner = args.banner;

        let task = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            scan_port(&ip, port, timeout_ms, verbose, banner).await
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete and collect results
    for task in tasks {
        if let Ok(Some((port, banner_text))) = task.await {
            if let Some(banner) = banner_text {
                open_ports.push((port, Some(banner)));
            } else {
                open_ports.push((port, None));
            }
        }
    }

    // Sort and display results
    open_ports.sort_by_key(|&(port, _)| port);
    
    for (port, banner_text) in open_ports {
        println!("Port {:5} is OPEN", port);
        if let Some(banner) = banner_text {
            println!("Banner: {}", banner);
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("\nScan complete in {:.2} seconds.", elapsed);
}