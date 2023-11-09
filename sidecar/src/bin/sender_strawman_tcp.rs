use std::net::SocketAddr;

use clap::Parser;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use quack::StrawmanAQuack;
use sidecar::buffer::{Direction, UdpParser, BUFFER_SIZE};
use sidecar::socket::SockAddr;
use sidecar::Socket;

/// Sends quACKs in the sidecar protocol, receives data in the base protocol.
#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `eth1'.
    #[arg(long, short = 'i')]
    interface: String,
    /// Address of the TCP socket to quack to e.g., <IP:PORT>.
    #[arg(long)]
    addr: SocketAddr,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();

    let sock = Socket::new(args.interface.clone())?;
    sock.set_promiscuous()?;
    let mut stream = loop {
        match TcpStream::connect(args.addr).await {
            Ok(stream) => {
                break stream;
            }
            Err(_) => {
                continue;
            }
        }
    };
    stream.set_nodelay(true).unwrap();

    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut addr = SockAddr::new_sockaddr_ll();
    let ip_protocol = (libc::ETH_P_IP as u16).to_be();
    loop {
        let n = match sock.recvfrom(&mut addr, &mut buf) {
            Ok(n) => n,
            Err(_) => {
                return Ok(());
            }
        };
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
        let quack = StrawmanAQuack {
            sidecar_id: UdpParser::parse_identifier(&buf),
        };
        let bytes = bincode::serialize(&quack).unwrap();
        stream.write_all(&bytes).await.unwrap();
        stream.flush().await.unwrap();
    }
}
