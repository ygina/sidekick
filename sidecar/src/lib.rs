use std::sync::{Arc, Mutex};
use quack::*;
use bincode;
use log::{trace, debug, info, error};
use tokio;
use tokio::{sync::mpsc, net::UdpSocket};

pub enum SidecarType {
    QuackSender,
    QuackReceiver,
}

pub struct Sidecar {
    pub ty: SidecarType,
    pub interface: String,
    pub threshold: usize,
    pub bits: usize,
    // TODO: is there a better way to do synchronization?
    quack_log: Arc<Mutex<(Quack, IdentifierLog)>>,
}

impl Sidecar {
    /// Create a new sidecar.
    pub fn new(ty: SidecarType, interface: &str, threshold: usize, bits: usize) -> Self {
        assert_eq!(bits, 32, "ERROR: <num_bits_id> must be 32");
        Self {
            ty,
            interface: interface.to_string(),
            threshold,
            bits,
            quack_log: Arc::new(Mutex::new((Quack::new(threshold), vec![]))),
        }
    }

    /// Start the raw socket that listens to the specified interface and
    /// accumulates those packets in a quACK. If the sidecar is a quACK sender,
    /// only listens for incoming packets. If the sidecar is a quACK receiver,
    /// only listens for outgoing packets, and additionally logs the packet
    /// identifiers.
    pub fn start(&self) {
        error!("unimplemented raw socket");
    }

    /// Receive quACKs on the given UDP port. Returns the channel on which
    /// to loop received quACKs.
    pub fn listen(&self, port: u16) -> mpsc::Receiver<Quack> {
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html
        // buffer up to 100 messages
        let (tx, rx) = mpsc::channel(100);
        let buf_len = {
            let quack = Quack::new(self.threshold);
            bincode::serialize(&quack).unwrap().len()
        };
        debug!("allocating {} bytes for receiving quACKs", buf_len);
        tokio::spawn(async move {
            let addr = format!("127.0.0.1:{}", port);
            info!("listening for quACKs on {}", addr);
            let socket = UdpSocket::bind(addr).await.unwrap();
            let mut buf = vec![0; buf_len];
            loop {
                let (nbytes, _) = socket.recv_from(&mut buf).await.unwrap();
                assert_eq!(nbytes, buf.len());
                // TODO: check that it's actually a quack
                let quack: Quack = bincode::deserialize(&buf).unwrap();
                trace!("received quACK with count {}", quack.count);
                tx.send(quack).await.unwrap();
            }
        });
        rx
    }

    /// Snapshot the quACK.
    pub fn quack(&self) -> Quack {
        self.quack_log.lock().unwrap().0.clone()
    }

    /// Snapshot the quACK and current log.
    pub fn quack_with_log(&self) -> (Quack, IdentifierLog) {
        // TODO: don't clone the log
        self.quack_log.lock().unwrap().clone()
    }

    /// Decode the quACK given the current snapshot.
    pub fn quack_decode(&self, quack: Quack) -> DecodedQuack {
        let (my_quack, my_log) = self.quack_with_log();
        let difference_quack = my_quack - quack;
        DecodedQuack::decode(difference_quack, my_log)
    }
}
