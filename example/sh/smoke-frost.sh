#!/usr/bin/env bash
set -eo pipefail

# Test for FROST SIGN using GraphQL API
# This test uses the shared account created during smoke-dkg

# Source common functions
SCRIPT_DIR="$(dirname "$0")"
source "${SCRIPT_DIR}/smoke-common.sh"

echo "=== Using existing DKG instances for FROST SIGN Test ==="
echo ""

# Kill any existing instances and start fresh
pkill zkool_graphql 2>/dev/null || true
sleep 2

# Start the 3 participant instances
GRAPHQL_URLS=()
for i in 1 2 3; do
    PORT=$((8000 + i))
    echo "Starting participant $i on port $PORT with database /tmp/regtest_dkg_test_${i}.db"
    "$ZKOOL_GRAPHQL" \
        -d /tmp/regtest_dkg_test_${i}.db \
        -p $PORT \
        -l http://localhost:8137 \
        > /tmp/graphql_dkg_${i}.log 2>&1 &
    eval "PID_${i}=\$!"
    GRAPHQL_URLS+=("http://localhost:$PORT/graphql")
done

# Start the default instance
echo "Starting default instance on port 8000 with database /tmp/regtest_dkg_default.db"
"$ZKOOL_GRAPHQL" \
    -d /tmp/regtest_dkg_default.db \
    -p 8000 \
    -l http://localhost:8137 \
    > /tmp/graphql_dkg_default.log 2>&1 &
PID_DEFAULT=$!
DEFAULT_URL="http://localhost:8000/graphql"

echo "Waiting for instances to start..."
sleep 3

echo "All instances started"
echo ""

echo "=== Step 1: Find the shared account ID for participant #2 (coordinator) ==="

# Get the FROST account ID from database for participant #2 (coordinator)
DB_PATH="/tmp/regtest_dkg_test_2.db"
if [ ! -f "$DB_PATH" ]; then
    echo "ERROR: DKG database not found at $DB_PATH"
    echo "Please run smoke-dkg.sh first to create the shared account"
    cleanup "regtest_dkg"
    exit 1
fi

COORDINATOR_FROST_ACCOUNT_ID=$(sqlite3 "$DB_PATH" "SELECT id_account FROM accounts WHERE name = 'Dkg-Test-2';" 2>/dev/null)

if [ -z "$COORDINATOR_FROST_ACCOUNT_ID" ]; then
    echo "ERROR: No FROST account found for participant #2 (coordinator)"
    echo "Looking for account named 'Dkg-Test-2' in $DB_PATH"
    cleanup "regtest_dkg"
    exit 1
fi

echo "Coordinator (participant #2) FROST account ID: $COORDINATOR_FROST_ACCOUNT_ID"

echo ""
echo "=== Step 2: Create a new regular account to receive funds ==="

# Create a regular account to receive funds after FROST signing
RECEIVER_ACCOUNT=$(gq "$DEFAULT_URL" \
  -q 'mutation {
  createAccount(newAccount: {
    name: "FROST-Receiver"
    key: ""
    aindex: 0
    useInternal: false
    birth: 1
  })
}' | jq -r '.data.createAccount' | tr -d '[:space:]')

echo "Receiver account ID: $RECEIVER_ACCOUNT"

# Get the receiver's address
RECEIVER_ADDRESS=$(gq "$DEFAULT_URL" \
  -q 'query ($account: Int!) {
    addressByAccount(idAccount: $account) {
      orchard
    }
  }' \
  -v "account=$RECEIVER_ACCOUNT" | jq -r '.data.addressByAccount.orchard' | tr -d '[:space:]')

echo "Receiver address: $RECEIVER_ADDRESS"

echo ""
echo "=== Step 3: Prepare payment from FROST shared account to receiver ==="

# Get the GraphQL URL for participant #2 (coordinator)
COORDINATOR_URL="${GRAPHQL_URLS[1]}"  # Index 1 is participant #2

