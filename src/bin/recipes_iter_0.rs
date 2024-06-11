use once_cell::sync::Lazy;
use libp2p::{
  core::upgrade,
  identity, 
//  Transport, 
  PeerId, 
  floodsub::{Floodsub, FloodsubEvent, Topic},
  swarm::{Swarm, NetworkBehaviour},
  mdns::tokio::Behaviour,
  tcp::tokio::Transport,
  noise
};
use tokio::{sync::mpsc};
use serde::Serialize;
use serde::Deserialize;
use log::{error, info};
use tracing_subscriber::EnvFilter;
// use libp2p_core::{Transport, upgrade, transport::MemoryTransport};

use std::error::Error;


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
  Floodsub(FloodsubEvent),
  Mdns(libp2p::mdns::Event)
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .try_init();

//    pretty_env_logger::init();

    info!("Peer Id: {}", PEER_ID.clone());
    println!("Peer Id: {}", PEER_ID.clone());
   
    let (response_sender, mut response_rcv) = 
      mpsc::unbounded_channel::<mpsc::UnboundedSender<ListResponse>>(); 
    let auth_keys = noise::Config::new(&KEYS).unwrap();

    let transp = Transport::new(libp2p::tcp::Config::default().nodelay(true));
//        .upgrade(upgrade::Version::V1)
//        .authenticate(noise::Config::xx(auth_keys).into_authenticated()) // XX Handshake pattern, IX exists as well and IK - only XX currently provides interop with other libp2p impls
//        .multiplex(mplex::MplexConfig::new())
//        .boxed();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key: &identity::Keypair| {

            let mdns = libp2p::mdns::tokio::Behaviour::new(
              libp2p::mdns::Config::default(), 
              key.public().to_peer_id())?;
            
            Ok(RecipeBehaviour {floodsub, mdns})
          
        })?
        .build();

  Ok(())
}
