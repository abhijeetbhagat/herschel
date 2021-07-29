use herschel::pmtud::Pmtud;
use pnet::datalink::interfaces;

#[test]
fn test_get_local_ip() {
    let all_interfaces = interfaces();
    for interface in all_interfaces.iter() {
        println!("{}", interface);
    }
    let default_interface = all_interfaces
        .iter()
        .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty());

    match default_interface {
        Some(interface) => assert!(interface.name == "Wireless LAN adapter Wi-Fi"),
        _ => {
            panic!("error getting interfaces")
        }
    }
}
