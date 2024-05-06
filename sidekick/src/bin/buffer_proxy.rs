use clap::Parser;
use log::{debug, info};
use sidekick::{
    Socket,
    buffer::{Direction, UdpParser, BUFFER_SIZE},
    socket::SockAddr,
};

/// Sends quACKs in the sidekick protocol, receives data in the base protocol.
#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `eth1'.
    #[arg(long, short = 'i')]
    interface: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();

    let sock = Socket::new(args.interface.clone())?;
    sock.set_promiscuous()?;
    info!("tapping socket on fd={} interface={}", sock.fd, args.interface);

    // Loop over outgoing packets
    tokio::task::spawn_blocking(move || {
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
    });
    Ok(())
}
