#!/bin/sh
set -e

echo "Waiting for the database to be available..."
until pg_isready --dbname=tracks_service -p 5432 -U postgres
do
  sleep 1
done
echo "Database is up and ready!"

cargo run -- 3002
