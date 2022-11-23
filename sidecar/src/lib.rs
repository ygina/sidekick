use std::sync::{Arc, Mutex};
use std::mem::MaybeUninit;
use std::io::ErrorKind;
use std::ffi::CString;
use quack::*;
use bincode;
use tokio;
use tokio::{sync::mpsc, net::UdpSocket, runtime::Runtime};
use socket2::{Socket, Domain, Type, Protocol};

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
    quack: Arc<Mutex<Quack>>,
    log: IdentifierLog,
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
            quack: Arc::new(Mutex::new(Quack::new(threshold))),
            log: vec![],
        }
    }

    /// Start the raw socket that listens to the specified interface and
    /// accumulates those packets in a quACK. If the sidecar is a quACK sender,
    /// only listens for incoming packets. If the sidecar is a quACK receiver,
    /// only listens for outgoing packets, and additionally logs the packet
    /// identifiers.
    pub fn start(&self, rt: &Runtime) -> std::io::Result<()> {
        use libc::*;

        // Create a socket
        let sock = nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Packet,
                nix::sys::socket::SockType::Raw,
                nix::sys::socket::SockFlag::empty(),
                nix::sys::socket::SockProtocol::EthAll, // Udp
            ).unwrap();
        //let sock = unsafe { socket(PF_PACKET, SOCK_RAW, ETH_P_IP.to_be()) };
        //if sock < 0 {
        //    eprintln!("socket");
        //    return Err(ErrorKind::Other.into());
        //}
        println!("sock = {}", sock);

        /*
        // Bind the sniffer to a specific interface
        let interface = CString::new(self.interface.as_bytes()).unwrap();
        println!("sock = {}", sock);
        println!("{:?} {} bytes", interface, self.interface.as_bytes().len());
        if unsafe { setsockopt(
            sock,
            SOL_SOCKET,
            SO_BINDTODEVICE,
            interface.as_ptr() as _,
            (self.interface.as_bytes().len() + 1) as _,
        ) } < 0 {
            eprintln!("setsockopt");
            return Err(ErrorKind::Other.into());
        }
        */

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
        let mut buf: [i8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let quack = self.quack.clone();
        rt.spawn(async move {
            println!("hello");
            loop {
                let n = unsafe { recvfrom(
                    3,
                    buf.as_mut_ptr() as _,
                    BUFFER_SIZE,
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ) };
                // Packet contains at least Ethernet (14), IP (20),
                // and TCP/UDP (8) headers
                //if n < 42 {
                //    eprintln!("received <42 bytes");
                //    continue;
                //}
                if n < 0 {
                    eprintln!("error");
                    break;
                } else {
                    println!(" received {} bytes", n);
                }
                let identifier = 100;
                quack.lock().unwrap().insert(identifier);
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
            let (nbytes, _) = socket.recv_from(&mut buf).await.unwrap();
            assert_eq!(nbytes, buf.len());
            // TODO: check that it's actually a quack
            let quack: Quack = bincode::deserialize(&buf).unwrap();
            tx.send(quack).await.unwrap();
        });
        rx
    }

    /// Snapshot the quACK.
    pub fn quack(&self) -> Quack {
        self.quack.lock().unwrap().clone()
    }

    /// Snapshot the quACK and current log.
    pub fn quack_with_log(&self) -> (Quack, &IdentifierLog) {
        unimplemented!()
    }

    /// Decode the quACK given the current snapshot.
    pub fn quack_decode(&self, quack: Quack) -> DecodedQuack {
        unimplemented!()
    }
}
