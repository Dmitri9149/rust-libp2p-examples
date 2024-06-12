use libp2p::futures::StreamExt;
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::tokio::Behaviour,
    noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp::tokio::Transport,
    PeerId,
};
use tokio::{fs, io::AsyncBufReadExt, sync::mpsc};

use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

use std::collections::HashSet;
use std::error::Error;

const STORAGE_FILE_PATH: &str = "./recipes.json";

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
type Recipes = Vec<Recipe>;

static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());

static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));

static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("recipes"));

#[derive(Debug, Serialize, Deserialize)]
struct Recipe {
    id: usize,
    name: String,
    ingredients: String,
    instructions: String,
    public: bool,
}
#[derive(Debug, Serialize, Deserialize)]
enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ListRequest {
    mode: ListMode,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    mode: ListMode,
    data: Recipes,
    receiver: String,
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

async fn read_local_recipes() -> Result<Recipes, Box<dyn Error>> {
    let content = fs::read(STORAGE_FILE_PATH).await?;
    let result = serde_json::from_slice(&content)?;
    Ok(result)
}

async fn create_new_recipe(
    name: &str,
    ingredients: &str,
    instructions: &str,
) -> Result<(), Box<dyn Error>> {
    let mut local_recipes = read_local_recipes().await?;
    let new_id = match local_recipes.iter().max_by_key(|r| r.id) {
        Some(v) => v.id + 1,
        None => 0,
    };
    local_recipes.push(Recipe {
        id: new_id,
        name: name.to_owned(),
        ingredients: ingredients.to_owned(),
        instructions: instructions.to_owned(),
        public: false,
    });
    write_local_recipes(&local_recipes).await?;

    info!("Created story:");
    info!("Name: {}", name);
    info!("Ingredients: {}", ingredients);
    info!("Instructions:: {}", instructions);

    Ok(())
}

fn respond_with_public_recipes(sender: mpsc::UnboundedSender<ListResponse>, receiver: String) {
    tokio::spawn(async move {
        match read_local_recipes().await {
            Ok(recipes) => {
                let resp = ListResponse {
                    mode: ListMode::ALL,
                    receiver,
                    data: recipes.into_iter().filter(|r| r.public).collect(),
                };
                if let Err(e) = sender.send(resp) {
                    error!("error sending response via channel, {}", e);
                }
            }
            Err(e) => error!("error fetching local stories to answer ALL request, {}", e),
        }
    });
}

async fn write_local_recipes(recipes: &Recipes) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string(&recipes)?;
    fs::write(STORAGE_FILE_PATH, &json).await?;
    Ok(())
}

async fn publish_recipe(id: usize) -> Result<(), Box<dyn Error>> {
    let mut local_recipes = read_local_recipes().await?;
    local_recipes
        .iter_mut()
        .filter(|r| r.id == id)
        .for_each(|r| r.public = true);
    write_local_recipes(&local_recipes).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    //    pretty_env_logger::init();

    info!("Peer Id: {}", PEER_ID.clone());
    println!("Peer Id: {}", PEER_ID.clone());

    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();

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
                key.public().to_peer_id(),
            )?;

            Ok(RecipeBehaviour { floodsub, mdns })
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
                    swarm.behaviour_mut().floodsub.publish(TOPIC.clone(), json);
                }
                EventType::Input(line) => match line.as_str() {
                    "ls p" => handle_list_peers(&mut swarm).await,
                    cmd if cmd.starts_with("ls_r") => handle_list_recipes(cmd, &mut swarm).await,
                    cmd if cmd.starts_with("create r") => handle_create_recipes(cmd).await,
                    cmd if cmd.starts_with("publish r") => handle_publish_recipe(cmd).await,
                    _ => error!("unknown command"),
                },
                EventType::MdnsEvent(mdns_event) => match mdns_event {
                    libp2p::mdns::Event::Discovered(discovered_list) => {
                        for (peer, _addr) in discovered_list {
                            info!("Disocvered a peer:{} at {}", peer, _addr);
                            swarm
                                .behaviour_mut()
                                .floodsub
                                .add_node_to_partial_view(peer);
                        }
                    }
                    libp2p::mdns::Event::Expired(expired_list) => {
                        for (peer, _addr) in expired_list {
                            info!("Expired a peer:{} at {}", peer, _addr);
                            if !swarm
                                .behaviour_mut()
                                .mdns
                                .discovered_nodes()
                                .any(|p| *p == peer)
                            {
                                swarm
                                    .behaviour_mut()
                                    .floodsub
                                    .remove_node_from_partial_view(&peer);
                            }
                        }
                    }
                },
                EventType::FloodsubEvent(floodsub_event) => match floodsub_event {
                    FloodsubEvent::Message(msg) => {
                        if let Ok(resp) = serde_json::from_slice::<ListResponse>(&msg.data) {
                            if resp.receiver == PEER_ID.to_string() {
                                info!("Response from {}:", msg.source);
                                resp.data.iter().for_each(|r| info!("{:?}", r));
                            }
                        } else if let Ok(req) = serde_json::from_slice::<ListRequest>(&msg.data) {
                            match req.mode {
                                ListMode::ALL => {
                                    info!("Received ALL req: {:?} from {:?}", req, msg.source);
                                    respond_with_public_recipes(
                                        response_sender.clone(),
                                        msg.source.to_string(),
                                    )
                                }
                                ListMode::One(ref peer_id) => {
                                    if peer_id == &PEER_ID.to_string() {
                                        info!("Received req: {:?} from {:?}", req, msg.source);
                                        respond_with_public_recipes(
                                            response_sender.clone(),
                                            msg.source.to_string(),
                                        )
                                    }
                                }
                            }
                        }
                    }
                    _ => info!("Subscription events"), 
                },
            }
        }
    }
}

