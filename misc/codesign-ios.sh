#!/bin/bash
pushd misc
openssl enc -pbkdf2 -aes-256-cbc -salt -d -in zkool.mobileprovision.enc -out zkool.mobileprovision -pass pass:$APPLE_DEV_CERT_PWD
openssl enc -pbkdf2 -aes-256-cbc -salt -d -in apple.distrib.p12.enc -out apple.distrib.p12 -pass pass:$APPLE_DEV_CERT_PWD

security create-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security set-keychain-settings -ut 7200 $HOME/build.keychain
security default-keychain -s $HOME/build.keychain
security unlock-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security import apple.distrib.p12 -k $HOME/build.keychain -P $APPLE_DEV_CERT_PWD -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $KEYCHAIN_PWD $HOME/build.keychain

mkdir -p ~/Library/MobileDevice/Provisioning\ Profiles
cp zkool.mobileprovision ~/Library/MobileDevice/Provisioning\ Profiles/

popd
