rm -rf ~/music-hosting
sudo apt install git
git clone https://github.com/Dmitry909/music-hosting.git
cd music-hosting/tracks_service

sudo apt install -y postgresql postgresql-contrib postgresql-common postgresql-client
# sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh
sudo systemctl start postgresql
sudo systemctl enable postgresql
sudo -u postgres psql -c "CREATE DATABASE tracks_service;"
sudo -u postgres psql -d tracks_service -f migrations/0001_create_table.sql

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs < enter | sh -s -- -y
source $HOME/.cargo/env
cargo run -- 3002
