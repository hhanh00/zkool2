# ADR-0001: Static swap asset list

**Status**: Accepted

## Decision

The swap UI's supported asset list is a hardcoded Dart enum. It is not loaded dynamically from the 1Click API.

## Context

The 1Click (`/v0/tokens`) endpoint returns many tokens, most of which are noise for a ZEC wallet. The supported set is deliberately small: major assets on major chains (USDT, USDC, SOL, ETH).

## Reasons

- The assets in scope are protocol-stable. Their decimals and existence are effectively immutable (ETH has been 18 decimals since 2015; BTC, SOL, USDT, USDC are analogously stable).
- The live list has too many useless tokens to display raw. Any filter strategy (by blockchain, by price presence, etc.) requires its own maintenance and can drift.
- Remote config — fetching a curated list from a URL or GraphQL binary — adds complexity (offline fallback, privacy implications of outbound calls, a maintained URL) that is not proportionate to the problem.
- Adding a new asset is a one-line enum change; the live API already resolves the correct `assetId` and authoritative decimals at quote time.

## What this does not preclude

Adding more assets to the enum as NEAR Intents supports more valuable tokens. That remains the right path: deliberate, explicit, code-reviewed.
