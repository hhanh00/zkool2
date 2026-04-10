#!/usr/bin/env bash
set -euo pipefail

# Test for 3-out-of-3 FROST DKG using GraphQL API
# This test sets up 3 separate zkool_graphql instances with distinct databases

# Configuration
N=3                     # Number of participants
T=3                      # Threshold (all 3 must participate)
DEFAULT_PORT=8000        # Port for default GraphQL instance
PORT_BASE=8001            # Starting port for GraphQL instances
LWD_URL="http://localhost:8137"  # LWD URL for all participants

# Path to zkool_graphql binary (relative to script location in example/sh/)
ZKOOL_GRAPHQL="$(dirname "$0")/../../target/release/zkool_graphql"

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    # Clean up default instance
    if [ -n "$PID_DEFAULT" ]; then
        kill "$PID_DEFAULT" 2>/dev/null || true
    fi
    rm -rf "/tmp/regtest_dkg_default.db" 2>/dev/null || true
    rm -f "/tmp/graphql_default.log" 2>/dev/null || true

    # Clean up participant instances
    for i in $(seq 1 $N); do
        eval "pid=\$PID_${i}"
        if [ -n "$pid" ]; then
          kill "$pid" 2>/dev/null || true
        fi
        rm -f "/tmp/graphql_${i}.log" 2>/dev/null || true
    done
}

# Set trap for cleanup
# If SKIP_CLEANUP is set, don't clean up databases (for chaining with smoke-frost)
if [ -z "${SKIP_CLEANUP:-}" ]; then
    trap cleanup EXIT INT TERM
fi

pkill zkool_graphql 2>/dev/null || true
rm -rf "/tmp/regtest_dkg_test_*.db" 2>/dev/null || true
sleep 2
if [ -z "${SKIP_CLEANUP:-}" ]; then
    rm -rf /tmp/regtest_dkg_*.db 2>/dev/null
fi
rm -f /tmp/graphql_*.log 2>/dev/null || true

echo "=== Setting up 3-out-of-3 FROST DKG Test ==="

# Start default zkool_graphql instance
DEFAULT_DB="/tmp/regtest_dkg_default.db"
echo "Starting default instance on port $DEFAULT_PORT with database $DEFAULT_DB"
rm -f "$DEFAULT_DB"

"$ZKOOL_GRAPHQL" \
    -d "$DEFAULT_DB" \
    -p $DEFAULT_PORT \
    -l http://localhost:8137 \
    > "/tmp/graphql_default.log" 2>&1 &

PID_DEFAULT=$!
DEFAULT_URL="http://localhost:$DEFAULT_PORT/graphql"
echo "Waiting for default instance to start..."
sleep 2

# Start 3 separate zkool_graphql instances with distinct databases
GRAPHQL_URLS=()
for i in $(seq 1 $N); do
    PORT=$((PORT_BASE + i - 1))
    DB_PATH="/tmp/regtest_dkg_test_${i}.db"

    echo "Starting participant $i on port $PORT with database $DB_PATH"
    rm -f "$DB_PATH"

    # Start zkool_graphql in background
    "$ZKOOL_GRAPHQL" \
        -d "$DB_PATH" \
        -p $PORT \
        -l http://localhost:8137 \
        > "/tmp/graphql_${i}.log" 2>&1 &

    # Save PID for cleanup
    eval "PID_${i}=\$!"
    GRAPHQL_URLS+=("http://localhost:$PORT/graphql")

    # Wait for server to start
    echo "Waiting for participant $i to start..."
    sleep 2
done

echo "All participants started"

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

wait_for_height() {
    local target=$(($1 + $2))
    while (( $(height) < $target )); do
        sleep 1
    done
}

echo ""
echo "=== Step 1: Create funded wallet ==="
MAIN_WALLET=$(gq "$DEFAULT_URL" \
  -q 'mutation ($main: String!) {
  createAccount(newAccount: {
    name: "Main"
    key: $main
    aindex: 0
    useInternal: false
    birth: 1
  })
}' -v "main=$SEED" | jq -r '.data.createAccount' | tr -d '[:space:]')

echo "Funding wallet: $MAIN_WALLET"

# Synchronize wallet
gq "$DEFAULT_URL" \
  -q 'mutation ($account: Int!) {
    synchronizeAccount(idAccount: $account)
  }' -v "account=$MAIN_WALLET" > /dev/null

# Get funding wallet balance
FUNDING_BALANCE=$(gq "$DEFAULT_URL" \
  -q 'query ($account: Int!) {
    balanceByAccount(idAccount: $account) {
      orchard
    }
  }' \
  -v "account=$MAIN_WALLET" | jq -r '.data.balanceByAccount.orchard' | tr -d '[:space:]')

echo "Funding wallet balance: $FUNDING_BALANCE"

