---
name: voting-stack
description: "Bring the local coin-weighted voting stack (svoted, nf-server PIR, signed voting config) up on the regtest chain with tests/tools/voting_stack.sh, for the voting UI runs such as the docs screenshot run; how to get the binaries, what `up`/`down`/`status` do, and what bites."
---

# Local voting stack

`tests/tools/voting_stack.sh` brings up, on the regtest chain that is already
running (see the `frost-ui-tests` skill for zebra and lightwalletd), the same
voting services the CI job `.github/workflows/e2e-voting.yml` uses, through the
same scripts under `scripts/`:

| service | what | port |
|---|---|---|
| svoted | the shielded-vote chain (one validator, one vote manager) | 26657 RPC, 1317 REST, 9190 gRPC |
| nf-server | the nullifier PIR server, loaded with the regtest nullifiers up to the snapshot | 3000 |
| config server | `scripts/serve-e2e-voting-config.py`, serving the signed configuration and proxying `/vote` and `/pir` | 8443, TLS |

```bash
tests/tools/voting_stack.sh up --title "Zcash Community Poll" --proposals proposals.json --duration 3600
tests/tools/voting_stack.sh status
tests/tools/voting_stack.sh down
```

`up` always starts from nothing (it runs `down` first): a fresh chain, the PIR
import at `--snapshot` (default: the chain height rounded down to a multiple
of ten, which the import requires), one active round and a fresh signing key.
It takes about twenty seconds and ends by printing `data/voting/voting.env`:

```
VOTING_CONFIG_URL='https://localhost:8443/static-voting-config.json?checksum=sha256:…'
VOTING_CA_CERT='…/data/voting-ca/ca.crt'
VOTING_ROUND_TITLE='Zcash Community Poll'
VOTING_SNAPSHOT_HEIGHT=1510
```

A wallet votes with the notes it held at the snapshot height, so fund the
voter before `up`, or pass `--snapshot` with a height after the funding.
`--proposals` is the array the session takes (`id`, `title`, `description`,
`options[{index, label, description}]`); without it the round carries CI's
"Proposal 1 / Choice 1" placeholders. `--duration` is the voting window in
seconds (CI uses 120; a screenshot run wants an hour).

## Binaries (untracked `tools/voting`)

```bash
mkdir -p tools/voting && cd tools/voting
V=v1.6.1-rc.6
curl -fsSLO https://github.com/valargroup/vote-sdk/releases/download/$V/shielded-vote-$V-darwin-arm64.tar.gz
mkdir svoted-archive && tar -xzf shielded-vote-$V-darwin-arm64.tar.gz -C svoted-archive
cp svoted-archive/*/bin/svoted svoted
git clone --depth 1 --branch $V https://github.com/valargroup/vote-sdk.git vote-sdk   # init script, protos, config tool
go build -C vote-sdk -o "$PWD/voting-config" ./cmd/voting-config
git clone https://github.com/hhanh00/vote-nullifier-pir.git && git -C vote-nullifier-pir checkout 8ce99cc667282c584c05df745a2b7b446e8665ec
cargo build --locked --release --manifest-path vote-nullifier-pir/Cargo.toml -p nf-server --features serve
cp vote-nullifier-pir/target/release/nf-server nf-server
brew install grpcurl   # if missing
```

The versions are the ones pinned in the e2e workflow; bump them together.

## What bites

- The wallet reads the config from `VOTING_CONFIG_URL` (the Settings page's
  "Voting Config URL", or the `voting_config_url` prop / `setVotingConfigUrl`
  mutation). The checksum in the URL is of the static config, so a new `up`
  means a new URL.
- A vote is final for the notes it used: a wallet that voted in a round cannot
  vote again in it, headless or in the app. Check with `votingRounds` only.
- The PIR import reads lightwalletd (`REGTEST_LWD_URL`, default :8137) and
  needs the snapshot height to be a multiple of ten.
- macOS ships bash 3: the scripts avoid `mapfile`; keep it that way.
- The voting SDK insists on https for the static config, so the config
  server speaks TLS with a certificate issued by a throwaway CA kept in
  `data/voting-ca/ca.crt` (`voting_stack.sh ca` prints its path). The app
  trusts an extra root through the `ZKOOL_EXTRA_ROOT_CERT_PATH` environment
  variable; the docs orchestrator copies the CA into the app container and
  sets it before launching. curl needs `--cacert`.
- `down` removes `data/voting` entirely, including the svoted chain, so a round
  does not survive it; that is what makes `up` deterministic.
