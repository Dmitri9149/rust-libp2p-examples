use once_cell::sync::Lazy;
use libp2p::{
  core::upgrade,
  identity, 
//  Transport, 
  PeerId, 
  floodsub::{Floodsub, FloodsubEvent, Topic},
  swarm::{Swarm, NetworkBehaviour, SwarmEvent},
  mdns::tokio::Behaviour,
  tcp::tokio::Transport,
  noise
};
use tokio::{sync::mpsc, io::AsyncBufReadExt, fs};
use serde::{Deserialize, Serialize};
use log::{error, info};
use tracing_subscriber::EnvFilter;
use libp2p::futures::StreamExt;
// use libp2p_core::{Transport, upgrade, transport::MemoryTransport};

use std::error::Error;
use std::collections::HashSet;


const STORAGE_FILE_PATH: &str = "./recipes.json";

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
type Recipes = Vec<Recipe>;

static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());

static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));

static TOPIC : Lazy<Topic> = Lazy::new(|| Topic::new("recipes"));

#[derive(Debug, Serialize, Deserialize)]
struct Recipe {
  id:usize,
  name: String, 
  ingredients: String,
  instructions: String,
  public: bool
}
#[derive(Debug, Serialize, Deserialize)]
enum ListMode {
  ALL,
  One(String)
}

#[derive(Debug, Serialize, Deserialize)]
struct ListRequest {
  mode: ListMode
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
  mode: ListMode,
  data: Recipes,
  receiver: String
}

enum EventType {
  Response(ListResponse),
  Input(String),
  FloodsubEvent(FloodsubEvent),
  MdnsEvent(libp2p::mdns::Event),
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "RecipeBehaviourEvent")]
struct RecipeBehaviour {
    floodsub: Floodsub,
    mdns: libp2p::mdns::tokio::Behaviour,
}

#[derive(Debug)]
enum RecipeBehaviourEvent {
  Floodsub(FloodsubEvent),
  Mdns(libp2p::mdns::Event),
}

impl From<FloodsubEvent> for RecipeBehaviourEvent {
  fn from(event: FloodsubEvent) -> Self {
      RecipeBehaviourEvent::Floodsub(event)
  }
}

impl From<libp2p::mdns::Event> for RecipeBehaviourEvent {
  fn from(event: libp2p::mdns::Event) -> Self {
      RecipeBehaviourEvent::Mdns(event)
  }
}

async fn read_local_recipes() -> Result <Recipes, Box<dyn Error>> {
  let content = fs::read(STORAGE_FILE_PATH).await?;
  let result = serde_json::from_slice(&content)?;
  Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .try_init();

//    pretty_env_logger::init();

    info!("Peer Id: {}", PEER_ID.clone());
    println!("Peer Id: {}", PEER_ID.clone());
   
    let (response_sender, mut response_rcv) = 
//      mpsc::unbounded_channel::<mpsc::UnboundedSender<ListResponse>>(); 
      mpsc::unbounded_channel();

    let auth_keys = noise::Config::new(&KEYS).unwrap();

//    let transp = Transport::new(libp2p::tcp::Config::default().nodelay(true));
//        .upgrade(upgrade::Version::V1)
//        .authenticate(noise::Config::xx(auth_keys).into_authenticated()) // XX Handshake pattern, IX exists as well and IK - only XX currently provides interop with other libp2p impls
//        .multiplex(mplex::MplexConfig::new())
//        .boxed();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default().nodelay(true),
            noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key: &identity::Keypair| {
            let mut floodsub = Floodsub::new(PEER_ID.clone());
            floodsub.subscribe(TOPIC.clone());

            let mdns = libp2p::mdns::tokio::Behaviour::new(
              libp2p::mdns::Config::default(), 
              key.public().to_peer_id())?;
            
            Ok(RecipeBehaviour {floodsub, mdns})
          
        })?
        .build();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

/* 
        Swarm::listen_on(
          &mut swarm,
          "/ip4/0.0.0.0/tcp/0"
              .parse()
              .expect("can get a local socket"),
      )
      .expect("swarm can be started");
*/

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
      let evt = {
          tokio::select! {
              line = stdin.next_line() => Some(EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
              response = response_rcv.recv() => Some(EventType::Response(response.expect("response exists"))),
              event = swarm.select_next_some() => {
                  match event {
                      SwarmEvent::Behaviour(RecipeBehaviourEvent::Floodsub(event)) => Some(EventType::FloodsubEvent(event)),
                      SwarmEvent::Behaviour(RecipeBehaviourEvent::Mdns(event)) => Some(EventType::MdnsEvent(event)),
                      _ => {
                          info!("Unhandled Swarm Event: {:?}", event);
                          None
                      }
                  }
              },
          }
      };

      if let Some(event) = evt {
        match event {
            EventType::Response(resp) => {
                let json = serde_json::to_string(&resp)
                  .expect("can jsonify response")
                  .as_bytes()
                  .to_vec();
                swarm
                    .behaviour_mut()
                    .floodsub
                    .publish(TOPIC.clone(), json);
            }
            EventType::Input(line) => match line.as_str() {
             "ls p" => handle_list_peers(&mut swarm).await,
//              cmd if cmd.starts_with("ls_r") => handle_list_recipes(cmd, &mut swarm).await,
//              cmd if cmd.starts_with("create r") => handle_create_recipe(cmd).await,
//              cmd if cmd.starts_with("publish r") => handle_publish_recipe(cmd).await,
              _ => error!("unknown command"),
            }
            _ => {}
        }

    }

//  Ok(())
  }
}

async fn handle_list_peers(swarm: &mut Swarm<RecipeBehaviour>) {
  info!("Discovered Peers: ");
  let nodes = swarm.behaviour().mdns.discovered_nodes();
  let mut unique_peers = HashSet::new();
  for peer in nodes {
    unique_peers.insert(peer);
  }
  unique_peers.iter().for_each(|p| info!("{}",p));
}

async fn handle_create_recipe(cmd: &str, swarm: &mut Swarm<RecipeBehaviour>) {
  let rest = cmd.strip_prefix("ls r");

  let from_json_to_publishing = |req: ListRequest| {
    let json = serde_json::to_string(&req).expect("can jsonify request");
    swarm
      .behaviour_mut()
      .floodsub
      .publish(TOPIC.clone(), json.as_bytes().to_vec()); 
  }; 
  match rest {
    Some("all") => {
      let req = ListRequest {
        mode: ListMode::ALL
      };
      let json = serde_json::to_string(&req).expect("can jsonify request");
      swarm
        .behaviour_mut()
        .floodsub
        .publish(TOPIC.clone(), json.as_bytes().to_vec());
    }
    Some(recipe_peer_id) => {
      let req = ListRequest {
        mode: ListMode::One(recipe_peer_id.to_owned())
      };
      let json = serde_json::to_string(&req).expect("can jsonify request");
      swarm
        .behaviour_mut()
        .floodsub
        .publish(TOPIC.clone(), json.as_bytes().to_vec());
    }
    None => {
      match read_local_recipes().await {
        Ok(v) => {
          info!("Local Recipes ({})", v.len());
          v.iter().for_each(|r| info!("{:?}", r));
        }
        Err(e) => error!("error fetching local recipes: {}", e),
      }
    }
  }
}

