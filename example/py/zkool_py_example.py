import gql
import gql.transport.aiohttp
from gql.transport.aiohttp import AIOHTTPTransport

transport = gql.transport.aiohttp.AIOHTTPTransport(url="http://localhost:8000/graphql")
client = gql.Client(transport=transport)

# Create a new account
createAccountReq = gql.gql("""mutation {
    createAccount(newAccount: {
    name: "Test"
    useInternal: true
    key: ""
    aindex: 0
    })
}""")
result = client.execute(createAccountReq)
idAccount = result["createAccount"]

# Get its default address
getAddressReq = gql.gql("""query ($idAccount: Int!) {
    addressByAccount(idAccount: $idAccount) {
    ua
    }
}""")
result = client.execute(getAddressReq, variable_values = {
    "idAccount": idAccount
})
print(result["addressByAccount"]["ua"])