echo ""
echo "=== Step 2: Initialize DKG for each participant ==="

# Initialize DKG parameters for each participant
DKG_ADDRESSES=()
FUNDING_ADDRESSES=()
FUNDING_ACCOUNTS=()  # Store funding account IDs
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    # Create funding account for DKG
    FUNDING_ACCOUNT=$(gq "$GRAPHQL_URL" \
      -q 'mutation {
      createAccount(newAccount: {
        name: "DKG-Fund"
        key: ""
        aindex: 0
        useInternal: false
        birth: 1
      })
    }' | jq -r '.data.createAccount' | tr -d '[:space:]')

    # Store the funding account ID for later use
    FUNDING_ACCOUNTS+=("$FUNDING_ACCOUNT")

    # Retrieve funding wallet orchard address
    FUNDING_ADDRESS=$(gq "$GRAPHQL_URL" \
      -q 'query ($account: Int!) {
        addressByAccount(idAccount: $account) {
          orchard
        }
      }' \
      -v "account=$FUNDING_ACCOUNT" | jq -r '.data.addressByAccount.orchard' | tr -d '[:space:]')

    echo "Participant $i funding account: $FUNDING_ACCOUNT"
    echo "Participant $i funding address: $FUNDING_ADDRESS"
    FUNDING_ADDRESSES+=("$FUNDING_ADDRESS")

    # Initialize DKG with participant ID
    DKG_ADDRESS=$(gq "$GRAPHQL_URL" \
      -q 'mutation ($name: String!, $t: Int!, $n: Int!, $funding: Int!, $id: Int!) {
      dkgStart(
        name: $name
        threshold: $t
        participants: $n
        messageAccount: $funding
        idParticipant: $id
      )
    }' \
      -v "name=Dkg-Test-$i" \
      -v "t=$T" \
      -v "n=$N" \
      -v "funding=$FUNDING_ACCOUNT" \
      -v "id=$i" | jq -r '.data.dkgStart' | tr -d '[:space:]')

    echo "Participant $i DKG address: $DKG_ADDRESS"
    DKG_ADDRESSES+=("$DKG_ADDRESS")
done

echo ""
echo "=== Step 3: Wait for main wallet to receive funds from auto-mining ==="

# Wait for main wallet to receive funds
echo "Waiting for main wallet to receive funds..."
TIMEOUT=120
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
    # Synchronize wallet
    if ! SYNC_RESULT=$(gq_check "$DEFAULT_URL" \
      -q 'mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
      }' -v "account=$MAIN_WALLET"); then
        echo "WARNING: Failed to synchronize main wallet (continuing...)"
    fi

    # Get funding wallet balance
    FUNDING_BALANCE=$(gq "$DEFAULT_URL" \
      -q 'query ($account: Int!) {
        balanceByAccount(idAccount: $account) {
          orchard
        }
      }' \
      -v "account=$MAIN_WALLET" | jq -r '.data.balanceByAccount.orchard' | tr -d '[:space:]')

    if [ -n "$FUNDING_BALANCE" ] && [ "$FUNDING_BALANCE" != "null" ] && [ "$FUNDING_BALANCE" != "0" ]; then
        echo "Main wallet funded: $FUNDING_BALANCE"
        break
    fi

    echo "Waiting for funds... ($((ELAPSED))/$TIMEOUT)"
    sleep 5
    ELAPSED=$((ELAPSED + 5))
done

if [ "$FUNDING_BALANCE" = "null" ] || [ "$FUNDING_BALANCE" = "0" ] || [ -z "$FUNDING_BALANCE" ]; then
    echo "ERROR: Main wallet has insufficient balance (got: $FUNDING_BALANCE)"
    exit 1
fi

echo ""
echo "=== Step 4: Fund each participant's funding address ==="

# Build recipients array for payment
RECIPIENTS="{address: \"${FUNDING_ADDRESSES[0]}\", amount: 0.01}"
for i in $(seq 2 $N); do
    RECIPIENTS="$RECIPIENTS, {address: \"${FUNDING_ADDRESSES[$((i-1))]}\", amount: 0.01}"
done
TXID=$(gq "$DEFAULT_URL" \
  -q "mutation (\$account: Int!) {
    pay(idAccount: \$account
      payment: {
        recipients: [$RECIPIENTS]
      })
    }" \
  -v "account=$MAIN_WALLET" | jq -r '.data.pay' | tr -d '[:space:]')

echo "Funding transaction: $TXID"

# Mine blocks to confirm transaction
echo "Mining blocks for confirmation..."
HEIGHT=$(height)
curl --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [5] }' -H 'Content-type: application/json' http://127.0.0.1:18232/
wait_for_height $HEIGHT 5