async fn handle_list_peers(swarm: &mut Swarm<RecipeBehaviour>) {
    info!("Discovered Peers: ");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().for_each(|p| info!("{}", p));
}

async fn handle_list_recipes(cmd: &str, swarm: &mut Swarm<RecipeBehaviour>) {
    let rest = cmd.strip_prefix("ls r");

    let mut publish = |req: ListRequest| {
        let json = serde_json::to_string(&req).expect("can jsonify request");
        swarm
            .behaviour_mut()
            .floodsub
            .publish(TOPIC.clone(), json.as_bytes().to_vec());
    };
    match rest {
        Some("all") => {
            let req = ListRequest {
                mode: ListMode::ALL,
            };
            publish(req);
/* 
            let json = serde_json::to_string(&req).expect("can jsonify request");
            swarm
                .behaviour_mut()
                .floodsub
                .publish(TOPIC.clone(), json.as_bytes().to_vec());
*/
        }
        Some(recipe_peer_id) => {
            let req = ListRequest {
                mode: ListMode::One(recipe_peer_id.to_owned()),
            };
            publish(req);
/* 
            let json = serde_json::to_string(&req).expect("can jsonify request");
            swarm
                .behaviour_mut()
                .floodsub
                .publish(TOPIC.clone(), json.as_bytes().to_vec());
*/
        }
        None => match read_local_recipes().await {
            Ok(v) => {
                info!("Local Recipes ({})", v.len());
                v.iter().for_each(|r| info!("{:?}", r));
            }
            Err(e) => error!("error fetching local recipes: {}", e),
        },
    }
}

async fn handle_create_recipes(cmd: &str) {
    if let Some(rest) = cmd.strip_prefix("create s") {
        let elements: Vec<&str> = rest.split('|').collect();
        if elements.len() < 3 {
            info!("too few arguments: Format name|header|body");
        } else {
            let name = elements.get(0).expect("name is there");
            let header = elements.get(1).expect("header is there");
            let body = elements.get(2).expect("body is there");
            if let Err(e) = create_new_recipe(name, header, body).await {
                error!("error creating story: {}", e);
            };
        }
    }
}

async fn handle_publish_recipe(cmd: &str) {
    if let Some(rest) = cmd.strip_prefix("publish r") {
        match rest.trim().parse::<usize>() {
            Ok(id) => {
                if let Err(e) = publish_recipe(id).await {
                    info!("error publishing story with id {}, {}", id, e)
                } else {
                    info!("Published story with id: {}", id);
                }
            }
            Err(e) => error!("invalid id: {}, {}", rest.trim(), e),
        }
    }
}
