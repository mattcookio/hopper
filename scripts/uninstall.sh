#!/bin/sh
# Stops Hopper and removes the app and its login item.
set -eu
label=com.github.mattcookio.hopper
launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
rm -f "$HOME/Library/LaunchAgents/$label.plist"
rm -rf "$HOME/Applications/Hopper.app"
echo "Hopper removed."
