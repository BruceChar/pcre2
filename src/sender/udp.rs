use super::Sender;
use anyhow::{bail, Result};
use std::net::UdpSocket;

const AUTO_FD: &str = "0.0.0.0:0";

pub struct Udp {
    socket: UdpSocket,
}

impl Udp {
    pub fn new(addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(AUTO_FD).expect("unable to bind");
        if let Err(e) = socket.connect(addr) {
            bail!(e);
        };
        Ok(Self { socket })
    }
}

impl Sender for Udp {
    fn send(&self, buf: &[u8]) -> Result<usize> {
        match self.socket.send(buf) {
            Ok(s) => Ok(s),
            Err(e) => bail!(e.to_string()),
        }
    }
}
