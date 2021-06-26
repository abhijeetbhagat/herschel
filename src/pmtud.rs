use crate::errors::PmtudError;
use pnet::packet::ip::IpNextHeaderProtocols::Icmp;
use pnet::packet::ipv4::Ipv4Flags::DontFragment;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ipv4::checksum as ipv4_checksum;
use pnet::packet::Packet;
use pnet::packet::{
    icmp::{
        destination_unreachable::DestinationUnreachablePacket,
        echo_request::MutableEchoRequestPacket, IcmpCode, IcmpType,
    },
};
use pnet::transport::{
    ipv4_packet_iter, transport_channel, TransportChannelType, TransportReceiver, TransportSender,
};
use log::{debug, info};
use std::net::Ipv4Addr;

pub struct Pmtud {
    tx: TransportSender,
    rx: TransportReceiver,
    destination: Ipv4Addr,
}

impl Pmtud {
    pub fn new(destination: Ipv4Addr) -> Result<Self, PmtudError> {
        if std::env::var("HERSCHEL_LOG").is_err() {
            std::env::set_var("HERSCHEL_LOG", "info");
        }
        env_logger::init();

        let (tx, rx) = transport_channel(65535, TransportChannelType::Layer3(Icmp))
            .map_err(|e| PmtudError::PmtudLayer3TransportInitError(e.to_string()))?;

        Ok(Self {
            tx,
            rx,
            destination,
        })
    }

    pub fn discover(&mut self) -> Result<u16, PmtudError> {
        const ICMP_PAYLOAD_LEN: usize = 1472; // we start with 1500 (ethernet mtu) - 28 (ip header + icmp header) bytes of payload
        let payload = [0; ICMP_PAYLOAD_LEN];
        // header 8 bytes + payload above
        let total_icmp_packet_len = 8 + ICMP_PAYLOAD_LEN;

        let mut icmp_packet =
            MutableEchoRequestPacket::owned(vec![0; total_icmp_packet_len]).unwrap();
        icmp_packet.set_icmp_type(IcmpType(8));
        icmp_packet.set_icmp_code(IcmpCode(0));
        icmp_packet.set_checksum(0);
        icmp_packet.set_sequence_number(0);
        icmp_packet.set_identifier(0);
        icmp_packet.set_payload(&payload);

        // let chksum = checksum(&icmp_packet.packet(), 16); // checksum starts at offset 16
        icmp_packet.set_checksum(0xf7ff);


        info!("sending ip packet to host {}", self.destination);
        let mut adjusted_icmp_payload_len = ICMP_PAYLOAD_LEN;
        let mut ipv4_packet = Pmtud::get_packet(total_icmp_packet_len, icmp_packet.packet(), self.destination);

        loop {
            match self.tx.send_to(ipv4_packet, self.destination.into()) {
                Err(e) => {
                    info!("{}", e);
                    adjusted_icmp_payload_len = adjusted_icmp_payload_len - 8;
                    ipv4_packet = Pmtud::get_packet(adjusted_icmp_payload_len, &icmp_packet.packet()[0..adjusted_icmp_payload_len], self.destination);
                },
                Ok(_size) => {
                    if let Ok((packet, _addr)) = ipv4_packet_iter(&mut self.rx).next() {
                        debug!("packet recvd: {:?}", packet.packet());
                        debug!("payload recvd: {:?}", packet.payload());
                        if packet.payload()[0] == 0 {
                            debug!("ping reply recvd");
                            return Ok((adjusted_icmp_payload_len + 28) as u16)
                        }

                        if let Some(icmp_packet) = DestinationUnreachablePacket::new(packet.payload()) {
                            debug!("converted packet");
                            let unused = icmp_packet.get_unused();
                            let next_hop_mtu = (unused & 0x0000ffff) as u16;
                            debug!("next hop mtu is {}", next_hop_mtu);
                            return Ok(next_hop_mtu) 
                        } else {
                            return Ok((adjusted_icmp_payload_len + 28) as u16)
                        }
                    } else {
                        return Err(PmtudError::PmtudRecvError);
                    }
                }
            }
        }

    }

    fn get_packet(total_icmp_packet_len: usize, payload: &[u8], destination: Ipv4Addr) -> MutableIpv4Packet  {
        let packet = vec![0u8; 20 + total_icmp_packet_len]; // 20 bytes header
        let mut packet = MutableIpv4Packet::owned(packet).unwrap();
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
        packet.set_destination(destination);
        packet.set_payload(payload);
        // the routers will recalc the checksum before forwarding since they decrease ttl by 1
        packet.set_checksum(ipv4_checksum(&packet.to_immutable()));
        packet
    }
}
