// based on the: 
// examples from chapter 11 , Rust Servers, Services and App, Prabhu Eshwarla
// https://koonts.net/articles/rust-libp2p
// https://stackoverflow.com/questions/74126811/rust-libp2p-cannot-find-function-development-transport-in-crate-libp2p
// https://github.com/libp2p/rust-libp2p/blob/master/examples/chat/src/main.rs

// combination of Ping behavior with Peer Discovering

use libp2p::futures::StreamExt;
use libp2p::mdns;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, ping, Multiaddr, PeerId, SwarmBuilder};
use std::error::Error;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct PingAndDiscover {
    ping: ping::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let new_key: identity::Keypair = identity::Keypair::generate_ed25519();
    let new_peer_id: PeerId = PeerId::from(new_key.public());
    println!("New peer_id:   {new_peer_id:?}");

    let mut swarm = SwarmBuilder::with_existing_identity(new_key)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::tls::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key: &identity::Keypair| {
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            let ping = ping::Behaviour::default();
            Ok(PingAndDiscover { ping, mdns })
        })?
        .with_swarm_config(|cfg: libp2p::swarm::Config| {
            cfg.with_idle_connection_timeout(Duration::from_secs(50))
        })
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if let Some(remote_peer) = std::env::args().nth(1) {
        let remote_peer_multiaddr: Multiaddr = remote_peer.parse()?;
        swarm.dial(remote_peer_multiaddr)?;
        println!("Dialed remote peer: {:?}", remote_peer);
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on local address {:?}", address)
            }
            SwarmEvent::Behaviour(PingAndDiscoverEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discovered a new peer: {peer_id}");
                }
            }
            SwarmEvent::Behaviour(PingAndDiscoverEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discover peer has expired: {peer_id}");
                }
            }
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            _ => {}
        }
    }
}
