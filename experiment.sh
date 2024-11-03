#!/bin/bash

# Define the command to run your Rust application
COMMAND="cargo run --release"

# Open three terminals and run the command in each
gnome-terminal -- bash -c "$COMMAND; exec bash"
gnome-terminal -- bash -c "$COMMAND; exec bash"
gnome-terminal -- bash -c "$COMMAND; exec bash"
