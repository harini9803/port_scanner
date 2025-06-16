use clap::Parser;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
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
}

fn main() {
    let args = Args::parse();

    let (start_port, end_port) = match parse_port_range(&args.ports) {
        Ok(range) => range,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    println!(
        "Scanning {} ports {}–{} with timeout={}ms…\n",
        args.ip, start_port, end_port, args.timeout
    );

    let start_time = Instant::now();

    for port in start_port..=end_port {
        let addr: SocketAddr = format!("{}:{}", args.ip, port)
            .parse()
            .expect("Failed to parse socket address");

        if args.verbose {
            println!("Trying port {}...", port);
        }

        match TcpStream::connect_timeout(&addr, Duration::from_millis(args.timeout)) {
            Ok(mut stream) => {
                println!("Port {:5} is OPEN", port);

                if args.banner {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(args.timeout)));
                    let _ = stream.write_all(b"\n");

                    let mut buffer = [0; 512];
                    match stream.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let banner = String::from_utf8_lossy(&buffer[..n]);
                            println!("Banner: {}", banner.trim());
                        }
                        Ok(_) => {
                            println!("Banner: <no response>");
                        }
                        Err(e) => {
                            println!("Banner read failed: {}", e);
                        }
                    }
                }
            }
            Err(_) => {
                if args.verbose {
                    println!("Port {:5} is CLOSED", port);
                }
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("\nScan complete in {:.2} seconds.", elapsed);
}
