#!/bin/bash
echo "$APPLE_CERTIFICATE" | base64 --decode > /tmp/certs.p12
security create-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security set-keychain-settings -ut 7200 $HOME/build.keychain
security default-keychain -s $HOME/build.keychain
security unlock-keychain -p $KEYCHAIN_PWD $HOME/build.keychain
security import /tmp/certs.p12 -k $HOME/build.keychain -P $APPLE_CERTIFICATE_PASSWORD -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $KEYCHAIN_PWD $HOME/build.keychain
/usr/bin/codesign --force --entitlements macos/Runner/Release.entitlements -s "$APPLE_CERTIFICATE_NAME" --deep --options runtime build/macos/Build/Products/Release/$APPNAME.app -v
xcrun notarytool store-credentials "notarytool-profile" --apple-id $APPLE_ID --team-id $APPLE_TEAM_ID --password $APPLE_PASSWORD
ditto -c -k --keepParent "build/macos/Build/Products/Release/$APPNAME.app" "notarization.zip"
xcrun notarytool submit "notarization.zip" --keychain-profile "notarytool-profile" --wait
xcrun stapler staple "build/macos/Build/Products/Release/$APPNAME.app"
