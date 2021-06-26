use herschel::pmtud::Pmtud;

#[test]
fn test_icmp() {
    match Pmtud::new(
        "192.168.1.10".parse().unwrap(),
        "142.250.183.3".parse().unwrap(),
    ) {
        Err(e) => println!("err is {}", e),
        Ok(mut pmtud) => {
            let result = pmtud.discover();
            assert!(result.is_ok());
            assert_eq!(result, Ok(1492u16));
        }
    };
}
