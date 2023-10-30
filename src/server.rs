use std::sync::Arc;
use std::time::Duration;
use std::collections::{HashSet, HashMap};
use tokio::{task, io, time};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use rand::Rng;
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
  match client.read_u64().await? {
    HANDSHAKE => {
      let mut s = state.lock().await;
      let port = rand_cond(
        || rand::thread_rng().gen_range(2000..4000),
        |p| !s.ports.contains(p),
      );
      s.ports.insert(port);
      drop(s);
      let ip = client.peer_addr()?;
      info!("{} connected on {}", ip, port);
      client.write_u16(port).await?;
      let (mut read, mut write) = client.into_split();
      let listen = task::spawn(async move {
        let listener = TcpListener::bind(("0.0.0.0", port)).await.unwrap();
        while let Ok((stream, _)) = listener.accept().await {
          let mut s = state.lock().await;
          let id = rand_cond(rand::random, |i| {
            *i != HANDSHAKE && !s.connections.contains_key(i)
          });
          s.connections.insert(id, stream);
          write.write_u64(id).await.unwrap();
          task::spawn(delete(state.clone(), id));
        }
      });
      let _ = read.read_u8().await;
      info!("{} disconnected", ip);
      listen.abort();
    }
    id => {
      if let Some(mut conn) = state.lock().await.connections.remove(&id) {
        io::copy_bidirectional(&mut client, &mut conn).await?;
      }
    }
  }
  Ok(())
}

async fn delete(state: Arc<Mutex<State>>, id: u64) {
  time::sleep(Duration::from_secs(10)).await;
  if state.lock().await.connections.remove(&id).is_some() {
    debug!("Removed stale connection {}", id);
  }
}

#[derive(Default)]
struct State {
  ports: HashSet<u16>,
  connections: HashMap<u64, TcpStream>,
}

fn rand_cond<T, G: Fn() -> T, C: Fn(&T) -> bool>(gen: G, cond: C) -> T {
  loop {
    let x = gen();
    if cond(&x) {
      break x;
    }
  }
}
