curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

sudo apt install build-essential -y
cargo run -- 3004 http://158.160.49.228:3002
