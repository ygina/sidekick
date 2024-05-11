use clap::Parser;
use log::{trace, debug, info, warn};
use sidekick::{
    Socket,
    buffer::{Direction, UdpParser, BUFFER_SIZE},
    socket::SockAddr,
};
use quack::{
    PowerSumQuack, PowerSumQuackU32,
    arithmetic::{self, ModularArithmetic, ModularInteger},
};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::time::{Instant, Duration};

#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `r1-eth1'.
    #[arg(long, short = 'i')]
    interface: String,
    /// QuACK threshold.
    #[arg(long, short = 't')]
    threshold: usize,
}

#[derive(Debug, Default)]
struct DecodedQuack {
    // In increasing order.
    missing_indexes: Vec<usize>,
    missing_ids: HashSet<u32>,
    acked_ids: HashSet<u32>,
}

struct Sidekick {
    quack: PowerSumQuackU32,

    sidekick_reset_port: u16,
    sidekick_reset_threshold: Duration,

    last_decoded_quack_count: u32,
    last_quack_reset: Instant,

    sidekick_log: Vec<u32>,
    // Same length as `sidekick_log`
    sidekick_bytes: Vec<Vec<u8>>,

    // Raw socket for retransmitting packets to the data receiver
    sendsock: Socket,
}

impl Sidekick {
    fn new(threshold: usize) -> Self {
        Self {
            quack: PowerSumQuackU32::new(threshold),
            sidekick_reset_port: 1234,
            sidekick_reset_threshold: Duration::from_millis(10),
            last_decoded_quack_count: 0,
            last_quack_reset: Instant::now(),
            sidekick_log: Default::default(),
            sidekick_bytes: Default::default(),
            sendsock: Socket::new(String::from("r1-eth0")).unwrap(),
        }
    }

    async fn reset_quack(&mut self, mut addr: SocketAddr, now: Instant) -> std::io::Result<()> {
        // This time threshold should be long enough that if the host and proxy
        // are not in a valid state at this point, we can assume the previous
        // reset got lost.
        if now - self.last_quack_reset > self.sidekick_reset_threshold {
            let sock = UdpSocket::bind("0.0.0.0:0").await?;
            addr.set_port(self.sidekick_reset_port);
            sock.send_to(&[123], addr).await?;

            // Reset internal quack state
            self.last_quack_reset = now;
            self.quack = PowerSumQuack::new(self.quack.threshold());
            self.last_decoded_quack_count = 0;
            self.sidekick_log.clear();
            self.sidekick_bytes.clear();
        }
        Ok(())
    }

    fn process_outgoing_packet(&mut self, id: u32, bytes: Vec<u8>) {
        self.sidekick_log.push(id);
        self.sidekick_bytes.push(bytes);
    }

    fn decode_incoming_quack(&self, diff_quack: PowerSumQuackU32, log: &[u32]) -> Option<DecodedQuack> {
        // We'd be calling this if there are missing packets in the suffix.
        if diff_quack.count() == 0 {
            return Some(DecodedQuack {
                acked_ids: log.iter().copied().collect(),
                ..Default::default()
            });
        }

        let mut decoded = DecodedQuack::default();
        let mut coeffs = diff_quack.to_coeffs();
        for (index, &id) in log.iter().enumerate() {
            if coeffs.is_empty() || arithmetic::eval(&coeffs, id).value() != 0 {
                decoded.acked_ids.insert(id);
            } else {
                // Divide the coefficients by the binomial representing the
                // missing sidekick id
                let mod_id = ModularInteger::<u32>::new(id);
                if coeffs.len() == 1 {
                    assert_eq!(coeffs.pop(), Some(mod_id.neg()));
                } else {
                    coeffs[0].add_assign(mod_id);
                    for i in 1..coeffs.len() {
                        let addend = coeffs[i-1].mul(mod_id);
                        coeffs[i].add_assign(addend);
                    }
                    assert_eq!(coeffs.pop().unwrap().value(), 0);
                }

                // Track the missing id and its index in the log
                decoded.missing_indexes.push(index);
            }
        }

        if !coeffs.is_empty() {
            // unable to decode all missing packets
            return None;
        }

        decoded.missing_ids = decoded.missing_indexes.iter().map(|&index| log[index]).collect();
        if decoded.missing_ids.len() < decoded.missing_indexes.len() {
            // It is very unlikely that two packets have the same
            // identifier if they are truly different packets. In the
            // rare case that a duplicate identifier represents
            // different packets and not all the packets are missing,
            // it is possible we spuriously retransmit the wrong packet,
            // and the truly missing packet is later addressed in QUIC's
            // end-to-end retransmission mechanism.
            //
            // In the more likely scenario the same packet was just
            // sent multiple times, `missing_indexes` would include all
            // of them.
            warn!("duplicate ID is missing");
        }

        Some(decoded)
    }

