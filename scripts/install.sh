#!/bin/sh
# Builds Hopper.app, installs it in ~/Applications and starts it at login.
set -eu
cd "$(dirname "$0")/.."

label=com.github.mattcookio.hopper
app="$HOME/Applications/Hopper.app"
agent="$HOME/Library/LaunchAgents/$label.plist"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

cargo build --release

rm -rf "$app"
mkdir -p "$app/Contents/MacOS"
cp target/release/hopper "$app/Contents/MacOS/hopper"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key><string>$label</string>
  <key>CFBundleName</key><string>Hopper</string>
  <key>CFBundleExecutable</key><string>hopper</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSUIElement</key><true/>
</dict>
</plist>
PLIST
codesign --force --sign - "$app"

mkdir -p "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
cat > "$agent" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$label</string>
  <key>ProgramArguments</key><array><string>$app/Contents/MacOS/hopper</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$HOME/Library/Logs/Hopper.log</string>
  <key>StandardErrorPath</key><string>$HOME/Library/Logs/Hopper.log</string>
</dict>
</plist>
PLIST

launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$agent"
echo "Hopper installed and started. Log: ~/Library/Logs/Hopper.log"
