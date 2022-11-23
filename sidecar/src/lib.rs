use std::sync::{Arc, Mutex};
use quack::*;
use bincode;
use tokio;
use tokio::{sync::mpsc, net::UdpSocket, runtime::Runtime};

#[derive(Clone, PartialEq, Eq)]
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

const BUFFER_SIZE: usize = 65536;

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
    pub fn start(&self, rt: &Runtime) -> std::io::Result<()> {
        use nix::sys::socket::*;

        // Create a socket
        let sock = socket(
            AddressFamily::Packet,
            SockType::Raw,
            SockFlag::empty(),
            SockProtocol::EthAll, // Udp
        ).unwrap();
        println!("sock = {}", sock);

        // Bind the sniffer to a specific interface
        setsockopt(
            sock,
            sockopt::BindToDevice,
            &self.interface.clone().into(),
        ).unwrap();

        // Set the network card in promiscuous mode
        /*
        #[repr(C)]
        #[derive(Default)]
        struct ifreq {
            ifr_name: [u8; IF_NAMESIZE],
            ifr_flags: c_int,  // short?
        }
        let mut ethreq = ifreq::default();
        {
            let if_len = interface.as_bytes().len();
            assert!(if_len <= IF_NAMESIZE);
            ethreq.ifr_name[..if_len].clone_from_slice(interface.as_bytes());
        }
        if (unsafe { ioctl(sock, SIOCGIFFLAGS, &ethreq) } == -1) {
            eprintln!("ioctl 1");
            return Err(ErrorKind::Other.into());
        }
        ethreq.ifr_flags |= IFF_PROMISC;
        if (unsafe { ioctl(sock, SIOCSIFFLAGS, &ethreq) } == -1) {
            eprintln!("ioctl 2");
            return Err(ErrorKind::Other.into());
        }
        */

        // Loop over received packets
        let mut buf = [0; BUFFER_SIZE];
        let quack_log = self.quack_log.clone();
        let ty = self.ty.clone();
        rt.spawn(async move {
            println!("looping");
            loop {
                let (n, _) = recvfrom::<SockaddrStorage>(
                    sock,
                    &mut buf,
                ).unwrap();
                let identifier = 100;
                {
                    let mut quack_log = quack_log.lock().unwrap();
                    quack_log.0.insert(identifier);
                    if ty == SidecarType::QuackReceiver {
                        quack_log.1.push(identifier);
                    }
                }
                println!("received {} bytes", n);
            }
        });
        Ok(())
    }

    /// Receive quACKs on the given UDP port. Returns the channel on which
    /// to loop received quACKs.
    pub fn listen(&self, port: u16, rt: &Runtime) -> mpsc::Receiver<Quack> {
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html
        // buffer up to 100 messages
        let (tx, rx) = mpsc::channel(100);
        let buf_len = {
            let quack = Quack::new(self.threshold);
            bincode::serialize(&quack).unwrap().len()
        };
        rt.spawn(async move {
            let addr = format!("127.0.0.1:{}", port);
            let socket = UdpSocket::bind(addr).await.unwrap();
            let mut buf = vec![0; buf_len];
            loop {
                let (nbytes, _) = socket.recv_from(&mut buf).await.unwrap();
                assert_eq!(nbytes, buf.len());
                // TODO: check that it's actually a quack
                let quack: Quack = bincode::deserialize(&buf).unwrap();
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
        DecodedQuack::decode(&difference_quack, &my_log)
    }
}
