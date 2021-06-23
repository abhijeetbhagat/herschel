use crate::errors::PmtudError;
use pnet::packet::icmp::checksum as icmp_checksum;
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ip::IpNextHeaderProtocols::Icmp;
use pnet::packet::ipv4::checksum as ipv4_checksum;
use pnet::packet::ipv4::Ipv4Flags::DontFragment;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::util::checksum;
use pnet::packet::Packet;
use pnet::packet::{
    icmp::{
        destination_unreachable::DestinationUnreachablePacket,
        echo_request::MutableEchoRequestPacket, Icmp, IcmpCode, IcmpType,
    },
    ip::IpNextHeaderProtocols::Ipv4,
};
use pnet::transport::{
    ipv4_packet_iter, transport_channel, TransportChannelType, TransportReceiver, TransportSender,
};
use std::net::Ipv4Addr;

pub struct Pmtud {
    tx: TransportSender,
    rx: TransportReceiver,
    destination: Ipv4Addr,
}

impl Pmtud {
    pub fn new(destination: Ipv4Addr) -> Result<Self, PmtudError> {
        let (tx, rx) = transport_channel(65535, TransportChannelType::Layer3(Icmp))
            .map_err(|e| PmtudError::PmtudLayer3TransportInitError(e.to_string()))?;

        Ok(Self {
            tx,
            rx,
            destination,
        })
    }

    pub fn discover(&mut self) -> Result<u16, PmtudError> {
        const icmp_payload_len: usize = 1465; // "abcdefghijklmnopqrstuvwabcdefghi".as_bytes();
        let payload = [0; icmp_payload_len];
        // header 8 bytes + payload (above) 65500 bytes
        let total_icmp_packet_len = 8 + icmp_payload_len;

        let mut icmp_packet =
            MutableEchoRequestPacket::owned(vec![0; total_icmp_packet_len]).unwrap();
        icmp_packet.set_icmp_type(IcmpType(8));
        icmp_packet.set_icmp_code(IcmpCode(0));
        icmp_packet.set_checksum(0);
        icmp_packet.set_sequence_number(0);
        icmp_packet.set_identifier(0);
        icmp_packet.set_payload(&payload);

        let chksum = checksum(&icmp_packet.packet(), 16); // checksum starts at offset 16
        icmp_packet.set_checksum(0xf7ff);

        let mut packet = vec![0u8; 20 + total_icmp_packet_len]; // 20 bytes header + 40 bytes icmp
        let mut packet = MutableIpv4Packet::new(&mut packet).unwrap();
        packet.set_version(4);
        packet.set_header_length(5);
        packet.set_dscp(0); // standard diff service class
        packet.set_ecn(1); // we support congestion notification
        packet.set_total_length(20 + total_icmp_packet_len as u16);
        packet.set_identification(1);
        packet.set_flags(DontFragment);
        packet.set_fragment_offset(0);
        packet.set_ttl(10); // in seconds
        packet.set_next_level_protocol(Icmp);
        packet.set_source("192.168.1.10".parse().unwrap()); // nats can change this address
        packet.set_destination(self.destination);
        packet.set_payload(icmp_packet.packet());
        // the routers will recalc the checksum before forwarding since they decrease ttl by 1
        packet.set_checksum(ipv4_checksum(&packet.to_immutable()));
        println!("sending ip packet to host {}", self.destination);
        match self.tx.send_to(packet, self.destination.into()) {
            Err(e) => println!(
                "there was a problem sending the ip packet to destination - {}",
                e
            ),
            Ok(size) => {
                if let Ok((packet, addr)) = ipv4_packet_iter(&mut self.rx).next() {
                    println!("packet recvd: {:?}", packet);
                    println!("payload recvd: {:?}", packet.payload());
                    if let Some(icmp_packet) = DestinationUnreachablePacket::new(packet.payload()) {
                        println!("converted packet");
                        let unused = icmp_packet.get_unused();
                        let next_hop_mtu = (unused & 0x0000ffff) as u16;
                        return Ok(next_hop_mtu)
                    } else {
                        println!("conversion failed");
                    }
                } else {
                    return Err(PmtudError::PmtudRecvError);
                }
            }
        }
        Ok(1500)
    }
}
