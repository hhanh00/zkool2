import { createClient, Client } from "graphql-ws";
import Big from "big"

async function gql(
  client: Client,
  query: string,
  variables?: Record<string, unknown>
): Promise<any> {
  const iter = client.iterate({ query, variables });
  const result = await iter.next();
  if (result.done) throw new Error("GraphQL iterator completed with no data");
  if (!result.value.data) {
    const msg = result.value.errors?.map((e: any) => e.message).join(", ") ?? "unknown error";
    throw new Error(`GraphQL error: ${msg}`);
  }
  return result.value.data;
}

class WalletAPI {
  constructor(private client: Client) {}

  async getCurrentHeight(): Promise<number> {
    const data = await gql(this.client, `query { currentHeight }`);
    return data.currentHeight;
  }

  // NewAccount input: { name, key, aindex, birth }
  async createAccount(account: Record<string, unknown>): Promise<any> {
    const data = await gql(
      this.client,
      `mutation CreateAccount($account: NewAccount!) {
        createAccount(newAccount: $account)
      }`,
      { account }
    );
    return data.createAccount;
  }

  async synchronizeAccount(idAccount: number): Promise<any> {
    const data = await gql(
      this.client,
      `mutation SynchronizeAccount($idAccount: Int!) {
        synchronizeAccount(idAccount: $idAccount)
      }`,
      { idAccount }
    );
    return data.synchronizeAccount;
  }

  async notesByAccount(idAccount: number): Promise<any[]> {
    const data = await gql(
      this.client,
      `query NotesByAccount($idAccount: Int!) {
        notesByAccount(idAccount: $idAccount) {
          id
          height
          pool
          value
          address
        }
      }`,
      { idAccount }
    );
    return data.notesByAccount;
  }

  async newAddresses(idAccount: number): Promise<any> {
    const data = await gql(
      this.client,
      `mutation NewAddresses($idAccount: Int!) {
        newAddresses(idAccount: $idAccount) {
          ua
          transparent
          sapling
          orchard
        }
      }`,
      { idAccount }
    );
    return data.newAddresses;
  }

  // Payment input: { recipients: [Recipient], ... }
  async pay(idAccount: number, payment: Record<string, unknown>): Promise<any> {
    const data = await gql(
      this.client,
      `mutation pay($idAccount: Int!, $payment: Payment!) {
        pay(idAccount: $idAccount, payment: $payment)
      }`,
      { idAccount, payment }
    );
    return data.prepareSend;
  }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

const MATURITY_THRESHOLD = 100;
const MAX_NOTES = 10;

function selectNotes(notes: any[], currentHeight: number): any[] {
  return notes
    .filter((n) => n.height < currentHeight - MATURITY_THRESHOLD)
    .slice(0, MAX_NOTES);
}

function sumValues(notes: any[]): bigint {
  return notes.reduce((acc, n) => acc.add(new Big(n.value)), new Big("0"));
}

// ── Main workflow ────────────────────────────────────────────────────────────

async function run(seed: string, wsEndpoint: string, toAddress: string): Promise<void> {
  const client = createClient({
    url: wsEndpoint,
    webSocketImpl: WebSocket,
  });

  const api = new WalletAPI(client);

  try {
    // 1. Current chain height
    console.log("Fetching current height…");
    const height = await api.getCurrentHeight();
    console.log(`  Height: ${height}`);

    // 2. Create account from seed
    console.log("Creating account…");
    const account = await api.createAccount({
      name: "wallet",
      key: seed,
      aindex: 0,
      birth: 1,
      useInternal: false
    });
    console.log(`  Account id:   ${account}`);

    // 3. Synchronize account
    console.log("Synchronizing account…");
    const syncResult = await api.synchronizeAccount(account);
    console.log(`  Sync result: ${syncResult}`);

    // 4. Fetch notes
    console.log("Fetching notes…");
    const allNotes = await api.notesByAccount(account);
    console.log(`  Total notes: ${allNotes.length}`);

    // 5. Select up to 10 mature notes (mined before height - 100)
    const selectedNotes = selectNotes(allNotes, height);
    if (selectedNotes.length === 0) {
      throw new Error("No sufficiently mature notes found.");
    }
    console.log(`  Selected ${selectedNotes.length} mature note(s)`);

    // 7. Prepare send for the full amount of selected notes
    const totalValue = sumValues(selectedNotes);
    const notes = selectedNotes.map(n => n.id);
    console.log(`Preparing send (value: ${totalValue})…`);
    const pczt = await api.pay(account, {
      recipients: [
        {
          address: toAddress,
          amount: totalValue.toString(),
        },
      ],
      recipientPaysFee: true,
      confirmations: 110,
    });
    console.log("Done.");
  } finally {
    await client.dispose();
  }
}

// ── Entry point ──────────────────────────────────────────────────────────────

const SEED = process.env.MINER_SEED!
const WS_ENDPOINT = process.env.WS_ENDPOINT!
const DESTINATION_ADDRESS = process.env.DESTINATION_ADDRESS!

run(SEED, WS_ENDPOINT, DESTINATION_ADDRESS).catch((err) => {
  console.error("Fatal:", err);
  process.exit(1);
});
