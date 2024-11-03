#!/bin/bash

# Compile the Rust code if needed
cargo build --release

# Define the path to the compiled executable
EXECUTABLE="./target/release/rudp_client"

# Run three unique instances in parallel
$EXECUTABLE "instance1" "/home/bavly.remon2004@auc.egy/Downloads/png_images/png_test/instace1" &
pid1=$!

$EXECUTABLE "instance2" "/home/bavly.remon2004@auc.egy/Downloads/png_images/png_test/instace2" &
pid2=$!

$EXECUTABLE "instance3" "/home/bavly.remon2004@auc.egy/Downloads/png_images/png_test/instace3" &
pid3=$!

# Wait for all instances to complete
wait $pid1
wait $pid2
wait $pid3

echo "All instances have completed."
