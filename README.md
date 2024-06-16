I study (Rust) peer-to-peer network library [libp2p](https://github.com/libp2p/rust-libp2p).    
"libp2p is an open source project for building network applications free from runtime and address services."    
In the project there are several examples of using libp2p.  

#### Recipes 
Starting point for the code in my project is very interesting blog post [libp2p tutorial: Build a peer-to-peer app in Rust](https://blog.logrocket.com/libp2p-tutorial-build-a-peer-to-peer-app-in-rust/) which use libp2p      
version = "0.31"  ( see on [GitHub](https://github.com/zupzup/rust-peer-to-peer-example/tree/main)    )  
There were a lot of breaking changes in the libp2p from v0.31 to the current ( at the moment of the writing) v0.53.2.   
It was need to rewrite some parts of the original code to make it working with current version of libp2p. 
This happens to be not an easy but very good exercise with libp2p and networking in general for me.   
In the work I use the [p2p-play](https://github.com/bhagdave/p2p-play/tree/main) project, where the original code was adapted to the v0.49    
Even there are a lot of breaking changes from libp2p v0.49 to v0.53 too, the [p2p-play](https://github.com/bhagdave/p2p-play/tree/main) project essentially simplifies the job.   
See the full description of how all it works at [libp2p tutorial: Build a peer-to-peer app in Rust](https://blog.logrocket.com/libp2p-tutorial-build-a-peer-to-peer-app-in-rust/) 



+ `recipes.rs ` - list of locally stored recipes

To run the code from the root of my project:     
`RUST_LOG=info cargo run --bin recipes_iter_0`

+ `ls p` - list local peers
+ `ls r` - list local recipes
+ `ls r - all` list of all public recipes from all known peers
+ `ls r {peerId}` - list all public recipes from the given peer
+ `create r Name|Ingredients|Instructions` - create a new recipe with the given data
+ `publish r {recipeId}` - publish recipe with the given recipe ID

#### Ping And Discover
It is combination of Ping behavior and Discover behavior.   
Running:   
`cargo run --bin ping_and_discover` we will get something like this:    
```
New peer_id:   PeerId("12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL")
Listening on local address "/ip4/127.0.0.1/tcp/34983"
Listening on local address "/ip4/10.255.255.254/tcp/34983"
Listening on local address "/ip4/172.29.92.103/tcp/34983"
```   
Running same command from second terminal starts peers discovery :   
```
New peer_id:   PeerId("12D3KooWKwnjE8sFBqzoeHoxqDXJAkwNzXKtADYKcPCeLYdWkhdD")
Listening on local address "/ip4/127.0.0.1/tcp/37021"
Listening on local address "/ip4/10.255.255.254/tcp/37021"
Listening on local address "/ip4/172.29.92.103/tcp/37021"
mDNS discovered a new peer: 12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL
mDNS discovered a new peer: 12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL
```

If Running from the second terminal :  
`cargo run --bin ping_and_discover /ip4/127.0.0.1/tcp/34983`   
will start Ping behavior between two peers:    

```
New peer_id:   PeerId("12D3KooWH5XRVNQYguFY8tayDzzMNC9am54LrPdzJ3AaK6TbB6wA")
Dialed remote peer: "/ip4/127.0.0.1/tcp/34983"
Listening on local address "/ip4/127.0.0.1/tcp/42357"
Listening on local address "/ip4/10.255.255.254/tcp/42357"
Listening on local address "/ip4/172.29.92.103/tcp/42357"
mDNS discovered a new peer: 12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL
mDNS discovered a new peer: 12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL
PingAndDiscoverEvent: Event { peer: PeerId("12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL"), connection: ConnectionId(1), result: Ok(99.492958ms) }
PingAndDiscoverEvent: Event { peer: PeerId("12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL"), connection: ConnectionId(1), result: Ok(981.919µs) }
```    

If we stop first peer running, after some time we will get from the second peer:    
.................    

```  
PingAndDiscoverEvent: Event { peer: PeerId("12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL"), connection: ConnectionId(1), result: Ok(1.41176ms) }
mDNS discover peer has expired: 12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL
mDNS discover peer has expired: 12D3KooWNyyAPKSB6qMPEqb9oKqjKPMZrfkPecJmzimjygPVAhHL
```   
As we can see the second peer discovered the expiration of the first peer. 









