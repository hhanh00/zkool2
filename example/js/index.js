import { createClient } from "graphql-ws";

const client = createClient({
  url: "ws://localhost:8000/subscriptions",
});

// create a new account
async function main() {
  const createAccountReq = client.iterate({
    query: `mutation {
      createAccount(newAccount: {
        name: "Test"
        useInternal: true
        key: ""
        aindex: 0
      })
    }`,
  });
  var rep = await createAccountReq.next();
  const {
    data: { createAccount: idAccount },
  } = rep.value;

  // Get the account address
  const getAddressReq = client.iterate({
    query: `query {
      addressByAccount(idAccount: ${idAccount}) {
        ua
      }
    }`,
  });
  rep = await getAddressReq.next();
  const {
    data: { addressByAccount: { ua } },
  } = rep.value
  console.log(ua);

  // Monitor incoming transactions
  const subscription = client.iterate({
    query: `subscription { events(idAccount: ${idAccount}) { type txid } }`,
  });

  for await (const event of subscription) {
    console.log(event)
  }
}

main();
