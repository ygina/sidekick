use clap::Parser;
use log::{trace, debug, info};
use sidekick::{
    Socket,
    buffer::{Direction, UdpParser, BUFFER_SIZE},
    socket::SockAddr,
};
use quack::{PowerSumQuack, PowerSumQuackU32};
use tokio::net::UdpSocket;

#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `eth1'.
    #[arg(long, short = 'i')]
    interface: String,
}

fn buffering_loop(interface: String) -> Result<(), String> {
    let sock = Socket::new(interface.clone())?;
    sock.set_promiscuous()?;
    info!("tapping socket on fd={} interface={}", sock.fd, interface);

    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut addr = SockAddr::new_sockaddr_ll();
    let ip_protocol = (libc::ETH_P_IP as u16).to_be();
    while let Ok(n) = sock.recvfrom(&mut addr, &mut buf) {
        if Direction::Outgoing != addr.sll_pkttype.into() {
            continue;
        }
        if addr.sll_protocol != ip_protocol {
            continue;
        }
        if !UdpParser::is_udp(&buf) {
            continue;
        }

        // Parse the identifier and store it in the buffer.
        if n != (BUFFER_SIZE as _) {
            continue;
        }
        let id = UdpParser::parse_identifier(&buf);
        debug!("insert {} ({:#10x})", id, id);
    }
    Ok(())
}

async fn quack_listener_loop() -> std::io::Result<()> {
    let mut buf: [u8; 1500] = [0; 1500];
    let recvsock = UdpSocket::bind("0.0.0.0:5103").await.unwrap();
    loop {
        let (nbytes, src) = recvsock.recv_from(&mut buf).await?;
        let quack: PowerSumQuackU32 = bincode::deserialize(&buf[..nbytes]).unwrap();
        trace!("received quack: threshold={} count={}", quack.threshold(), quack.count());
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();

    tokio::task::spawn_blocking(move || buffering_loop(args.interface).unwrap());
    quack_listener_loop().await.unwrap();
    Ok(())
}
