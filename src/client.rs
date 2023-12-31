use std::env::args;
use tokio::{task, io};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use floppanet::{Result, SERVER, SERVER_PORT, HANDSHAKE};

#[tokio::main]
async fn main() -> Result {
  match args().nth(1).map(|a| a.parse()) {
    Some(Ok(local_port)) => {
      let mut server = TcpStream::connect((SERVER, SERVER_PORT)).await?;
      server.write_u128(HANDSHAKE).await?;
      let server_port = server.read_u16().await?;
      println!("floppanet");
      println!("{}:{} -> localhost:{}", SERVER, server_port, local_port);
      loop {
        let id = server.read_u128().await?;
        task::spawn(handle(id, local_port));
      }
    }
    Some(Err(_)) => println!("invalid port"),
    None => println!("usage: {} (port)", args().next().unwrap()),
  };
  Ok(())
}

async fn handle(id: u128, local_port: u16) -> Result {
  let mut stream = TcpStream::connect((SERVER, SERVER_PORT)).await?;
  stream.write_u128(id).await?;
  stream.flush().await?;
  io::copy_bidirectional(
    &mut stream,
    &mut TcpStream::connect(("0.0.0.0", local_port)).await?,
  )
  .await?;
  Ok(())
}
