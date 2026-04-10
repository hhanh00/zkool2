#!/usr/bin/env bash
# Common functions and setup for FROST smoke tests
# Source this file in smoke-dkg.sh and smoke-frost.sh

# Configuration
N=3                     # Number of participants
T=3                      # Threshold (all 3 must participate)
DEFAULT_PORT=8000        # Port for default GraphQL instance
PORT_BASE=8001            # Starting port for GraphQL instances
LWD_URL="http://localhost:8137"  # LWD URL for all participants

# Path to zkool_graphql binary (relative to script location in example/sh/)
ZKOOL_GRAPHQL="$(dirname "${BASH_SOURCE[0]}")/../../target/release/zkool_graphql"

# Cleanup function - takes a database prefix as argument
cleanup() {
    local db_prefix=$1
    echo "Cleaning up..."

    # Clean up default instance
    if [ -n "${PID_DEFAULT:-}" ]; then
        kill "$PID_DEFAULT" 2>/dev/null || true
    fi
    rm -rf "/tmp/${db_prefix}_default.db" 2>/dev/null || true
    rm -f "/tmp/graphql_${db_prefix}_default.log" 2>/dev/null || true

    # Clean up participant instances
    for i in $(seq 1 $N); do
        eval "pid=\${PID_${i:-}"
        if [ -n "$pid" ]; then
          kill "$pid" 2>/dev/null || true
        fi
        rm -rf "/tmp/${db_prefix}_test_${i}.db" 2>/dev/null || true
        rm -f "/tmp/graphql_${db_prefix}_${i}.log" 2>/dev/null || true
    done
}

# Start all GraphQL instances - takes database prefix as argument
start_instances() {
    local db_prefix=$1

    pkill zkool_graphql 2>/dev/null || true
    sleep 2

    echo "=== Setting up FROST Test ==="

    # Start default zkool_graphql instance
    DEFAULT_DB="/tmp/${db_prefix}_default.db"
    echo "Starting default instance on port $DEFAULT_PORT with database $DEFAULT_DB"
    rm -f "$DEFAULT_DB"

    "$ZKOOL_GRAPHQL" \
        -d "$DEFAULT_DB" \
        -p $DEFAULT_PORT \
        -l "$LWD_URL" \
        > "/tmp/graphql_${db_prefix}_default.log" 2>&1 &

    PID_DEFAULT=$!
    DEFAULT_URL="http://localhost:$DEFAULT_PORT/graphql"
    echo "Waiting for default instance to start..."
    sleep 2

    # Start 3 separate zkool_graphql instances with distinct databases
    GRAPHQL_URLS=()
    for i in $(seq 1 $N); do
        PORT=$((PORT_BASE + i - 1))
        DB_PATH="/tmp/${db_prefix}_test_${i}.db"

        echo "Starting participant $i on port $PORT with database $DB_PATH"
        rm -f "$DB_PATH"

        # Start zkool_graphql in background
        "$ZKOOL_GRAPHQL" \
            -d "$DB_PATH" \
            -p $PORT \
            -l "$LWD_URL" \
            > "/tmp/graphql_${db_prefix}_${i}.log" 2>&1 &

        # Save PID for cleanup
        eval "PID_${i}=\$!"
        GRAPHQL_URLS+=("http://localhost:$PORT/graphql")

        # Wait for server to start
        echo "Waiting for participant $i to start..."
        sleep 2
    done

    echo "All participants started"
}

# Function to execute gq and print full response on error
# Usage: gq_check <url> <args...>
# Returns the output on success, prints error and returns non-zero on failure
gq_check() {
    local output
    local tmpfile=$(mktemp)
    output=$(gq "$@" 2>&1 | tee "$tmpfile")
    local exit_code=$?

    if [ $exit_code -ne 0 ]; then
        echo "ERROR: gq command failed with exit code $exit_code" >&2
        echo "Command: gq $*" >&2
        echo "Full server response:" >&2
        cat "$tmpfile" >&2
        rm -f "$tmpfile"
        return 1
    fi

    if echo "$output" | grep -q '"error"'; then
        echo "ERROR: Server returned an error response" >&2
        echo "Command: gq $*" >&2
        echo "Full server response:" >&2
        cat "$tmpfile" >&2
        rm -f "$tmpfile"
        return 1
    fi

    rm -f "$tmpfile"
    echo "$output"
    return 0
}

# Function to get current height
height() {
    gq "http://localhost:$PORT_BASE/graphql" -q '
    query h {
      currentHeight
    }' | jq -r '.data.currentHeight'
}

# Wait for blockchain to reach a specific height
wait_for_height() {
    local target=$(($1 + $2))
    while (( $(height) < $target )); do
        sleep 1
    done
}
