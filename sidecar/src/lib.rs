use quack::*;

pub enum SidecarType {
    QuackSender,
    QuackReceiver,
}

pub struct Sidecar {
    pub ty: SidecarType,
    pub interface: String,
    pub threshold: usize,
    pub bits: usize,
    log: IdentifierLog,
}

impl Sidecar {
    /// Create a new sidecar.
    pub fn new(ty: SidecarType, interface: &str, threshold: usize, bits: usize) -> Self {
        unimplemented!()
    }

    /// Start the raw socket that listens to the specified interface and
    /// accumulates those packets in a quACK. If the sidecar is a quACK sender,
    /// only listens for incoming packets. If the sidecar is a quACK receiver,
    /// only listens for outgoing packets, and additionally logs the packet
    /// identifiers.
    pub fn start(&self) {
        unimplemented!()
    }

    /// Receive quACKs on the given UDP port. Returns the channel on which
    /// to loop received quACKs.
    pub fn listen(&self, port: u16) -> std::sync::mpsc::Receiver<Quack> {
        unimplemented!()
    }

    /// Snapshot the quACK.
    pub fn quack(&self) -> Quack {
        unimplemented!()
    }

    /// Snapshot the quACK and current log.
    pub fn quack_with_log(&self) -> (Quack, IdentifierLog) {
        unimplemented!()
    }

    /// Decode the quACK given the current snapshot.
    pub fn quack_decode(&self, quack: Quack) -> DecodedQuack {
        unimplemented!()
    }
}
