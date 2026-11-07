#!/usr/bin/env bash
# Fails if any ELF file in a Linux bundle needs a newer glibc or libstdc++ than
# the release promises to support. Usage: check-bundle-abi.sh <bundle-dir>
set -euo pipefail

MAX_GLIBC=2.35      # Ubuntu 22.04, Debian 12 is 2.36
MAX_GLIBCXX=3.4.30  # libstdc++ from GCC 12

bundle=${1:?usage: check-bundle-abi.sh <bundle-dir>}
status=0
checked=0

# True if version $1 is newer than version $2.
newer() {
  [ "$1" != "$2" ] && [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1)" = "$1" ]
}

# Highest version of symbol prefix $2 listed in the version references $1.
highest() {
  { grep -oE "\b$2_[0-9][0-9.]*" <<<"$1" || true; } | sed "s/^$2_//" | sort -V | tail -1
}

while IFS= read -r -d '' file; do
  [ "$(head -c4 "$file" | tr -d '\0' | tail -c3)" = "ELF" ] || continue
  checked=$((checked + 1))

  refs=$(objdump -p "$file")
  glibc=$(highest "$refs" GLIBC)
  glibcxx=$(highest "$refs" GLIBCXX)
  printf '%-40s GLIBC %-8s GLIBCXX %s\n' "$(basename "$file")" "${glibc:--}" "${glibcxx:--}"

  if [ -n "$glibc" ] && newer "$glibc" "$MAX_GLIBC"; then
    echo "::error::$file needs GLIBC_$glibc, above the $MAX_GLIBC limit"
    status=1
  fi
  if [ -n "$glibcxx" ] && newer "$glibcxx" "$MAX_GLIBCXX"; then
    echo "::error::$file needs GLIBCXX_$glibcxx, above the $MAX_GLIBCXX limit"
    status=1
  fi
done < <(find "$bundle" -type f -print0)

# An empty scan must not read as a pass.
if [ "$checked" -eq 0 ]; then
  echo "::error::no ELF files found in $bundle"
  exit 1
fi

exit "$status"