# Get the funding account for the coordinator (for message passing)
COORDINATOR_FUNDING_ACCOUNT=$(sqlite3 "/tmp/regtest_dkg_test_2.db" "SELECT id_account FROM accounts WHERE name = 'DKG-Fund';" 2>/dev/null)

if [ -z "$COORDINATOR_FUNDING_ACCOUNT" ]; then
    echo "ERROR: No funding account found for coordinator"
    cleanup "regtest_dkg"
    exit 1
fi

echo "Coordinator funding account: $COORDINATOR_FUNDING_ACCOUNT"

# Synchronize the FROST shared account before preparing payment
echo "Synchronizing coordinator's FROST account..."
gq "$COORDINATOR_URL" \
  -q 'mutation ($account: Int!) {
    synchronizeAccount(idAccount: $account)
  }' \
  -v "account=$COORDINATOR_FROST_ACCOUNT_ID" > /dev/null

# Prepare payment from FROST shared account to receiver
# Use prepare_send query to get the PCZT
PREPARE_RESULT=$(gq "$COORDINATOR_URL" \
  -q 'query ($account: Int!, $address: String!, $amount: BigDecimal!) {
    prepareSend(
      idAccount: $account
      payment: {
        recipients: [{
          address: $address
          amount: $amount
        }]
      }
    )
  }' \
  -v "account=$COORDINATOR_FROST_ACCOUNT_ID" \
  -v "address=$RECEIVER_ADDRESS" \
  -v 'amount=0.05')

echo "Prepare result: $PREPARE_RESULT"

# Extract the PCZT (hex-encoded) from the prepare result
PCZT=$(echo "$PREPARE_RESULT" | jq -r '.data.prepareSend' | tr -d '[:space:]')

if [ -z "$PCZT" ] || [ "$PCZT" = "null" ]; then
    echo "ERROR: Failed to prepare payment"
    echo "Response: $PREPARE_RESULT"
    cleanup "regtest_dkg"
    exit 1
fi

echo "PCZT prepared: ${PCZT:0:50}..."

echo ""
echo "=== Step 4: Get FROST account IDs for all participants ==="

# Get FROST account IDs for all participants from their DKG databases
for i in 1 2 3; do
    DKG_DB="/tmp/regtest_dkg_test_${i}.db"

    if [ ! -f "$DKG_DB" ]; then
        echo "ERROR: DKG database not found at $DKG_DB"
        echo "Please run smoke-dkg.sh first to create the shared account"
        cleanup "regtest_dkg"
        exit 1
    fi

    # Get the FROST account ID for this participant
    PARTICIPANT_FROST_ACCOUNT_ID=$(sqlite3 "$DKG_DB" "SELECT id_account FROM accounts WHERE name = 'Dkg-Test-${i}';" 2>/dev/null)

    if [ -z "$PARTICIPANT_FROST_ACCOUNT_ID" ]; then
        echo "ERROR: No FROST account found for participant #$i"
        cleanup "regtest_dkg"
        exit 1
    fi

    echo "Participant $i FROST account ID: $PARTICIPANT_FROST_ACCOUNT_ID"

    # Store the account ID for later use
    eval "FROST_ACCOUNT_${i}=\$PARTICIPANT_FROST_ACCOUNT_ID"
done

echo "All participants have their FROST account IDs loaded"

echo ""
echo "=== Step 5: Each participant performs FROST signing ==="

