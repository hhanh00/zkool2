#!/usr/bin/env bash
set -euo pipefail
set -x

# Test for 3-out-of-3 FROST DKG using GraphQL API
# This test sets up 3 separate zkool_graphql instances with distinct databases

# Configuration
N=3                     # Number of participants
T=3                      # Threshold (all 3 must participate)
PORT_BASE=8001            # Starting port for GraphQL instances
DB_BASE=8001              # Starting port for database connections
GRAPHQL_BASE_URL="http://localhost:8137"  # LWD URL for all participants

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    for i in $(seq 1 $N); do
        eval "pid=\$PID_${i}"
        if [ -n "$pid" ]; then
            kill "$pid" 2>/dev/null || true
        fi
        rm -rf "/tmp/dkg_test_${i}.db" 2>/dev/null || true
        rm -f "/tmp/graphql_${i}.log" 2>/dev/null || true
    done
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

# Seed for funded wallet
SEED="secret-extended-key-test1"
FUNDING_SEED="secret-extended-key-test-spending"

echo "=== Setting up 3-out-of-3 FROST DKG Test ==="

# Start 3 separate zkool_graphql instances with distinct databases
GRAPHQL_URLS=()
for i in $(seq 1 $N); do
    PORT=$((PORT_BASE + i - 1))
    DB_PATH="/tmp/dkg_test_${i}.db"

    echo "Starting participant $i on port $PORT with database $DB_PATH"
    rm -f "$DB_PATH"

    # Start zkool_graphql in background
    /Users/hanhhuynhhuu/projects/zkool/target/release/zkool_graphql \
        -d "$DB_PATH" \
        -p $PORT \
        -c "127.0.0.1:18232" \
        > "/tmp/graphql_${i}.log" 2>&1 &

    # Save PID for cleanup
    eval "PID_${i}=\$!"
    GRAPHQL_URLS+=("http://localhost:$PORT/graphql")

    # Wait for server to start
    echo "Waiting for participant $i to start..."
    sleep 2
done

echo "All participants started"

# Function to query GraphQL
gq() {
    local url=$1
    shift
    gq "$url" "$@"
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
MAIN_WALLET=$(gq "${GRAPHQL_URLS[0]}" \
  -q 'mutation ($main: String!) {
  createAccount(newAccount: {
    name: "Funding"
    key: $main
    aindex: 0
    useInternal: false
    birth: 1
  })
}' -v "main=$SEED" | jq -r '.data.createAccount')

echo "Funding wallet: $MAIN_WALLET"

echo ""
echo "=== Step 2: Initialize DKG for each participant ==="

# Initialize DKG parameters for each participant
DKG_ADDRESSES=()
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
    }' | jq -r '.data.createAccount')

    echo "Participant $i funding account: $FUNDING_ACCOUNT"

    # Initialize DKG with participant ID
    DKG_ADDRESS=$(gq "$GRAPHQL_URL" \
      -q 'mutation ($name: String!, $t: Int!, $n: Int!, $funding: Int!, $id: Int!) {
      dkgStart(
        name: "Frost-Test-3of3"
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
      -v "id=$i" | jq -r '.data.dkgStart')

    echo "Participant $i DKG address: $DKG_ADDRESS"
    DKG_ADDRESSES+=("$DKG_ADDRESS")

    # Synchronize the funding account
    gq "$GRAPHQL_URL" \
      -q 'mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
      }' -v "account=$FUNDING_ACCOUNT" > /dev/null
done

echo ""
echo "=== Step 3: Exchange DKG addresses between participants ==="

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
echo "=== Step 4: Execute DKG on all participants ==="

# Execute DKG on each participant (they should complete in parallel)
FROST_ACCOUNTS=()
SHARED_ADDRESSES=()
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    echo "Executing DKG on participant $i..."
    gq "$GRAPHQL_URL" \
      -q 'mutation {
        doDkg
      }' > /dev/null

    echo "Participant $i DKG execution started"

    # Give it time to process
    sleep 2
done

echo ""
echo "=== Step 5: Wait for DKG completion and verify results ==="

# Check that all participants generated the same shared address
SHARED_ADDRESS=null
for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    # Get all accounts and find the FROST account
    ACCOUNTS=$(gq "$GRAPHQL_URL" \
      -q 'query {
        accounts {
          id
          name
        }
      }' | jq -r '.data.accounts[] | select(.name | startswith("frost-")) | .id')

    if [ -z "$ACCOUNTS" ]; then
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
      -v "account=$ACCOUNTS" | jq -r '.data.addressByAccount.orchard')

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

    FROST_ACCOUNTS+=("$ACCOUNTS")
    SHARED_ADDRESSES+=("$FROST_ADDRESS")
done

echo ""
echo "=== Step 6: Fund the shared FROST address ==="

# Mine some blocks to generate funds for funding wallet
echo "Generating initial blocks..."
HEIGHT=$(height)
curl --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [100] }' -H 'Content-type: application/json' http://127.0.0.1:18232/
wait_for_height $HEIGHT 100

# Get funding wallet balance
FUNDING_BALANCE=$(gq "${GRAPHQL_URLS[0]}" \
  -q 'query ($account: Int!) {
    balanceByAccount(idAccount: $account) {
      orchard
    }
  }' \
  -v "account=$MAIN_WALLET" | jq -r '.data.balanceByAccount.orchard')

echo "Funding wallet balance: $FUNDING_BALANCE"

# Send funds to shared FROST address
echo "Sending 10 ZEC to shared FROST address..."
TXID=$(gq "${GRAPHQL_URLS[0]}" \
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
  -v 'amount=10.0' | jq -r '.data.pay')

echo "Funding transaction: $TXID"

echo ""
echo "=== Step 7: Mine blocks and synchronize ==="

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
echo "=== Step 8: Verify shared address balance ==="

for i in $(seq 1 $N); do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"

    FINAL_BALANCE=$(gq "$GRAPHQL_URL" \
      -q 'query ($account: Int!) {
        balanceByAccount(idAccount: $account) {
          orchard
        }
      }' \
      -v "account=${FROST_ACCOUNTS[$((i-1))]}" | jq -r '.data.balanceByAccount.orchard')

    echo "Participant $i FROST balance: $FINAL_BALANCE"

    if [ "$FINAL_BALANCE" != "10.00000000" ]; then
        echo "ERROR: Expected 10.00000000, got $FINAL_BALANCE"
        echo "Test failed!"
        exit 1
    fi
done

echo ""
echo "=== ✅ DKG Test Passed! ==="
echo "All 3 participants successfully:"
echo "  - Generated the same shared address: $SHARED_ADDRESS"
echo "  - Received funding of 10 ZEC"
echo "  - Synchronized independently"
echo ""
echo "Shared FROST address: $SHARED_ADDRESS"
echo "FROST accounts: ${FROST_ACCOUNTS[@]}"

# Cleanup will happen automatically on exit
