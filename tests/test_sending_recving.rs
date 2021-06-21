use herschel::pmtud::Pmtud;

#[test]
fn test_icmp() {
    match Pmtud::new("142.250.183.3".parse().unwrap()) {
        Err(e) => println!("err is {}", e),
        Ok(mut pmtud) => {
            assert!(pmtud.discover().is_ok());
        }
    };
}
