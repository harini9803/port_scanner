use port_scanner::parse_port_range;

#[test]
fn test_valid_range() {
    let result = parse_port_range("10-20");
    assert_eq!(result.unwrap(), (10, 20));
}

#[test]
fn test_invalid_format() {
    let result = parse_port_range("1020");
    assert!(result.is_err());
}

#[test]
fn test_non_numeric() {
    let result = parse_port_range("abc-def");
    assert!(result.is_err());
}

#[test]
fn test_start_greater_than_end() {
    let result = parse_port_range("20-10");
    assert!(result.is_err());
}
