#!/bin/bash

FEATURE_NAME=$1

cat <<EOF
cargo:
  release:
    extra_flags:
      - features=$FEATURE_NAME
EOF
