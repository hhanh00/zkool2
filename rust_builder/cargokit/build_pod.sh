#!/bin/sh
set -e

BASEDIR=$(dirname "$0")

# Workaround for https://github.com/dart-lang/pub/issues/4010
BASEDIR=$(cd "$BASEDIR" ; pwd -P)

# Remove XCode SDK from path. Otherwise this breaks tool compilation when building iOS project
NEW_PATH=`echo $PATH | tr ":" "\n" | grep -v "Contents/Developer/" | tr "\n" ":"`

export PATH=${NEW_PATH%?} # remove trailing :

env

# Platform name (macosx, iphoneos, iphonesimulator)
export CARGOKIT_DARWIN_PLATFORM_NAME=$PLATFORM_NAME

# Arctive architectures (arm64, armv7, x86_64), space separated.
export CARGOKIT_DARWIN_ARCHS=$ARCHS

# Current build configuration (Debug, Release)
export CARGOKIT_CONFIGURATION=$CONFIGURATION

# Path to directory containing Cargo.toml.
export CARGOKIT_MANIFEST_DIR=$PODS_TARGET_SRCROOT/$1

# Temporary directory for build artifacts.
export CARGOKIT_TARGET_TEMP_DIR=$TARGET_TEMP_DIR

# Output directory for final artifacts.
export CARGOKIT_OUTPUT_DIR=$PODS_CONFIGURATION_BUILD_DIR/$PRODUCT_NAME

# Directory to store built tool artifacts.
export CARGOKIT_TOOL_TEMP_DIR=$TARGET_TEMP_DIR/build_tool

# Directory inside root project. Not necessarily the top level directory of root project.
export CARGOKIT_ROOT_PROJECT_DIR=$SRCROOT

# Propagate Cargo rustflags for iOS builds:
# - If `CARGO_ENCODED_RUSTFLAGS` is already set in the environment, keep it.
# - Otherwise, try to read `.cargo/config.toml` in the manifest dir and extract
#   a `rustflags = [...]` entry, encoding the flags with the unit separator.
if [ -z "$CARGO_ENCODED_RUSTFLAGS" ]; then
  CONFIG_FILE="$PODS_TARGET_SRCROOT/$1/.cargo/config.toml"
  if [ -f "$CONFIG_FILE" ]; then
    # Extract the rustflags line and parse comma-separated items. This is a
    # lightweight parser that handles simple cases like: rustflags = ['--cfg', 'foo']
    RUSTFLAGS_LINE=$(sed -n "s/^[[:space:]]*rustflags[[:space:]]*=\(.*\)/\1/p" "$CONFIG_FILE" | tr -d '\r' | tr -d '\n' | sed "s/^\s*//;s/\s*$//")
    if [ -n "$RUSTFLAGS_LINE" ]; then
      # Remove surrounding brackets and quotes, then split on commas
      # Example input: ['--cfg', 'zcash_unstable="nu7"']
      RUSTFLAGS_CONTENT=$(echo "$RUSTFLAGS_LINE" | sed -e "s/^\[//" -e "s/\]$//" -e "s/\'\"\'//g")
      # Remove single quotes and double quotes, then split by comma
      # and trim whitespace
      IFS=',' read -ra PARTS <<< "$RUSTFLAGS_CONTENT"
      ENCODED=""
      SEP=$(printf "\x1f")
      for part in "${PARTS[@]}"; do
        # Trim spaces
        trimmed=$(echo "$part" | sed -e 's/^\s*//' -e 's/\s*$//')
        # Remove leading/trailing single or double quotes
        trimmed=$(echo "$trimmed" | sed -e "s/^['\"]//" -e "s/['\"]$//")
        if [ -n "$trimmed" ]; then
          if [ -z "$ENCODED" ]; then
            ENCODED="$trimmed"
          else
            ENCODED="$ENCODED$SEP$trimmed"
          fi
        fi
      done
      if [ -n "$ENCODED" ]; then
        export CARGO_ENCODED_RUSTFLAGS="$ENCODED"
      fi
    fi
  fi
fi

FLUTTER_EXPORT_BUILD_ENVIRONMENT=(
  "$PODS_ROOT/../Flutter/ephemeral/flutter_export_environment.sh" # macOS
  "$PODS_ROOT/../Flutter/flutter_export_environment.sh" # iOS
)

for path in "${FLUTTER_EXPORT_BUILD_ENVIRONMENT[@]}"
do
  if [[ -f "$path" ]]; then
    source "$path"
  fi
done

sh "$BASEDIR/run_build_tool.sh" build-pod "$@"

# Make a symlink from built framework to phony file, which will be used as input to
# build script. This should force rebuild (podspec currently doesn't support alwaysOutOfDate
# attribute on custom build phase)
ln -fs "$OBJROOT/XCBuildData/build.db" "${BUILT_PRODUCTS_DIR}/cargokit_phony"
ln -fs "${BUILT_PRODUCTS_DIR}/${EXECUTABLE_PATH}" "${BUILT_PRODUCTS_DIR}/cargokit_phony_out"
