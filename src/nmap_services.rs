use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_nmap_services(path: &str) -> HashMap<(u16, String), String> {
    let file = File::open(path).expect("Could not open nmap-services file");
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let service = parts[0];
        let port_proto: Vec<&str> = parts[1].split('/').collect();
        if port_proto.len() != 2 {
            continue;
        }
        let proto = port_proto[1].to_lowercase();
        if proto != "tcp" { continue; } 
        if let Ok(port) = port_proto[0].parse::<u16>() {
            map.insert((port, proto), service.to_string());
        }
    }
    map
}

pub fn get_service(port: u16, protocol: &str, services: &HashMap<(u16, String), String>) -> String {
    services.get(&(port, protocol.to_lowercase())).cloned().unwrap_or_else(|| "Unknown".to_string())
} 