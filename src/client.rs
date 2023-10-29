use std::env::args;
use tokio::task;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use floppanet::{Result, proxy, SERVER, SERVER_PORT, HANDSHAKE};

#[tokio::main]
async fn main() -> Result {
  match args().nth(1).map(|a| a.parse()) {
    Some(Ok(local_port)) => {
      let mut server = TcpStream::connect((SERVER, SERVER_PORT)).await?;
      server.write_u64(HANDSHAKE).await?;
      let server_port = server.read_u16().await?;
      println!("{}:{} -> localhost:{}", SERVER, server_port, local_port);
      loop {
        let id = server.read_u64().await?;
        task::spawn(handle(id, local_port));
      }
    }
    Some(Err(_)) => println!("invalid port"),
    None => println!("usage: {} (port)", args().nth(0).unwrap()),
  };
  Ok(())
}

async fn handle(id: u64, local_port: u16) -> Result {
  let mut stream = TcpStream::connect((SERVER, SERVER_PORT)).await?;
  stream.write_u64(id).await?;
  proxy(stream, TcpStream::connect(("0.0.0.0", local_port)).await?).await?;
  Ok(())
}
