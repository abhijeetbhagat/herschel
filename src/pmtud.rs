use crate::errors::PmtudError;
use std::net::Ipv4Addr;
use pnet::packet::{icmp::Icmp, ip::IpNextHeaderProtocols::Ipv4};
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ip::IpNextHeaderProtocols::Icmp;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ipv4::checksum;
use pnet::transport::{
    transport_channel, TransportChannelType, TransportReceiver, TransportSender,
};
use pnet::packet::ipv4::Ipv4Flags::DontFragment;

pub struct Pmtud {
    tx: TransportSender,
    rx: TransportReceiver,
    destination: Ipv4Addr
}

impl Pmtud {
    pub fn new(destination: Ipv4Addr) -> Result<Self, PmtudError> {
        let (tx, rx) = transport_channel(
            1500,
            TransportChannelType::Layer3(Ipv4),
        )
        .map_err(|e| PmtudError::PmtudLayer3TransportInitError)?;

        Ok(Self {
            tx,
            rx,
            destination
        })
    }

    pub fn discover(&mut self) -> u8 {
        let mut packet = [0u8; 20];
        let mut packet = MutableIpv4Packet::new(&mut packet).unwrap();
        packet.set_version(4);
        packet.set_header_length(5);
        packet.set_dscp(0); // standard diff service class
        packet.set_ecn(1); // we support cogestion notification
        packet.set_total_length(20); // only header
        packet.set_identification(1);
        packet.set_flags(DontFragment);
        packet.set_fragment_offset(0);
        packet.set_ttl(10); // in seconds
        packet.set_next_level_protocol(Icmp);
        packet.set_source("127.0.0.1".parse().unwrap()); // the nat can change this address
        packet.set_destination(self.destination);
        // the routers will recalc the checksum before forwarding since they decrease ttl by 1
        packet.set_checksum(checksum(&packet.to_immutable())); 
        match self.tx.send_to(packet, self.destination.into()) {
            Err(e) => println!("there was a problem sending the ip packet to destination - {}", e),
            _ => {}
        }
        todo!()
    }
}
