// Ethernet (14), IP (20), TCP/UDP (8) headers + 32 bits from QUIC (4)
pub const BUFFER_SIZE: usize = 46;

#[derive(Debug)]
pub enum UdpParserError {
    WrongDirection,
    NotIp,
    NotUdp,
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
    pub fn parse(x: &[u8; BUFFER_SIZE]) -> Result<Self, UdpParserError> {
        let src_mac = x[0..4].iter().map(|b| format!("{:x}", b))
            .collect::<Vec<_>>().join(":");
        let dst_mac = x[4..8].iter().map(|b| format!("{:x}", b))
            .collect::<Vec<_>>().join(":");
        let src_ip = format!("{}.{}.{}.{}", x[26], x[27], x[28], x[29]);
        let dst_ip = format!("{}.{}.{}.{}", x[30], x[31], x[32], x[33]);
        let src_port = u16::from_be_bytes([x[34], x[35]]);
        let dst_port = u16::from_be_bytes([x[36], x[37]]);
        let identifier = u32::from_be_bytes([x[42], x[43], x[44], x[45]]);
        Ok(UdpParser {
            src_mac, dst_mac, src_ip, dst_ip, identifier, src_port, dst_port,
        })
    }
}
