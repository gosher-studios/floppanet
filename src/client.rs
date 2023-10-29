use std::env::args;
use std::time::Duration;
use tokio::{task, io, time};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, tcp::OwnedWriteHalf};
use floppanet::{Result, SERVER, SERVER_PORT, HANDSHAKE};

#[tokio::main]
async fn main() -> Result {
  match args().nth(1).map(|a| a.parse()) {
    Some(Ok(local_port)) => {
      let (mut read, mut write) = TcpStream::connect((SERVER, SERVER_PORT))
        .await?
        .into_split();
      write.write_u64(HANDSHAKE).await?;
      let server_port = read.read_u16().await?;
      println!("{}:{} -> localhost:{}", SERVER, server_port, local_port);
      task::spawn(keep_alive(write));
      loop {
        let id = read.read_u64().await?;
        task::spawn(handle(id, local_port));
      }
    }
    Some(Err(_)) => println!("invalid port"),
    None => println!("usage: {} (port)", args().next().unwrap()),
  };
  Ok(())
}

async fn handle(id: u64, local_port: u16) -> Result {
  let mut stream = TcpStream::connect((SERVER, SERVER_PORT)).await?;
  stream.write_u64(id).await?;
  io::copy_bidirectional(
    &mut stream,
    &mut TcpStream::connect(("0.0.0.0", local_port)).await?,
  )
  .await?;
  Ok(())
}

async fn keep_alive(mut server: OwnedWriteHalf) -> Result {
  loop {
    time::sleep(Duration::from_secs(10)).await;
    server.write_u8(0).await?;
  }
}
