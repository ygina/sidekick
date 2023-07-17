use libc::c_uchar;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// Ethernet (14), IP (20), TCP/UDP (8) headers
// The randomly-encrypted payload in a QUIC packet with a short header is at
// offset 63.
pub const ID_OFFSET: usize = 63;
pub const BUFFER_SIZE: usize = ID_OFFSET+4;

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Incoming,
    Outgoing,
    Unknown,
}

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/if_packet.h
pub const PACKET_HOST: c_uchar = 0;
pub const PACKET_OTHERHOST: c_uchar = 3;
pub const PACKET_OUTGOING: c_uchar = 4;

impl Into<Direction> for c_uchar {
    fn into(self) -> Direction {
        match self {
            PACKET_HOST | PACKET_OTHERHOST => Direction::Incoming,
            PACKET_OUTGOING => Direction::Outgoing,
            _ => Direction::Unknown,
        }
    }
}

pub struct UdpParser {
    pub src_mac: String,
    pub dst_mac: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub identifier: u32,
}

impl UdpParser {
    pub fn _parse(x: &[u8; BUFFER_SIZE]) -> Option<Self> {
        let ip_protocol = x[23];
        if i32::from(ip_protocol) != libc::IPPROTO_UDP {
            return None;
        }

        let src_mac = x[0..4].iter().map(|b| format!("{:x}", b))
            .collect::<Vec<_>>().join(":");
        let dst_mac = x[4..8].iter().map(|b| format!("{:x}", b))
            .collect::<Vec<_>>().join(":");
        let src_ip = format!("{}.{}.{}.{}", x[26], x[27], x[28], x[29]);
        let dst_ip = format!("{}.{}.{}.{}", x[30], x[31], x[32], x[33]);
        let src_port = u16::from_be_bytes([x[34], x[35]]);
        let dst_port = u16::from_be_bytes([x[36], x[37]]);
        let identifier = u32::from_be_bytes([
            x[ID_OFFSET],
            x[ID_OFFSET+1],
            x[ID_OFFSET+2],
            x[ID_OFFSET+3],
        ]);
        Some(UdpParser {
            src_mac, dst_mac, src_ip, dst_ip, identifier, src_port, dst_port,
        })
    }

    /// Returns True if and only if the buffer represents a UDP packet.
    pub fn is_udp(x: &[u8; BUFFER_SIZE]) -> bool {
        let ip_protocol = x[23];
        i32::from(ip_protocol) == libc::IPPROTO_UDP
    }

    /// Returns the src_addr assuming the buffer represents a UDP packet.
    pub fn parse_src_addr(x: &[u8; BUFFER_SIZE]) -> SocketAddr {
        let ip = [x[26], x[27], x[28], x[29]];
        let port = u16::from_be_bytes([x[34], x[35]]);
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
    }

    /// Returns the dst_addr assuming the buffer represents a UDP packet.
    pub fn parse_dst_addr(x: &[u8; BUFFER_SIZE]) -> SocketAddr {
        let ip = [x[30], x[31], x[32], x[33]];
        let port = u16::from_be_bytes([x[36], x[37]]);
        SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port)
    }

    /// Returns the sidecar identifier assuming the buffer represents
    /// a QUIC UDP packet.
    pub fn parse_identifier(x: &[u8; BUFFER_SIZE]) -> u32 {
        u32::from_be_bytes([
            x[ID_OFFSET],
            x[ID_OFFSET+1],
            x[ID_OFFSET+2],
            x[ID_OFFSET+3],
        ])
    }
}
