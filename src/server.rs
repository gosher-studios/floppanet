use std::sync::Arc;
use std::time::Duration;
use std::collections::{HashSet, HashMap};
use tokio::{task, io, time};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use rand::Rng;
use floppanet::{Result, SERVER_PORT, HANDSHAKE};

#[tokio::main]
async fn main() -> Result {
  let state = Arc::new(Mutex::new(State::default()));
  let listener = TcpListener::bind(("0.0.0.0", SERVER_PORT)).await?;
  while let Ok((stream, _)) = listener.accept().await {
    let state = state.clone();
    task::spawn(async move {
      if let Err(e) = handle(state, stream).await {
        println!("err: {}", e);
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
      println!("{} connected on {}", client.peer_addr()?, port);
      client.write_u16(port).await?;
      let listener = TcpListener::bind(("0.0.0.0", port)).await?;
      loop {
        tokio::select! {
          Ok((stream, _)) = listener.accept() => {
            let mut s = state.lock().await;
            let id = rand_cond(rand::random, |i| {
              *i != HANDSHAKE && !s.connections.contains_key(i)
            });
            s.connections.insert(id, stream);
            client.flush().await?;
            client.write_u64(id).await?;
            task::spawn(delete(state.clone(), id));
          },
          _ = time::sleep(Duration::from_secs(15)) => {
            match client.read_to_end(&mut vec![]).await {
              Ok(0) | Err(_) => {
                println!("{} disconnected",client.peer_addr()?);
                state.lock().await.ports.remove(&port);
                break;
              },
              _=> {}
            }
          }
        }
      }
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
  state.lock().await.connections.remove(&id);
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
