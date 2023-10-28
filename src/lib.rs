use std::error::Error;
use tokio::io::{self, AsyncRead, AsyncWrite};

pub const CLIENT_PORT: u16 = 1998;
pub const SERVER_PORT: u16 = 1999;
pub const SERVER: &str = "0.0.0.0";
// pub const SERVER: &str = "net.chxry.me";

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub async fn proxy<A: AsyncRead + AsyncWrite, B: AsyncRead + AsyncWrite>(a: A, b: B) -> Result {
  let mut a = io::split(a);
  let mut b = io::split(b);
  tokio::select! {
      res = io::copy(&mut a.0, &mut b.1) => res,
      res = io::copy(&mut b.0, &mut a.1) => res,
  }?;
  Ok(())
}