# Synchronize the funding account for each participant
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"
    FUNDING_ACCOUNT="${FUNDING_ACCOUNTS[$((i-1))]}"

    echo "Synchronizing participant $i funding account..."
    gq "$GRAPHQL_URL" \
      -q 'mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
      }' -v "account=$FUNDING_ACCOUNT" > /dev/null
done

echo ""
echo "=== Step 5: Verify funding accounts received funds ==="

for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"
    FUNDING_ACCOUNT="${FUNDING_ACCOUNTS[$((i-1))]}"

    echo "Checking participant $i funding account balance..."
    TIMEOUT=30
    ELAPSED=0
    while [ $ELAPSED -lt $TIMEOUT ]; do
        # Synchronize before checking balance
        if ! SYNC_RESULT=$(gq_check "$GRAPHQL_URL" \
          -q 'mutation ($account: Int!) {
            synchronizeAccount(idAccount: $account)
          }' -v "account=$FUNDING_ACCOUNT"); then
            echo "WARNING: Failed to synchronize participant $i funding account (continuing...)"
        fi

        FUNDING_BALANCE=$(gq "$GRAPHQL_URL" \
          -q 'query ($account: Int!) {
            balanceByAccount(idAccount: $account) {
              orchard
            }
          }' \
          -v "account=$FUNDING_ACCOUNT" | jq -r '.data.balanceByAccount.orchard' | tr -d '[:space:]')

        if [ -n "$FUNDING_BALANCE" ] && [ "$FUNDING_BALANCE" != "null" ] && [ "$FUNDING_BALANCE" != "0" ]; then
            echo "Participant $i funding account balance: $FUNDING_BALANCE"
            break
        fi

        echo "Participant $i funding account balance: $FUNDING_BALANCE, retrying... ($((ELAPSED))/$TIMEOUT)"
        sleep 2
        ELAPSED=$((ELAPSED + 2))
    done

    if [ "$FUNDING_BALANCE" = "null" ] || [ "$FUNDING_BALANCE" = "0" ] || [ -z "$FUNDING_BALANCE" ]; then
        echo "ERROR: Participant $i funding account has insufficient balance (expected > 0, got: $FUNDING_BALANCE)"
        exit 1
    fi
done

echo ""
echo "=== Step 6: Exchange DKG addresses between participants ==="

# Each participant sets addresses for all other participants
for i in $(seq 1 $N); do
    for j in $(seq 1 $N); do
        if [ "$i" -eq "$j" ]; then
            continue  # Don't set own address
        fi

        GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"
        TARGET_ADDRESS="${DKG_ADDRESSES[$((j-1))]}"

        echo "Participant $i sets address for participant $j: $TARGET_ADDRESS"

        gq "$GRAPHQL_URL" \
          -q 'mutation ($id: Int!, $address: String!) {
            dkgSetAddress(idParticipant: $id, address: $address)
          }' \
          -v "id=$j" \
          -v "address=$TARGET_ADDRESS" > /dev/null
    done
done

echo ""
echo "=== Step 7: Execute DKG on all participants ==="

# Execute DKG on each participant (they should complete in parallel)
FROST_ACCOUNTS=()
SHARED_ADDRESSES=()

# First, initiate doDkg on all participants
echo "Initiating DKG on all participants..."
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"
    echo "Starting DKG on participant $i..."
    if ! DKG_RESULT=$(gq_check "$GRAPHQL_URL" \
      -q 'mutation {
        doDkg
      }'); then
        echo "ERROR: Failed to initiate DKG for participant $i (see above for details)"
    fi
done

# Now loop until all participants complete DKG or timeout (5 minutes)
echo "Waiting for all participants to complete DKG..."
TIMEOUT=300
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
    ALL_COMPLETED=true

    # Check each participant
    for i in $(seq 1 $N); do
        # Check if DKG completed by checking if account with name Dkg-Test-$i exists
        DB_PATH="/tmp/regtest_dkg_test_${i}.db"
        FROST_CHECK=$(sqlite3 "$DB_PATH" "SELECT name FROM accounts WHERE name = 'Dkg-Test-$i';" 2>/dev/null || echo "")

        if [ -z "$FROST_CHECK" ]; then
            # This participant hasn't completed yet
            ALL_COMPLETED=false
        fi
    done

    # If all completed, break out of loop
    if [ "$ALL_COMPLETED" = true ]; then
        echo "All participants completed DKG successfully"
        break
    fi

    sleep 10
    ELAPSED=$((ELAPSED + 10))
done

if [ $ELAPSED -ge $TIMEOUT ]; then
    echo "ERROR: DKG timed out after $TIMEOUT seconds"
    exit 1
fi

echo ""
echo "=== Step 8: Wait for DKG completion and verify results ==="