# Each participant performs the signing step
for i in 1 2 3; do
    GRAPHQL_URL="${GRAPHQL_URLS[$((i-1))]}"
    eval "frost_account_id=\$FROST_ACCOUNT_${i}"

    if [ -z "$frost_account_id" ]; then
        echo "ERROR: No FROST account ID found for participant $i"
        exit 1
    fi

    # Get the funding account for this participant (for message passing)
    DB_PATH="/tmp/regtest_dkg_test_${i}.db"
    FUNDING_ACCOUNT=$(sqlite3 "$DB_PATH" "SELECT id_account FROM accounts WHERE name = 'DKG-Fund';" 2>/dev/null)

    if [ -z "$FUNDING_ACCOUNT" ]; then
        echo "ERROR: No funding account found for participant $i"
        exit 1
    fi

    echo "Participant $i signing with funding account $FUNDING_ACCOUNT..."
    SIGN_RESULT=$(gq "$GRAPHQL_URL" \
      -q 'mutation ($account: Int!, $coordinator: Int!, $funding: Int!, $pczt: String!) {
        frostSign(
          idAccount: $account
          idCoordinator: $coordinator
          messageAccount: $funding
          pczt: $pczt
        )
      }' \
      -v "account=$frost_account_id" \
      -v "coordinator=2" \
      -v "funding=$FUNDING_ACCOUNT" \
      -v "pczt=$PCZT")

    echo "Participant $i sign result: $SIGN_RESULT"

    # Check for errors - either GraphQL errors or frostSign returning false
    if echo "$SIGN_RESULT" | grep -q '"error"' || echo "$SIGN_RESULT" | jq -e '.data.frostSign == false' >/dev/null 2>&1; then
        echo "ERROR: Participant $i signing failed"
        echo "Response: $SIGN_RESULT"
        exit 1
    fi
done

echo "All participants completed signing round"

# Check FROST shared account balance to see if transaction was created
COORDINATOR_URL="${GRAPHQL_URLS[1]}"
FROST_BALANCE_BEFORE=$(gq "$COORDINATOR_URL" \
  -q 'query ($account: Int!) {
    balanceByAccount(idAccount: $account) {
      orchard
    }
  }' \
  -v "account=$COORDINATOR_FROST_ACCOUNT_ID" | jq -r '.data.balanceByAccount.orchard')
echo "FROST shared account balance after signing: $FROST_BALANCE_BEFORE ZEC"

echo ""
echo "=== Step 6: Verify transaction completed and receiver received funds ==="

# Loop for up to 1 minute to verify transaction completed
TIMEOUT=60
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
    # Synchronize receiver account
    echo "Synchronizing receiver account..."
    gq "$DEFAULT_URL" \
      -q 'mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
      }' \
      -v "account=$RECEIVER_ACCOUNT" > /dev/null

    # Check receiver balance
    RECEIVER_BALANCE=$(gq "$DEFAULT_URL" \
      -q 'query ($account: Int!) {
        balanceByAccount(idAccount: $account) {
          orchard
        }
      }' \
      -v "account=$RECEIVER_ACCOUNT" | jq -r '.data.balanceByAccount.orchard' | tr -d '[:space:]')

    echo "Receiver balance: $RECEIVER_BALANCE ZEC (waiting for 0.05 ZEC)"

    # Check if balance is 0.05 ZEC (the amount we sent)
    if [ "$RECEIVER_BALANCE" = "0.05000000" ]; then
        echo ""
        echo "=== ✅ Transaction completed successfully! ==="
        echo "Receiver account received 0.05 ZEC from FROST shared account"
        echo ""
        echo "=== Test completed successfully ==="
        exit 0
    fi

    sleep 10
    ELAPSED=$((ELAPSED + 10))
done

echo ""
echo "ERROR: Transaction did not complete within $TIMEOUT seconds"
echo "Final receiver balance: $RECEIVER_BALANCE ZEC (expected 0.05 ZEC)"
echo ""
echo "Check the logs for more details: tail -f /tmp/graphql_dkg_*.log"
echo ""
echo "To see signing status, you can:"
echo "  1. Check participant logs: tail -f /tmp/graphql_dkg_1.log"
echo "  2. Check database: sqlite3 /tmp/regtest_dkg_test_2.db \"SELECT * FROM frost_commitments\""
echo "  3. Continue calling frost_sign on participants to progress the signing"

# Don't auto-cleanup - preserve DKG databases for further testing
# Users can manually clean up with: pkill zkool_graphql

exit 1
