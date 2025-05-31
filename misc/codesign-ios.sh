#!/bin/bash
pushd ios
pushd certs
openssl enc -pbkdf2 -aes-256-cbc -salt -d -in zkool.mobileprovision.enc -out zkool.mobileprovision -pass pass:$APPLE_DEV_CERT_PWD
openssl enc -pbkdf2 -aes-256-cbc -salt -d -in apple-distrib.p12.enc -out apple-distrib.p12 -pass pass:$APPLE_DEV_CERT_PWD
popd

security create-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security set-keychain-settings -ut 7200 $HOME/build.keychain
security default-keychain -s $HOME/build.keychain
security unlock-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security import certs/apple-distrib.p12 -k $HOME/build.keychain -P $APPLE_DEV_CERT_PWD -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $KEYCHAIN_PWD $HOME/build.keychain

mkdir -p ~/Library/MobileDevice/Provisioning\ Profiles
cp certs/zkool3.mobileprovision ~/Library/MobileDevice/Provisioning\ Profiles/

popd

# Request a IOS distribution certificate from the Apple Developer Portal
# Download the certificate and import it into the keychain
# Export the certificate and private key as a .p12 file
# Encode the .p12 file in base64 and set it as the APPLE_DEV_CERT repository secret
# Request a provisioning profile from the Apple Developer Portal with Apple Store Connect
# and include the previous certificate
# Download the provisioning profile and encode it in base64
# Set the provisioning profile as the IOS_PROVISION repository secret