# Check that all participants generated the same shared address
SHARED_ADDRESS=null
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    # Get the FROST account ID from database (account named Dkg-Test-$i)
    DB_PATH="/tmp/regtest_dkg_test_${i}.db"
    FROST_ACCOUNT_ID=$(sqlite3 "$DB_PATH" "SELECT id_account FROM accounts WHERE name = 'Dkg-Test-$i';" 2>/dev/null)

    if [ -z "$FROST_ACCOUNT_ID" ]; then
        echo "ERROR: No FROST account found for participant $i"
        exit 1
    fi

    # Get the FROST account address
    FROST_ADDRESS=$(gq "$GRAPHQL_URL" \
      -q 'query ($account: Int!) {
        addressByAccount(idAccount: $account) {
          orchard
        }
      }' \
      -v "account=$FROST_ACCOUNT_ID" | jq -r '.data.addressByAccount.orchard' | tr -d '[:space:]')

    if [ -z "$FROST_ADDRESS" ] || [ "$FROST_ADDRESS" = "null" ]; then
        echo "ERROR: No FROST account found for participant $i"
        exit 1
    fi

    echo "Participant $i shared address: $FROST_ADDRESS"

    # Verify all participants have the same shared address
    if [ "$SHARED_ADDRESS" = "null" ]; then
        SHARED_ADDRESS="$FROST_ADDRESS"
    elif [ "$SHARED_ADDRESS" != "$FROST_ADDRESS" ]; then
        echo "ERROR: Participants generated different shared addresses!"
        echo "  Participant 1: $SHARED_ADDRESS"
        echo "  Participant $i: $FROST_ADDRESS"
        exit 1
    fi

    FROST_ACCOUNTS+=("$FROST_ACCOUNT_ID")
    SHARED_ADDRESSES+=("$FROST_ADDRESS")
done

echo ""
echo "=== Step 9: Fund the shared FROST address ==="

# Synchronize the main wallet to get latest balance
gq "$DEFAULT_URL" \
  -q 'mutation ($account: Int!) {
    synchronizeAccount(idAccount: $account)
  }' -v "account=$MAIN_WALLET" > /dev/null

# Get funding wallet balance
FUNDING_BALANCE=$(gq "$DEFAULT_URL" \
  -q 'query ($account: Int!) {
    balanceByAccount(idAccount: $account) {
      orchard
    }
  }' \
  -v "account=$MAIN_WALLET" | jq -r '.data.balanceByAccount.orchard' | tr -d '[:space:]')

echo "Funding wallet balance: $FUNDING_BALANCE"

# Send funds to shared FROST address
echo "Sending 0.1 ZEC to shared FROST address..."
TXID=$(gq "$DEFAULT_URL" \
  -q 'mutation ($account: Int!, $address: String!, $amount: BigDecimal!){
    pay(idAccount: $account
      payment: {
        recipients: [{
          address: $address
          amount: $amount
        }]
      })
    }' \
  -v "account=$MAIN_WALLET" \
  -v "address=$SHARED_ADDRESS" \
  -v 'amount=0.1' | jq -r '.data.pay')

echo "Funding transaction: $TXID"

echo ""
echo "=== Step 10: Mine blocks and synchronize ==="

# Mine blocks to confirm transaction
echo "Mining blocks..."
HEIGHT=$(height)
curl --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [5] }' -H 'Content-type: application/json' http://127.0.0.1:18232/
wait_for_height $HEIGHT 5

# Synchronize all participants
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    echo "Synchronizing participant $i..."
    gq "$GRAPHQL_URL" \
      -q 'mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
      }' \
      -v "account=${FROST_ACCOUNTS[$((i-1))]}" > /dev/null
done

echo ""
echo "=== Step 11: Verify shared address balance ==="

for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    FINAL_BALANCE=$(gq "$GRAPHQL_URL" \
      -q 'query ($account: Int!) {
        balanceByAccount(idAccount: $account) {
          orchard
        }
      }' \
      -v "account=${FROST_ACCOUNTS[$((i-1))]}" | jq -r '.data.balanceByAccount.orchard' | tr -d '[:space:]')

    echo "Participant $i FROST balance: $FINAL_BALANCE"

    if [ "$FINAL_BALANCE" != "0.10000000" ]; then
        echo "ERROR: Expected 0.10000000, got $FINAL_BALANCE"
        echo "Test failed!"
        exit 1
    fi
done

echo ""
echo "=== ✅ DKG Test Passed! ==="
echo "All 3 participants successfully:"
echo "  - Generated the same shared address: $SHARED_ADDRESS"
echo "  - Received funding of 0.1 ZEC"
echo "  - Synchronized independently"
echo ""
echo "Shared FROST address: $SHARED_ADDRESS"
echo "FROST accounts: ${FROST_ACCOUNTS[@]}"

# Cleanup will happen automatically on exit
