use std::env::args;
use tokio::task;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use floppanet::{Result, CLIENT_PORT, SERVER_PORT, SERVER, proxy};

#[tokio::main]
async fn main() -> Result {
  match args().nth(1).map(|a| a.parse()) {
    Some(Ok(local_port)) => {
      let mut server = TcpStream::connect((SERVER, SERVER_PORT)).await?;
      let server_port = server.read_u16().await?;
      println!("{}:{} -> localhost:{}", SERVER, server_port, local_port);
      let listener = TcpListener::bind(("0.0.0.0", CLIENT_PORT)).await?;
      while let Ok((stream, _)) = listener.accept().await {
        task::spawn(proxy(
          stream,
          TcpStream::connect(("0.0.0.0", local_port)).await?,
        ));
      }
    }
    Some(Err(_)) => println!("invalid port"),
    None => println!("usage: {} (port)", args().nth(0).unwrap()),
  };
  Ok(())
}
