use std::error::Error;

pub const HANDSHAKE: u128 = u128::from_le_bytes(*b"floppanet\0\0\0\0\0\0\0");
pub const SERVER_PORT: u16 = 1999;
pub const SERVER: &str = "0.0.0.0";
// pub const SERVER: &str = "net.colon3.lol";

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
