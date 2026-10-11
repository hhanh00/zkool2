#!/usr/bin/env python3
"""Refresh or verify dependency hashes with Nix; release CI uses x86_64 Linux."""

from pathlib import Path
import argparse
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]
FLAKE = ROOT / "flake.nix"
FAKE_HASH = "sha256-" + "A" * 43 + "="
TARGETS = (
    ("pubCache =", "outputHash", "zkool-pub-cache"),
    ("cargo-vendor = pkgs.stdenv.mkDerivation", "outputHash", "cargo-vendor"),
    ("zkool-graphql = pkgs.rustPlatform.buildRustPackage", "cargoHash",
     "zkool-graphql.cargoDeps"),
)


def replace_hash(source, marker, attribute, value):
    start = source.index(marker)
    match = re.search(rf'\b{attribute} = "[^"]+";', source[start:])
    if match is None:
        raise RuntimeError(f"Cannot find {attribute} after {marker}")
    return (source[:start + match.start()] + f'{attribute} = "{value}";'
            + source[start + match.end():])


def build(target):
    result = subprocess.run(
        ["nix", "build", f".#{target}", "--no-link",
         "--no-update-lock-file"],
        cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    print(result.stdout, end="", flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true",
                        help="Verify existing hashes without modifying flake.nix")
    parser.add_argument("--pub", action="store_true", help="Refresh Dart dependencies")
    parser.add_argument("--cargo", action="store_true", help="Refresh both Rust dependency hashes")
    args = parser.parse_args()
    # No flags retains the manual refresh-all behavior.
    targets = TARGETS if not (args.pub or args.cargo) else tuple(
        entry for index, entry in enumerate(TARGETS)
        if (args.pub if index == 0 else args.cargo)
    )
    if args.verify:
        for _, _, target in targets:
            print(f"Verifying {target}", flush=True)
            if build(target).returncode != 0:
                raise RuntimeError(f"Verification failed for {target}")
        return
    original = FLAKE.read_text()
    source = original
    try:
        for marker, attribute, target in targets:
            print(f"Refreshing {target}", flush=True)
            FLAKE.write_text(replace_hash(source, marker, attribute, FAKE_HASH))
            result = build(target)
            hashes = re.findall(r"got:\s+(sha256-[A-Za-z0-9+/]{43}=)", result.stdout)
            if (result.returncode == 0 or len(hashes) != 1
                    or not re.search(r"specified:\s+" + re.escape(FAKE_HASH), result.stdout)
                    or "hash mismatch in fixed-output derivation" not in result.stdout):
                raise RuntimeError(f"Expected a dependency hash mismatch for {target}")
            source = replace_hash(source, marker, attribute, hashes[0])
            FLAKE.write_text(source)
    except BaseException:
        FLAKE.write_text(original)
        raise


if __name__ == "__main__":
    main()
