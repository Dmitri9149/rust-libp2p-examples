use once_cell::sync::Lazy;
use libp2p::{
  core::upgrade,
//  identity, 
  Transport, 
  PeerId, 
  floodsub::{Floodsub, FloodsubEvent, Topic},
  swarm::{NetworkBehaviour},
  mdns::tokio::Behaviour,
  tcp,
//  noise::{Keypair, X25519Spec, NoiseConfig}
};
use libp2p_mplex;
use tokio::{sync::mpsc};
use serde::Serialize;
use serde::Deserialize;
use log::{error, info};
use libp2p_noise as noise;
use libp2p_identity as identity;
// use libp2p_core::{Transport, upgrade, transport::MemoryTransport};

use std::fmt::Error;


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
  Input(String)
}

#[derive(NetworkBehaviour)]
struct RecipeBehaviour {
    floodsub: Floodsub,
    mdns: Behaviour,
//    #[behaviour(ignore)]
//    response_sender: mpsc::UnboundedSender<ListResponse>,
}
const STORAGE_FILE_PATH: &str = "./recipes.json";
static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("recipes"));