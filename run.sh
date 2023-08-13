#! /bin/sh

# Remove old db.sqlite
rm -f db.sqlite

# Create new db.sqlite
touch db.sqlite

# Run Programm
cargo run
