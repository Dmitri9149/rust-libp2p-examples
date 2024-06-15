I study (Rust) peer-to-peer network library [libp2p](https://github.com/libp2p/rust-libp2p).    
"libp2p is an open source project for building network applications free from runtime and address services."   
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






