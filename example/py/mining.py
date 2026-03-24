import gql
import gql.transport.aiohttp
from gql.transport.aiohttp import AIOHTTPTransport
import os
import time

transport = gql.transport.aiohttp.AIOHTTPTransport(url="http://localhost:8000/graphql", timeout=60)
client = gql.Client(transport=transport, execute_timeout = 60)

from decimal import Decimal

MATURITY_THRESHOLD = 100
MAX_NOTES = 10

def run(miner_seed: str, seed: str, to_address: str):
    height = client.execute(gql.gql("query { currentHeight }"))["currentHeight"]
    print(f"Height: {height}")

    miner = client.execute(gql.gql("""
        mutation CreateAccount($account: NewAccount!) {
            createAccount(newAccount: $account)
        }"""), variable_values = {"account": {"name": "miner", "key": miner_seed, "aindex": 0, "birth": 1, "useInternal": False}})["createAccount"]
    print(f"Miner id: {miner}")

    wallet = client.execute(gql.gql("""
        mutation CreateAccount($account: NewAccount!) {
            createAccount(newAccount: $account)
        }"""), variable_values = {"account": {"name": "wallet", "key": seed, "aindex": 0, "birth": 1, "useInternal": False}})["createAccount"]
    print(f"Wallet id: {wallet}")

    client.execute(gql.gql("""
        mutation Synchronize($ids: [Int!]!) { synchronize(idAccounts: $ids) }
    """), variable_values = {"ids": [miner, wallet]})

    all_notes = client.execute(gql.gql("""
        query NotesByAccount($id: Int!) {
            notesByAccount(idAccount: $id) { id height value }
        }
    """), variable_values = {"id": miner})["notesByAccount"]

    notes = [n for n in all_notes if n["height"] < height - MATURITY_THRESHOLD][:MAX_NOTES]
    if not notes:
        raise RuntimeError("No sufficiently mature notes found.")
    print(f"Selected {len(notes)} mature note(s)")

    total = sum(Decimal(n["value"]) for n in notes)
    print(total)
    txid = client.execute(gql.gql("""
        mutation Pay($id: Int!, $payment: Payment!) {
            pay(idAccount: $id, payment: $payment)
        }
    """), variable_values = {"id": miner, "payment": {
        "recipients": [{"address": to_address, "amount": str(total)}],
        "recipientPaysFee": True,
        "confirmations": MATURITY_THRESHOLD,
    }})["pay"]
    print(f"Done. txid: {txid}")

    time.sleep(30)

    client.execute(gql.gql("""
        mutation Synchronize($id: Int!) { synchronizeAccount(idAccount: $id) }
    """), variable_values = {"id": wallet})

    balance = client.execute(gql.gql("""
        query GetBalance($id: Int!) { balanceByAccount(idAccount: $id) {
        orchard }}
    """), variable_values = {"id": wallet})["balanceByAccount"]
    print(balance)
    orchard = float(balance['orchard'])
    assert orchard > 0, f"Expected positive orchard balance, got {orchard}"

run(os.environ["MINER_SEED"], os.environ["SEED"],
    os.environ["DESTINATION_ADDRESS"])
