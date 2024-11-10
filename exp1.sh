#!/bin/bash

# Number of iterations to run the client code
NUM_RUNS=10

# Initialize counters
SUCCESS_COUNT=0
FAILURE_COUNT=0

# Output file to store results
OUTPUT_FILE="client_results.txt"
> "$OUTPUT_FILE"  # Clear the file before starting

# Function to run the client and capture success/failure and timing
run_client() {
    # Record the start time
    START_TIME=$(date +%s.%N)

    # Run the Rust client code and capture the output
    OUTPUT=$(cargo run 2>&1)
    EXIT_STATUS=$?

    # Record the end time
    END_TIME=$(date +%s.%N)

    # Calculate the elapsed time (time taken to run the client)
    ELAPSED_TIME=$(echo "$END_TIME - $START_TIME" | bc)

    # Extract the leader socket information from the output
    LEADER_SOCKET=$(echo "$OUTPUT" | grep -oP 'Leader socket: \K\S+')

    # Check if the client run was successful or failed
    if [[ $EXIT_STATUS -eq 0 && $OUTPUT == *"Extracted file saved to"* ]]; then
        RESULT="SUCCESS"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        RESULT="FAILURE"
        FAILURE_COUNT=$((FAILURE_COUNT + 1))
    fi

    # Save the result, leader socket, and elapsed time to the output file
    echo "Run: $i - $RESULT - Leader socket: $LEADER_SOCKET - Time taken: $ELAPSED_TIME seconds" >> "$OUTPUT_FILE"
}

# Run all clients with a delay between each run
for ((i = 1; i <= NUM_RUNS; i++)); do
    # Run each client in the background, but add a delay before starting the next one
    run_client &
    
    # Delay (in seconds) between each client run (adjust the time as needed)
    sleep 3  # This will add a 2-second delay between starting each client
done

# Wait for all background jobs to finish
wait

# Log final summary
echo "Total Successes: $SUCCESS_COUNT"
echo "Total Failures: $FAILURE_COUNT"
echo "Results saved to $OUTPUT_FILE"
