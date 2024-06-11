use once_cell::sync::Lazy;
use libp2p::{
  core::upgrade,
  identity, 
  Transport, 
  PeerId, 
  floodsub::{Floodsub, FloodsubEvent, Topic},
  swarm::{Swarm, NetworkBehaviour},
  mdns::tokio::Behaviour,
  tcp,
//  noise::{Keypair, X25519Spec, NoiseConfig}
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

  Ok(())
}
