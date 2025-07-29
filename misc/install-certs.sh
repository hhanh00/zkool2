#!/bin/sh
# KEYCHAIN_PWD $PATH $PASSWORD

security create-keychain -p $1 $HOME/build.keychain
security set-keychain-settings -ut 7200 $HOME/build.keychain
security default-keychain -s $HOME/build.keychain
security unlock-keychain -p $1 $HOME/build.keychain
security import $2 -k $HOME/build.keychain -P $3 -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $1 $HOME/build.keychain

