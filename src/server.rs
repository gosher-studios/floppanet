use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use tokio::task;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use rand::Rng;
use floppanet::{Result, SERVER_PORT, proxy};

#[tokio::main]
async fn main() -> Result {
  let ports = Arc::new(Mutex::new(HashSet::new()));
  let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await?;
  while let Ok((stream, _)) = listener.accept().await {
    task::spawn(handle(ports.clone(), stream));
  }
  Ok(())
}

async fn handle(ports: Arc<Mutex<HashSet<u16>>>, mut client: TcpStream) -> Result {
  let port = loop {
    let port = rand::thread_rng().gen_range(2000..4000);
    if !ports.lock().unwrap().contains(&port) {
      break port;
    }
  };
  ports.lock().unwrap().insert(port);
  client.write_u16(port).await?;
  let listener = TcpListener::bind(("0.0.0.0", port)).await?;
  while let Ok((stream, _)) = listener.accept().await {
    proxy(&mut client, stream).await?;
  }
  // todo unconnect people
  Ok(())
}
