# zkool-tests

Python test utilities and integration tests for zkool.

## Setup (uv)

```bash
cd tests
uv sync
```

## Environment Variables

```bash
export GRAPHQL_URL="http://localhost:8000/graphql"
export WS_URL="ws://localhost:8000/subscriptions"
export RPC_URL="http://127.0.0.1:18232/"
export SEED="invite couch cloud pave stuff cabbage usual rigid dragon warm cable price fame warfare next swallow worth opera suggest flame patch undo position arctic"
```

## Running Tests

```bash
# Run all tests
.venv/bin/uv run pytest

# Run smoke test
.venv/bin/uv run pytest tests/test_smoke.py -v

# Run with output
.venv/bin/uv run pytest tests/test_smoke.py -v -s
```

## Setup (pip)

```bash
cd tests
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pytest
```
