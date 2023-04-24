use super::Sender;
use std::net::UdpSocket;

const AUTO_FD: &'static str = "0.0.0.0:0";

pub struct Udp {
    socket: UdpSocket,
}

impl Udp {
    pub fn new(addr: &str) -> Result<Self, String> {
        let socket = UdpSocket::bind(AUTO_FD).expect("unable to bind");
        if let Err(e) = socket.connect(addr) {
            return Err(e.to_string());
        };
        Ok(Self { socket })
    }
}

impl Sender for Udp {

    fn send(&self, buf: &[u8]) -> Result<usize, String> {
        match self.socket.send(buf) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }
}
