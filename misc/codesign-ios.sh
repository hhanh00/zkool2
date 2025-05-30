#!/bin/bash
echo "$APPLE_DEV_CERT" | base64 --decode > /tmp/certs.p12
security create-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security set-keychain-settings -ut 7200 $HOME/build.keychain
security default-keychain -s $HOME/build.keychain
security unlock-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security import /tmp/certs.p12 -k $HOME/build.keychain -P $APPLE_DEV_CERT_PWD -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $KEYCHAIN_PWD $HOME/build.keychain

mkdir -p ~/Library/MobileDevice/Provisioning\ Profiles
echo "$IOS_PROVISION" | base64 --decode > ~/Library/MobileDevice/Provisioning\ Profiles/$UUID.mobileprovision
