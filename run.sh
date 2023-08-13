#! /bin/sh

# Remove old db.sqlite
rm -f dq.sqlite

# Create new db.sqlite
touch db.sqlite

# Run Programm
cargo run
