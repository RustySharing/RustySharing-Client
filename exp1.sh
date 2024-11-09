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

    # Check if the client run was successful or failed
    if [[ $EXIT_STATUS -eq 0 && $OUTPUT == *"Extracted file saved to"* ]]; then
        RESULT="SUCCESS"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        RESULT="FAILURE"
        FAILURE_COUNT=$((FAILURE_COUNT + 1))
    fi

    # Save the result and elapsed time to the output file
    echo "Run: $i - $RESULT - Time taken: $ELAPSED_TIME seconds" >> "$OUTPUT_FILE"
}

# Run all clients in parallel and capture their results
for ((i = 1; i <= NUM_RUNS; i++)); do
    # Start each client in the background, capturing the output and timing
    (run_client) & # Parenthesis are used to group the command and run it in a subshell
done

# Wait for all background jobs to finish
wait

# Log final summary
echo "Total Successes: $SUCCESS_COUNT"
echo "Total Failures: $FAILURE_COUNT"
echo "Results saved to $OUTPUT_FILE"
