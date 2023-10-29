use std::sync::Arc;
use std::collections::{HashSet, HashMap};
use tokio::task;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use rand::Rng;
use floppanet::{Result, proxy, SERVER_PORT, HANDSHAKE};

#[tokio::main]
async fn main() -> Result {
  let state = Arc::new(Mutex::new(State::default()));
  let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await?;
  while let Ok((stream, _)) = listener.accept().await {
    task::spawn(handle(state.clone(), stream));
  }
  Ok(())
}

async fn handle(state: Arc<Mutex<State>>, mut client: TcpStream) -> Result {
  match client.read_u64().await? {
    HANDSHAKE => {
      let mut ports = state.lock().await;
      let port = loop {
        let port = rand::thread_rng().gen_range(2000..4000);
        if !ports.ports.contains(&port) {
          break port;
        }
      };
      ports.ports.insert(port);
      drop(ports);
      client.write_u16(port).await?;
      let listener = TcpListener::bind(("0.0.0.0", port)).await?;
      while let Ok((stream, _)) = listener.accept().await {
        let id = rand::random(); // dont pick handshake as an id
        state.lock().await.connections.insert(id, stream);
        client.write_u64(id).await?;
      }
      // todo unconnect people
    }
    id => {
      proxy(
        client,
        state.lock().await.connections.remove(&id).unwrap(), // shouldnt unwrap
      )
      .await?;
    }
  }
  Ok(())
}

#[derive(Default)]
struct State {
  ports: HashSet<u16>,
  connections: HashMap<u64, TcpStream>,
}
