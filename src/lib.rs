use std::error::Error;

pub const HANDSHAKE: u64 = u64::from_le_bytes(*b"floppa\0\0");
pub const SERVER_PORT: u16 = 1999;
pub const SERVER: &str = "0.0.0.0";
// pub const SERVER: &str = "net.chxry.me";

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
