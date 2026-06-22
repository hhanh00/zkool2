# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rules

- **Never commit macOS-specific files**: Do NOT commit files under `macos/` (e.g., `macos/Runner.xcodeproj/project.pbxproj`, `macos/Runner.xcworkspace/`). These are local development artifacts that should not be checked into the repository.
- Generated files must not be edited by hand: `frb_generated.rs`, `freezed.dart`, Riverpod `.g.dart` files — regenerate with codegen instead.
- Do not commit or push unless instructed.
- **RUSTFLAGS**: Building requires `RUSTFLAGS='--cfg zcash_unstable="nu7"'` for NU7 consensus support. This applies to `cargo build`, `cargo check`, and `flutter_rust_bridge_codegen`.
- To regenerate flutter_rust_bridge bindings:
  ```
  RUSTFLAGS='--cfg zcash_unstable="nu7"' flutter_rust_bridge_codegen generate
  ```
