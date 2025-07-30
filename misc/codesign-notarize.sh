#!/bin/bash
/usr/bin/codesign --force --entitlements macos/Runner/Release.entitlements -s "$APPLE_CERTIFICATE_NAME" --deep --options runtime build/macos/Build/Products/Release/$APPNAME.app -v

xcrun notarytool store-credentials "notarytool-profile" --apple-id $APPLE_ID --team-id $APPLE_TEAM_ID --password $APPLE_PASSWORD
ditto -c -k --keepParent "build/macos/Build/Products/Release/$APPNAME.app" "notarization.zip"
xcrun notarytool submit "notarization.zip" --keychain-profile "notarytool-profile" --wait
xcrun stapler staple "build/macos/Build/Products/Release/$APPNAME.app"
