from graphql_api import GraphQLAPI, field
import uvicorn
from graphql_mcp import GraphQLMCP

server = GraphQLMCP.from_remote_url(
    url="http://localhost:8000/graphql",
    name="Zcash",
    headers={}  # Optional: auth headers
)

app = server.http_app()

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8001)
