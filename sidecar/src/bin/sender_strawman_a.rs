use std::net::SocketAddr;

use clap::Parser;
use tokio;
use tokio::net::UdpSocket;

use sidecar::Socket;
use sidecar::socket::SockAddr;
use sidecar::buffer::{BUFFER_SIZE, Direction, UdpParser};
use quack::StrawmanAQuack;

/// Sends quACKs in the sidecar protocol, receives data in the base protocol.
#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `eth1'.
    #[arg(long, short = 'i')]
    interface: String,
    /// Address of the UDP socket to quack to e.g., <IP:PORT>.
    #[arg(long)]
    addr: SocketAddr,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();

    let send_sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let recv_sock = Socket::new(args.interface.clone())?;
    recv_sock.set_promiscuous()?;

    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut addr = SockAddr::new_sockaddr_ll();
    let ip_protocol = (libc::ETH_P_IP as u16).to_be();
    let mut count: u32 = 0;
    loop {
        let n = recv_sock.recvfrom(&mut addr, &mut buf).unwrap();
        if Direction::Incoming != addr.sll_pkttype.into() {
            continue;
        }
        if addr.sll_protocol != ip_protocol {
            continue;
        }
        if !UdpParser::is_udp(&buf) {
            continue;
        }
        if n != (BUFFER_SIZE as _) {
            continue;
        }
        count += 1;
        let quack = StrawmanAQuack {
            sidecar_id: UdpParser::parse_identifier(&buf),
            count,
        };
        let bytes = bincode::serialize(&quack).unwrap();
        send_sock.send_to(&bytes, args.addr).await.unwrap();
    }
}
