# Blockchat

Blockchain based chat application

## Command Line Interface

```
BlockChat
USAGE:
  blochat [OPTIONS] --log LEVEL [INPUT]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --log LEVEL          Sets logging level
  --role ROLE          Sets the role of the user in the network
  --config NUMBER      Sets chosen config (default: 0)
ARGS:
  <INPUT>
```

## Config

To use `BlockChat`, a `config.toml` file needs to be setup to define different user profiles.
This should follow the following structure:

```
[config]
[[profiles]]
pub_key = ".."
priv_key = ".."
block_size = 10


[[profiles]]
pub_key = ".."
priv_key = ".."
block_size = 20
```

Each field can be ommitted if not needed. The use of each field in the program is:

- `pub_key`: RSA Public Key. If public key is defined, must also define a private key.
- `priv_key`: RSA Private Key. Like public key, private key also needs an public key to be used.
- `block_size`: If the role chosen is `Miner`, this option will define the size of blocks that it will add to the blockchain and will distribute to connected nodes.
