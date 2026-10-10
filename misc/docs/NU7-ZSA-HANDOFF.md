# NU7 / ZSA integration handoff

Recorded 2026-10-07. Resume here before revisiting the conversation.

## Current outcome

The ZSA-preserving NU7 dependency integration is published. Local Cargo path
overrides were replaced with pinned Git revisions, including transitive forks.
The user manually verified **sync and transfer on the deployed ZSA server**.

zkool PR [#1293](https://github.com/hhanh00/zkool2/pull/1293) is **merged** into
`main` (checked while writing this document):

- Integration branch: `zsa-integration`.
- Signed integration commit: `172709ac45bff8d4ba63f5093fe4f060af980ee0`.
- GitHub merge commit: `5dfe57865118a32c9c41b98ef8ee2b38e17b1ffa`.
- Merged at: `2026-10-07T06:31:43Z`.
- Local zkool checkout is still on `zsa-integration` at the integration commit;
  it has not been switched to or updated from `main` during this handoff.

**Decision: wait for a stable NU7-capable Zebra release before updating the
regtest tool pin and enabling NU7 in the regtest setup.** No Zebra pin or
activation configuration was changed for this document.

## Protocol constraints — do not undo

- ZSA is old, largely abandoned upstream code, but its deployed server and
  blockchain are alive. The deployed protocol MUST NOT change.
- ZSA is its own regtest network, not ZSA mainnet. The client selects the
  network semi-statically from `Coin`; it must not guess the protocol from a
  transaction branch ID alone.
- Deployed ZSA historically called its branch ID NU7. It is now represented
  semantically as `BranchId::Zsa`, retaining wire ID `0x77190ad8`.
- Official `BranchId::Nu7` uses `0x77190ad9`. These are distinct protocols.
- Keep the earlier NU6.3 merge's isolation of ZSA transaction versions, note
  formats, branch IDs and Orchard/Halo2 differences. In particular, do not
  reintroduce the incompatible generic/function-signature split into callers.
- Preserve merge history: the SDK NU7 integration is pre-release, and later
  work should integrate the marginal upstream changes through release, not
  repeat the entire merge or replace the existing history.
- Discuss any revert of previous isolation work with the user before making it.
- Retain the deployed FROST ciphersuite and serialized shares; conversion at
  the updated SDK boundary uses canonical bytes, not a deployed-format change.

## Published dependency baseline

The eight integration repositories were pushed on `zsa-integration` without
force-pushing. The consumer manifests and lockfile use published Git pins.
Normal intra-repository Cargo paths are not external overrides and remain valid.

| Repository | Pinned revision |
| --- | --- |
| `zcash-shielded-assets/librustzcash` | `53813b3a76546eabf2d1260d89436be7492a52dd` |
| `zcash-shielded-assets/halo2` | `0e3f1ea9dcba44fa4cd213b3f06b2aee3fc6e90e` |
| `zcash-shielded-assets/orchard` | `60697dd60ecbc14c7d8953316cea479944e605b8` |
| `zcash-shielded-assets/sapling-crypto` | `8f2e47e0d7a3db15507825f4c49a540779c6082e` |
| `zcash-shielded-assets/zcash_note_encryption` | `54d2360c3b16561d7c00167dc0af7358251469c9` |
| `hhanh00/zcash-trees` | `d22252485ad69ab236ef2bc42432edf68429ee6e` |
| `hhanh00/voting-circuits` | `f117ad8eb502385aec46d94585aae55be059c342` |
| `hhanh00/zcash_voting` | `d0e711a93742ad099daacaab08b89bd2bf7af90b` |

Other retained pins include `zcash_spec` at
`d5e84264d2ad0646b587a837f4e2424ca64d3a05` and legacy `reddsa` at
`975f9ca835c4b9196c81608e55192b0f711e951d`.
Treat root `Cargo.toml`, `rust/Cargo.toml` and `Cargo.lock` as the definitive
consumer dependency graph when resuming.

## Network and proof routing

See `rust/src/api/coin.rs` and `rust/src/pay/plan.rs`.

| Coin / branch | Current behavior |
| --- | --- |
| Coin 2, ordinary regtest | Upgrades through NU6.2 at height 1; NU6.3 at 250; `nu7: None`; `zsa: None`; normal Orchard mode |
| Coin 3, deployed ZSA regtest | Upgrades through NU6.2 at height 1; NU6.3 and official NU7 disabled; ZSA at height 1; ZSA Orchard mode |
| `BranchId::Zsa` | ZSA proving key |
| `BranchId::Nu6_3` / `BranchId::Nu7` | Ironwood proving key |
| Earlier ordinary branches | Vanilla proving key |

Ledger proof call sites were ported to the updated APIs, including the newly
required RNG arguments. This was an API adaptation, not a ZSA protocol change.

## Validation completed in this integration

These are the recorded results from the integration work, not tests rerun while
writing this document:

- User: deployed ZSA sync and transfer succeeded. ZSA issuance was not reported
  tested.
- User: ordinary ZEC logic is covered by CI tests; this is not evidence of
  activated official NU7 behavior.
- macOS debug application build succeeded with the local integration sources:
  `flutter build macos --debug`.
- Published dependency pins passed
  `cargo check --locked -p rlz --features zemu,bundled-sapling-params`.
  The recorded log was `/tmp/zkool-published-locked-check.log` (temporary;
  do not assume it survives).
- zkool: 22 payment-plan tests and 8 FROST DKG-plan tests passed.
- SDK: 193 core tests passed (81 PCZT, 68 primitives, 44 protocol);
  full-workspace all-feature test compilation, Rust 1.88 all-feature checking,
  and minimal-feature checking passed.
- Halo2: 8 ECC fixtures passed, with old/new fixtures checked for compatibility.
- Orchard: 12 encryption tests passed; deployed ZSA verifying-key compatibility
  and verification of an old 5120-byte proof passed.
- Sapling: 76 tests passed.
- No agent-driven live transaction broadcast or changes to the user's wallet.

Remaining gaps: official NU7 activation/sync/transfer testing, ZSA issuance,
and strict clippy. Existing warnings were not all eliminated.

Known separate limitation: standalone voting workspace resolution still has
the default Zakura backend's `bip32 = 0.6` pre-release pin conflicting with the
new librustzcash backend's stable `bip32` requirement. zkool's selected `lrz`
consumer build passes. Do not silently broaden this into a Zakura migration;
standalone voting lockfiles were not updated to hide the conflict.

## Facilitating the pre-release → release upstream merge

Treat the SDK release merge and the Zebra regtest upgrade as related but
separate changes. A stable Zebra tag is not an SDK release reference. Select
the actual upstream SDK/dependency commits needed for the final NU7 protocol;
do not merge a moving `main` merely because Zebra has released.

### Preserve these Git anchors

The local librustzcash history confirms two upstream merges:

- `265060cd` merged upstream `cd5eae71a000d426a63906be740952b5f232a558`.
- `3c016e30` then merged upstream
  `eb3e586765236a808dc82eee85f2be47e11e48c6`.
- The published fork tip is
  `53813b3a76546eabf2d1260d89436be7492a52dd`.
- `cfee612a` is the explicit deployed-ZSA branch-identity isolation commit.

For librustzcash, **`eb3e5867…` is the last integrated upstream baseline**, not
the fork's published tip. Its ancestry is already in the fork, allowing Git to
merge subsequent upstream commits without replaying the original ZSA merge.
For each other dependency, identify the corresponding integrated upstream
parent from its own merge history; do not substitute the librustzcash baseline.

### Preparation and delta review

1. Start from the published integration history (or its verified descendant)
   in a clean dedicated branch/worktree. Leave dirty original clones alone.
   Read each repository's instructions before modifying it.
2. Fetch official upstream refs into an explicitly named remote. Record a
   full SHA for both the integrated upstream baseline and chosen release
   target in every affected repository. SDK crates can have separate release
   tags; verify that the selected set is dependency-compatible.
3. Check ancestry before assuming this is an incremental merge. Example
   read-only commands, run in the librustzcash repository after substituting
   the actual release SHA for `RELEASE_SHA`:

   ```sh
   git merge-base --is-ancestor eb3e586765236a808dc82eee85f2be47e11e48c6 RELEASE_SHA
   git log --oneline eb3e586765236a808dc82eee85f2be47e11e48c6..RELEASE_SHA
   git diff --stat eb3e586765236a808dc82eee85f2be47e11e48c6 RELEASE_SHA
   git diff eb3e586765236a808dc82eee85f2be47e11e48c6 RELEASE_SHA -- components/zcash_protocol zcash_primitives pczt
   ```

   If the ancestor check fails, investigate whether the target is a different
   release line or rewritten history before merging. A failed check is not
   permission to rebase away the integration.
4. Inventory the delta: consensus/encoding changes, cryptographic circuits
   and keys, public API changes, dependency/version changes, and wallet/schema
   changes. Review fork-only adaptations against the baseline separately;
   a whole-tree fork-versus-release diff mixes those adaptations with new
   upstream work and is not the release delta.
5. Capture existing compatibility fixtures and expected results before edits:
   deployed ZSA branch ID, transaction/note encodings, verifying key and old
   proof verification, and proof routing. Locate existing tests rather than
   replacing them with newly generated fixtures that could conceal a change.

### Merge and conflict policy

- Use ancestry-preserving merge commits for upstream integration. Do not
  squash/rebase the fork, restart from upstream and reapply ZSA, or cherry-pick
  the entire release delta in place of preserving its ancestry.
- Review each conflict using three facts: the integrated upstream behavior,
  the fork's intentional ZSA adaptation, and the release's new requirement.
  Do not blanket-select `ours` or `theirs`, even for manifests or lockfiles.
- Keep official NU7 changes on the ordinary-network path. Preserve the old
  ZSA behavior behind its explicit network/branch/mode selection, especially
  transaction version selection, codecs, signature hashes, note formats,
  circuit selection and proof creation/verification.
- Upstream enum matches, height comparisons and defaults need explicit
  review for `BranchId::Zsa`: semantic variant ordering must not accidentally
  make the ZSA network inherit official NU7 rules.
- In Orchard/Halo2, retain the existing API boundary that absorbed ZSA's extra
  type parameter. Adapt it once if upstream APIs change; do not spread a new
  generic/signature split through zkool and the SDK again.
- Keep a concise record in merge/PR descriptions of non-obvious conflict
  resolutions and their compatibility tests. Discuss any proposed revert or
  deployed-protocol alteration with the user before proceeding.
- Local Git conflict reuse (`rerere`) may help repeated trial merges, but
  review reused resolutions; it is not evidence of protocol compatibility.
  Do not enable global settings as an incidental implementation step.

### Publish and validate in dependency order

Determine the order from the actual Cargo graph: merge/check cryptographic
dependencies first, then their SDK dependents, then trees/voting consumers and
zkool. Avoid assuming all eight repositories need code changes. Publish
dependency revisions before pinning them in dependent repositories, and
preserve exact SHA pins rather than replacing them with floating branches.

When pre-release crate versions become stable, update workspace versions,
direct requirements and patches consistently. Regenerate lockfiles through
Cargo and inspect the graph for duplicate/incompatible sources and accidental
registry fallbacks. Final consumer validation must use the published Git pins
with `--locked`, not just local path overrides. Keep the known standalone
Zakura/voting resolution issue separate unless its repair is explicitly scoped.

Acceptance gates:

- Rerun the compatibility tests recorded above, including verification of the
  old deployed ZSA proof and unchanged expected verifying-key/codec results.
- Test ordinary NU7 behavior against the final protocol and the updated
  isolated regtest, including activation-boundary behavior; keep pre-NU7 and
  NU6.3 migration coverage.
- Check SDK feature combinations and consumer builds, including Ledger/FROST,
  following repository-specific test instructions.
- Have the user repeat deployed ZSA sync/transfer as a regression check;
  do not broadcast on their behalf without authorization.
- Record the new upstream baselines, fork tips, lockfile pins, test results
  and remaining gaps here or in the release PR so the next merge is again
  a small, auditable delta.

## Deferred NU7 regtest work

The existing regtest setup already provides Zebra, lightwalletd and wallet
tests. Extend it when a stable Zebra release supporting official NU7 is ready;
do not create an unrelated replacement setup.

Files to revisit:

- `.github/actions/install-regtest-tools/action.yml`: current Zebra tag is
  `v6.2.1`; installation uses `--locked` and `--features=internal-miner`.
  The cache key includes the Zebra version, so updating the pin changes it.
- `misc/zebra.toml`: current activation overrides are NU6.2 at 1 and NU6.3
  at 250; no NU7 override.
- `rust/src/api/coin.rs`: Coin 2's NU7 activation must match Zebra exactly.
- `example/sh/setup-regtest.sh`: its missing-tool installation hint still
  mentions the older `v4.2.0`; align it with the eventual chosen stable pin.
- `tests/tests/test_migration.py`: preserves a funding phase before NU6.3,
  with default NU6.3 height 250. Choose NU7 timing without breaking that
  fixture. Its comment describing ZSA as NU7 is historical and should be
  clarified when updating the tests.
- `.github/actions/zebra/action.yml` and wallet/migration/Ledger/voting
  workflows: inspect their bootstrap and mining assumptions.

Resume sequence:

1. Check official Zebra releases for a stable NU7-capable tag; review its
   consensus and database changes against the SDK baseline above.
2. Update the shared pin and local installation guidance. Keep internal mining.
3. Choose an explicit official NU7 height after the existing NU6.3/funding
   phases; set the same height in Zebra and Coin 2. No height was chosen today.
4. Review accelerated-network subsidy validation and any required
   `slow_start_interval = 0` setting. Configure NSM reissuance separately if
   testing that feature; do not copy public-testnet heights into this regtest.
5. Use a fresh, isolated ordinary-regtest state/fixture if consensus changes
   invalidate the old one. Do not alter the deployed ZSA chain or user wallets;
   confirm exact targets before deleting any cached state.
6. Build and run existing CI regtest coverage plus explicit tests crossing
   official NU7 activation, syncing and transferring afterward. Keep the
   pre-NU6.3 Orchard-to-Ironwood migration coverage intact.
7. Recheck deployed ZSA isolation. Any further SDK update should preserve the
   history and integrate only the upstream delta since this baseline.

Reference checked today:
[Zebra v7.0.0-rc.0 release notes](https://github.com/ZcashFoundation/zebra/releases/tag/v7.0.0-rc.0)
confirm official NU7 regtest support, branch ID `0x77190ad9`, configurable NSM
parameters and stricter accelerated-network schedule validation.
[Zebra regtest documentation](https://zebra.zfnd.org/user/regtest.html) includes
an NU7/reissuance example. The RC is evidence of support, not our selected pin.
Recheck the documentation against the selected stable tag when resuming.

## Workspace and signing notes

At handoff creation, existing uncommitted Flutter-generated changes remain in:

- `linux/flutter/generated_plugins.cmake`
- `macos/Flutter/GeneratedPluginRegistrant.swift`
- `windows/flutter/generated_plugins.cmake`

Existing untracked data/tool/config directories also remain. They were not
part of the integration commit and must not be swept into a future commit or
deleted as cleanup. This handoff document itself is newly created and uncommitted.

The VS Code extension did not inherit the terminal's signing agent. The user
provided the working socket; signed commits worked with the per-command
environment setting:

```sh
SSH_AUTH_SOCK=/Users/hanhhuynhhuu/.gnupg/S.gpg-agent.ssh git commit ...
```

Verify the socket/agent again when needed. Do not change global Git settings or
re-sign/force-update merged history without an explicit request.
