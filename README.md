# LukChain

Generalised BlockChain

Currently uses `nightly` features

## Config

To use `LukChain`, a `config.toml` file needs to be setup to define different user profiles.
This should follow the following structure:

```
[config]
[[profiles]]
block_size = 10
lookup_address = "127.0.0.1:8181"
lookup_filter = "user"
user_location = "user1.json"
bc_location = "blockchain1.bin"

[[profiles]]
block_size = 20
lookup_address = "127.0.0.1:8181"
lookup_filter = "miner"
user_location = "user2.json"
bc_location = "blockchain2.bin"
```

Each field can be ommitted if not needed. The use of each field in the program is:

- `block_size`: If the role chosen is `Miner`, this option will define the size of blocks that it will add to the blockchain and will distribute to connected nodes.
- `lookup_address`: Address to contact initially to obtain addresses of other nodes in the network to connect to.
- `lookup_filter`: Included in message to LookUp and allows filtering of address book members based on their role in the network.
- `user_location`: Location where information about user is stored when it is generated or needs to be retrieved on startup
- `bc_location`: Location of binary file containing local copy of blockchain

## Examples

### Chat

To run with cargo: `cargo run --example chat -- --role lookup `

```
Chat
USAGE:
  chat [OPTIONS] --log LEVEL [INPUT]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --log LEVEL          Sets logging level
  --role ROLE          Sets the role of the user in the network
  --config NUMBER      Sets chosen config (default: 0)
ARGS:
  <INPUT>
```
