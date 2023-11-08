use crate::buffer::BUFFER_SIZE;
use libc::*;
use log::{debug, error};
use std::ffi::CString;

pub struct Socket {
    pub fd: i32,
    interface: String,
    interface_c: CString,
}

pub struct SockAddr {}

impl SockAddr {
    pub fn new_sockaddr_ll() -> sockaddr_ll {
        sockaddr_ll {
            sll_family: 0,
            sll_protocol: 0,
            sll_ifindex: 0,
            sll_hatype: 0,
            sll_pkttype: 0,
            sll_halen: 0,
            sll_addr: [0; 8],
        }
    }
}

impl Socket {
    /// Create a raw socket and bind it to a specific interface.
    pub fn new(interface: String) -> Result<Self, String> {
        let protocol = (ETH_P_ALL as i16).to_be() as c_int;
        let fd = unsafe { socket(AF_PACKET, SOCK_RAW, protocol) };
        if fd < 0 {
            Err(format!("socket: {}", fd))
        } else {
            debug!("opened socket with fd={}", fd);
            let sock = Self {
                fd,
                interface: interface.clone(),
                interface_c: CString::new(interface).unwrap(),
            };
            sock.bind(protocol)?;
            Ok(sock)
        }
    }

    /// Bind the sniffer to a specific interface.
    fn bind(&self, protocol: c_int) -> Result<(), String> {
        debug!("binding the socket to interface={}", self.interface);
        let res = unsafe {
            setsockopt(
                self.fd,
                SOL_SOCKET,
                SO_BINDTODEVICE,
                self.interface_c.as_ptr() as _,
                (self.interface.len() + 1) as _,
            )
        };
        if res < 0 {
            return Err(format!("setsockopt: {}", res));
        }
        let addr = sockaddr_ll {
            sll_family: AF_PACKET as u16,
            sll_protocol: protocol as u16,
            sll_ifindex: unsafe { if_nametoindex(self.interface_c.as_ptr()) } as i32,
            sll_hatype: 0,
            sll_pkttype: 0,
            sll_halen: 0,
            sll_addr: [0; 8],
        };
        let addr_ptr = (&addr) as *const sockaddr_ll;
        let addr_len = std::mem::size_of::<sockaddr_ll>();
        let res = unsafe { bind(self.fd, addr_ptr as _, addr_len as u32) };
        if res < 0 {
            return Err(format!("setsockopt: {}", res));
        }
        Ok(())
    }

    /// Set the network card in promiscuous mode.
    pub fn set_promiscuous(&self) -> Result<(), String> {
        debug!("setting the network card to promiscuous mode");
        let mut ethreq = ifreq {
            ifr_name: [0; IF_NAMESIZE],
            ifr_ifru: __c_anonymous_ifr_ifru { ifru_flags: 0 },
        };
        assert!(self.interface.len() <= IF_NAMESIZE); // <?
        ethreq.ifr_name[..self.interface.len()].clone_from_slice(
            &self
                .interface_c
                .as_bytes()
                .iter()
                .map(|&byte| byte as i8)
                .collect::<Vec<i8>>()[..],
        );
        if unsafe { ioctl(self.fd, SIOCGIFFLAGS, &ethreq) } == -1 {
            return Err(String::from("ioctl 1"));
        }
        unsafe { ethreq.ifr_ifru.ifru_flags |= IFF_PROMISC as i16 };
        if unsafe { ioctl(self.fd, SIOCSIFFLAGS, &ethreq) } == -1 {
            return Err(String::from("ioctl 2"));
        }
        Ok(())
    }

    /// Receive first `BUFFER_SIZE` packets of a buffer.
    pub fn recv(&self, buf: &[u8; BUFFER_SIZE]) -> Result<isize, String> {
        let n = unsafe { recv(self.fd, buf.as_ptr() as *mut c_void, buf.len(), 0) };
        if n < 0 {
            error!("failed to recv: {}", n);
            return Err(format!("recv: {}", n));
        }
        Ok(n)
    }

    /// Receive first `BUFFER_SIZE` packets of a buffer, and fill in socket
    /// address information.
    pub fn recvfrom(
        &self,
        addr: &mut sockaddr_ll,
        buf: &mut [u8; BUFFER_SIZE],
    ) -> Result<isize, String> {
        let mut socklen = std::mem::size_of::<sockaddr_ll>() as u32;
        // wrapping our own libc functions because nix-rust is buggy:
        // https://github.com/nix-rust/nix/pull/1896
        let n = unsafe {
            recvfrom(
                self.fd,
                buf.as_ptr() as *mut c_void,
                buf.len(),
                0,
                (addr as *mut sockaddr_ll) as _,
                &mut socklen,
            )
        };
        if n < 0 {
            error!("failed to recv: {}", n);
            return Err(format!("recv: {}", n));
        }
        Ok(n)
    }
}
