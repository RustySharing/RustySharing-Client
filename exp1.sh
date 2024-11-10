#!/bin/bash

# Number of iterations to run the client code
NUM_RUNS=100

# Initialize counters
SUCCESS_COUNT=0
FAILURE_COUNT=0

# Output files to store results
OUTPUT_FILE="client_results.txt"
RAW_OUTPUT_FILE="raw_output.txt"

# Clear the output files before starting
> "$OUTPUT_FILE"
> "$RAW_OUTPUT_FILE"

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

    # Save the result summary to the results file
    echo "Run: $i - $RESULT - Leader socket: $LEADER_SOCKET - Time taken: $ELAPSED_TIME seconds" >> "$OUTPUT_FILE"
    
    # Save the raw output with a label to the raw output file
    echo "========== Run $i ==========" >> "$RAW_OUTPUT_FILE"
    echo "$OUTPUT" >> "$RAW_OUTPUT_FILE"
    echo "============================" >> "$RAW_OUTPUT_FILE"
}

# Run all clients with a delay between each run
for ((i = 1; i <= NUM_RUNS; i++)); do
    # Run each client in the background
    run_client &
    
    # Delay (in seconds) between each client run
    sleep 2  # Adjust the delay as needed (2 seconds in this case)
done

# Wait for all background jobs to finish
wait

# Log final summary
echo "Total Successes: $SUCCESS_COUNT"
echo "Total Failures: $FAILURE_COUNT"
echo "Results saved to $OUTPUT_FILE"
echo "Full raw outputs saved to $RAW_OUTPUT_FILE"
