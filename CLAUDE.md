# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rules

- **Never commit macOS-specific files**: Do NOT commit files under `macos/` (e.g., `macos/Runner.xcodeproj/project.pbxproj`, `macos/Runner.xcworkspace/`). These are local development artifacts that should not be checked into the repository.
- Generated files must not be edited by hand: `frb_generated.rs`, `freezed.dart`, Riverpod `.g.dart` files — regenerate with codegen instead.
- Do not update code until instructed
- Do not commit or push unless instructed.
- To regenerate flutter_rust_bridge bindings:
  ```
  flutter_rust_bridge_codegen generate
  ```
- **Never edit `.cargo/git` checkouts**. Always edit a local git clone and use a `[patch]` path override in `Cargo.toml`.
- **If no local clone exists**, create one and add a path override. Do not create a new clone if one already exists in `Cargo.toml`.