    /// Garbage collect and retransmit any missing packets.
    fn handle_decoded_quack(&mut self, decoded: DecodedQuack, next_log_index: usize) {
        // Everything we drain from the log has already been determined to
        // be quacked or lost. Remove the lost packets from the quack. Remove
        // any potentially in-flight packets from the quack.
        println!("quack threshold={} acked={} missing={:?} suffix={}",
            self.quack.threshold(), decoded.acked_ids.len(),
            decoded.missing_ids, self.sidekick_log.len() - next_log_index);
        for index in decoded.missing_indexes {
            self.quack.remove(self.sidekick_log[index]);
            // Retransmit missing bytes and add it to the end of the log since
            // we just retransmitted a packet in that connection
            self.sidekick_bytes.push(vec![]);
            let mut bytes = self.sidekick_bytes.swap_remove(index);

            self.sendsock.send(&bytes).expect(
                "failed to retransmit missing packets on send socket");
            println!("retransmit {} id={} bytes={}",
                u32::from_be_bytes([bytes[42], bytes[43], bytes[44], bytes[45]]),
                self.sidekick_log[index], bytes.len());
            self.sidekick_log.push(self.sidekick_log[index]);
            self.sidekick_bytes.push(bytes.clone());
        }
        self.sidekick_log.drain(..next_log_index);
        self.sidekick_bytes.drain(..next_log_index);
    }

    /// Return false if quack needs to be reset
    fn process_incoming_quack(&mut self, quack: PowerSumQuackU32) -> bool {
        // Immediately return if we've already processed this quack.
        // Cache the count of the last decoded quack to avoid duplicate work.
        // Immediately return if no packets have been received.
        let count = quack.count();
        if self.last_decoded_quack_count == count || count == 0 {
            return true;
        } else {
            self.last_decoded_quack_count = count;
        }

        // Add up to the `last_value()` received at the sender.
        let mut next_log_index = 0;
        while next_log_index < self.sidekick_log.len() {
            let id = self.sidekick_log[next_log_index];
            next_log_index += 1;
            self.quack.insert(id);
            if Some(id) == quack.last_value() {
                // If this condition isn't met in the loop, eventually the
                // quack will fail to decode the packets in this quack are not
                // a subset of our own quack, and we'll reset.
                break;
            }
        }

        // We can't decode the quACK if the difference in the number of packets
        // sent and received exceeds the threshold. Send a RESET packet to the
        // proxy to resynchronize. The host keeps resending RESET packets with
        // the same quack epoch in response to each quack until it receives a
        // quack that it can decode.
        let threshold = self.quack.threshold() as u32;
        if self.quack.count() > count + threshold {
            println!("reset exceeded quack threshold {} > {}", self.quack.count() - count, threshold);
            return false;
        }

        // Either the counts overflowed, or we sent a RESET packet that hasn't
        // been synchronized at the proxy yet. Either way, send a RESET if it
        // has been more than an RTT (of the quack subpath).
        if self.quack.count() < count {
            println!("reset overflowed or sender hasn't processed reset, expected {} <= {}",
                count, self.quack.count());
            return false;
        }

        // Find the missing packets that are not in the suffix.
        let decoded = if let Some(decoded) = self.decode_incoming_quack(
            self.quack.clone().sub(quack),
            &self.sidekick_log[..next_log_index],
        ) {
            decoded
        } else {
            println!("reset failed to decode");
            return false;
        };

        self.handle_decoded_quack(decoded, next_log_index);
        true
    }
}

fn buffering_loop(sidekick: Arc<Mutex<Sidekick>>, interface: String) -> Result<(), String> {
    let sock = Socket::new(interface.clone())?;
    sock.set_promiscuous()?;
    debug!("tapping socket on fd={} interface={}", sock.fd, interface);

    let mut buf: [u8; 1500] = [0; 1500];
    let mut addr = SockAddr::new_sockaddr_ll();
    let ip_protocol = (libc::ETH_P_IP as u16).to_be();
    while let Ok(n) = sock.recvfrom(&mut addr, &mut buf) {
        if Direction::Incoming != addr.sll_pkttype.into() {
            continue;
        }
        if addr.sll_protocol != ip_protocol {
            continue;
        }
        if !UdpParser::is_udp(&buf) {
            continue;
        }

        // Parse the identifier and store it in the buffer.
        if n < (BUFFER_SIZE as _) {
            continue;
        }
        let id = UdpParser::parse_identifier(&buf);
        debug!("insert {} ({:#10x})", id, id);
        sidekick.lock().unwrap().process_outgoing_packet(id, buf[..(n as usize)].to_vec());
    }
    Ok(())
}

async fn quack_listener_loop(sidekick: Arc<Mutex<Sidekick>>) -> std::io::Result<()> {
    let mut buf: [u8; 1500] = [0; 1500];
    let recvsock = UdpSocket::bind("0.0.0.0:5103").await.unwrap();

    loop {
        let (nbytes, src) = recvsock.recv_from(&mut buf).await?;
        let quack: PowerSumQuackU32 = bincode::deserialize(&buf[..nbytes]).unwrap();

        let mut sidekick = sidekick.lock().unwrap();
        if !sidekick.process_incoming_quack(quack) {
            sidekick.reset_quack(src, Instant::now()).await.unwrap();
        }
        drop(sidekick);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    let sidekick = Arc::new(Mutex::new(Sidekick::new(args.threshold)));

    let sidekick_clone = sidekick.clone();
    tokio::task::spawn_blocking(move || {
        buffering_loop(sidekick_clone, args.interface)
    });
    quack_listener_loop(sidekick).await.unwrap();
    Ok(())
}
