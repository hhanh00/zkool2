---
title: Zcash Shielded Assets
---

# Zcash Shielded Assets

Zcash Shielded Assets (ZSA) allow users to issue their own custom assets on
Zcash, stored in the Orchard shielded pool. Implemented per ZIP-226
(issuance keys) and ZIP-227 (shielded assets), ZSA lets anyone issue a
confidential asset, transfer it between shielded addresses, and optionally
burn it.

Issuance notes are published as plaintext (not encrypted) so that anyone can
verify the total supply of an asset. Each asset is
identified by an asset ID derived from the issuer's key and a
BLAKE2b-256 hash of the asset description (name).

::: important
The goal of the ZSA effort is to gauge real interest in shielded assets, and
real interest shows in participation: an application built, an asset issued, a
transaction sent. Each of those is worth more than any number of social media
likes, and each makes the case that shielded assets are wanted. The ecosystem
grows with every project that builds on it.
:::

## Use Cases

- **Project tokens**: a project or DAO issues its own token and distributes it
  to holders, who can send it between shielded addresses with the same privacy
  as ZEC.
- **Stablecoins**: a trusted issuer backs each unit with reserves and publishes
  issuance in plaintext, so anyone can audit that the total supply never
  exceeds the reserves.
- **Loyalty points**: a business issues reward points as an asset and lets
  customers hold, transfer, or redeem them privately.
- **Tokenized real-world assets**: ownership of an asset (e.g. a security or
  collectible) is represented on-chain; the issuer can finalize the asset to
  guarantee no further units can ever be issued.

## Tokenomics

- **Supply**: the total supply is set by the issuer at issuance time. Because
  issuance notes are plaintext, the circulating supply is always publicly
  verifiable on-chain, even though individual transfers stay shielded.
- **Capped supply**: finalizing an asset permanently disables further
  issuance, locking the supply and guaranteeing scarcity. Until finalized, the
  issuer can print additional units at will.
- **Burning**: assets can be permanently destroyed via the `assetBurn`
  mechanism (defined in ZIP-226). A transaction declares the asset and amount
  to burn; the protocol verifies it against the tracked circulating supply and
  decrements it, so burnt value is gone forever. Burning is the recommended way
  to off-board and redeem bridged assets.
- **Fees**: transaction fees on Zcash are always paid in ZEC, never in the
  custom asset itself. Holding or transferring an asset costs ZEC for fees, so
  users need some ZEC alongside their asset holdings.
- **Value**: ZSA imposes no economic policy. An asset's value is entirely
  market-determined (backing, demand, and issuer reputation). The protocol only
  guarantees supply transparency and, after finalization, supply cap.

## Implementation Notes

- **Fees**: fees are not yet fully implemented per the spec. The cost to issue
  an asset should be higher.
- **Finalization**: assets are always finalized for now; reissuance is not yet
  supported.
- **Burning**: burning assets is not implemented.
