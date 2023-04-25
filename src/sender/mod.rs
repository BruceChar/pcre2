mod udp;
use anyhow::Result;
pub use udp::*;
pub trait Sender {
    fn send(&self, buf: &[u8]) -> Result<usize>;
}
