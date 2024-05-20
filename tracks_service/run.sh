sudo apt install -y postgresql postgresql-contrib postgresql-common postgresql-client
# sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh
sudo systemctl start postgresql
sudo systemctl enable postgresql
sudo -u postgres psql -c "CREATE DATABASE tracks_service;"
sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'qwerty';"
sudo -u postgres psql -d tracks_service -f migrations/0001_create_table.sql

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

sudo apt install build-essential -y
mkdir tracks
cargo run -- 3002
