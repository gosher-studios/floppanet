use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::{task, io, time};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use tracing::{info, debug, error};
use floppanet::{Result, SERVER_PORT, HANDSHAKE};

#[tokio::main]
async fn main() -> Result {
  tracing_subscriber::registry()
    .with(console_subscriber::spawn())
    .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::DEBUG))
    .init();
  let state = Arc::new(Mutex::new(State::default()));
  let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await?;
  info!("Started server on {}", SERVER_PORT);
  while let Ok((stream, _)) = listener.accept().await {
    let state = state.clone();
    task::spawn(async move {
      if let Err(e) = handle(state, stream).await {
        error!("{}", e);
      }
    });
  }
  Ok(())
}

async fn handle(state: Arc<Mutex<State>>, mut client: TcpStream) -> Result {
  match client.read_u128().await? {
    HANDSHAKE => {
      let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
      let port = listener.local_addr()?.port();
      let ip = client.peer_addr()?;
      info!("{} connected on {}", ip, port);
      client.write_u16(port).await?;
      let (mut read, mut write) = client.into_split();
      let listen = task::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
          let id = rand::random();
          state.lock().await.connections.insert(id, stream);
          write.write_u128(id).await.unwrap();
          write.flush().await.unwrap();
          task::spawn(delete(state.clone(), id));
        }
      });
      let _ = read.read_u8().await;
      info!("{} disconnected", ip);
      listen.abort();
    }
    id => {
      let conn = state.lock().await.connections.remove(&id);
      if let Some(mut conn) = conn {
        io::copy_bidirectional(&mut client, &mut conn).await?;
      }
    }
  }
  Ok(())
}

async fn delete(state: Arc<Mutex<State>>, id: u128) {
  time::sleep(Duration::from_secs(10)).await;
  if state.lock().await.connections.remove(&id).is_some() {
    debug!("Removed stale connection {}", id);
  }
}

#[derive(Default)]
struct State {
  connections: HashMap<u128, TcpStream>,
}
