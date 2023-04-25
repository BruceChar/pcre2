mod udp;
pub use udp::*;
use anyhow::Result;
pub trait Sender {
    fn send(&self, buf: &[u8]) -> Result<usize>;
}