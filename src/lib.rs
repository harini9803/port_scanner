
pub fn parse_port_range(range: &str) -> Result<(u16, u16), String> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid port range format. Use START-END (e.g., 1-1024)".to_string());
    }

    let start = parts[0].parse::<u16>().map_err(|_| "Invalid start port".to_string())?;
    let end = parts[1].parse::<u16>().map_err(|_| "Invalid end port".to_string())?;

    if start > end {
        return Err("Start port must be less than or equal to end port".to_string());
    }

    Ok((start, end))
}
