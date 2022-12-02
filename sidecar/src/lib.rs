use std::sync::{Arc, Mutex};
use quack::*;
use bincode;
use log::{trace, debug, info, error};
use tokio;
use tokio::{sync::mpsc, net::UdpSocket};

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
    pub fn start(&self) -> Result<(), String> {
        use libc::*;

        // Create a socket
        let protocol = (ETH_P_IP as i16).to_be() as c_int;
        let sock = unsafe { socket(AF_PACKET, SOCK_RAW, protocol) };
        if sock < 0 {
            return Err(format!("socket: {}", sock));
        }
        info!("opened socket with fd={}", sock);

        // Bind the sniffer to a specific interface
        info!("binding the socket to interface={}", self.interface);
        let interface = std::ffi::CString::new(self.interface.clone()).unwrap();
        let res = unsafe { setsockopt(
            sock,
            SOL_SOCKET,
            SO_BINDTODEVICE,
            interface.as_ptr() as _,
            (self.interface.len() + 1) as _,
        ) };
        if res < 0 {
            return Err(format!("setsockopt: {}", res));
        }

        // Set the network card in promiscuous mode
        info!("setting the network card to promiscuous mode");
        let mut ethreq = ifreq {
            ifr_name: [0; IF_NAMESIZE],
            ifr_ifru: __c_anonymous_ifr_ifru {
                ifru_flags: 0,
            },
        };
        assert!(self.interface.len() <= IF_NAMESIZE); // <?
        ethreq.ifr_name[..self.interface.len()].clone_from_slice(
            &interface.as_bytes().iter()
                .map(|&byte| byte as i8).collect::<Vec<i8>>()[..]);
        if unsafe { ioctl(sock, SIOCGIFFLAGS, &ethreq) } == -1 {
            return Err(String::from("ioctl 1"));
        }
        unsafe { ethreq.ifr_ifru.ifru_flags |= IFF_PROMISC as i16 };
        if unsafe { ioctl(sock, SIOCSIFFLAGS, &ethreq) } == -1 {
            return Err(String::from("ioctl 2"));
        }

        // Loop over received packets
        let buf = [0; BUFFER_SIZE];
        let quack_log = self.quack_log.clone();
        let ty = self.ty.clone();
        tokio::spawn(async move {
            debug!("tapping raw socket");
            loop {
                let n = unsafe { recv(
                    sock,
                    buf.as_ptr() as *mut c_void,
                    buf.len(),
                    0,
                ) };
                if n < 0 {
                    error!("failed to recv: {}", n);
                    return;
                }

                let identifier = 100;  // TODO: extract identifier from buf
                {
                    let mut quack_log = quack_log.lock().unwrap();
                    quack_log.0.insert(identifier);
                    if ty == SidecarType::QuackReceiver {
                        quack_log.1.push(identifier);
                    }
                }
                trace!("received {} bytes", n);
            }
        });
        Ok(())
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
