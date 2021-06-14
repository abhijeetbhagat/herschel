use crate::errors::PmtudError;
use pnet::packet::{icmp::Icmp, ip::IpNextHeaderProtocols::Ipv4};
use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ip::IpNextHeaderProtocols::Icmp;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::transport::{
    transport_channel, TransportChannelType, TransportReceiver, TransportSender,
};
use pnet::packet::ipv4::Ipv4Flags::DontFragment;

pub struct Pmtud {
    tx: TransportSender,
    rx: TransportReceiver,
}

impl Pmtud {
    pub fn new() -> Result<Self, PmtudError> {
        let (tx, rx) = transport_channel(
            1500,
            TransportChannelType::Layer3(Ipv4),
        )
        .map_err(|e| PmtudError::PmtudLayer3TransportInitError)?;

        Ok(Self {
            tx,
            rx
        })
    }

    pub fn discover(&self) -> u8 {
        // TODO abhi -
        // construct an IPv4 packet with DF set

        todo!()
    }
}
