# Hopper

Keyboard shortcuts for macOS: switch apps with one key, and use Caps Lock and H / J / K / L as arrow keys.

## Keys

| Keys | Does |
|---|---|
| Right Command + a letter | Brings the app whose name starts with that letter to the front. Press again to cycle through apps with the same letter. |
| Caps Lock | Control |
| Control + H / J / K / L | Left / Down / Up / Right arrow. Shift, Option and Command still work with them. |
| Control + Command + H / L | Move one Space left / right |

### App switching

- Only regular apps (with a Dock icon) count.
- Right Command + a letter or digit always goes to Hopper, even if no app matches. Apps never see it, so their shortcuts cannot fire from the right Command key.
- Left Command shortcuts are not changed.

### Remaps

- H / J / K / L are found by key position on a US (QWERTY) layout.
- Control + H / J / K / L no longer reach apps. In a terminal, Control + H, K and L lose their usual meaning.
- Space moves use the macOS shortcut Control + Left / Right arrow. Keep it on in System Settings > Keyboard > Keyboard Shortcuts > Mission Control.
- macOS forgets the Caps Lock change on restart. Hopper sets it again each time it starts.

## Install

Needs [mise](https://mise.jdx.dev). It installs Rust for you.

```sh
mise run install
```

This builds `Hopper.app`, puts it in `~/Applications` and starts it now and at each login. It runs in the background, with no Dock icon.

macOS then asks for Accessibility access. Allow Hopper in System Settings > Privacy & Security > Accessibility. Hopper starts working when you do.

Access stays after a reinstall. If Hopper is in the list but does not work, clear its access and allow it again:

```sh
tccutil reset Accessibility com.github.mattcookio.hopper
launchctl kickstart -k gui/$(id -u)/com.github.mattcookio.hopper
```

Log: `~/Library/Logs/Hopper.log`.

## Remove

```sh
mise run uninstall
```

This stops Hopper, removes the app and its login item, and gives Caps Lock back.

## Run from the terminal

```sh
cargo run --release
```

This needs Accessibility access for your terminal app.

## Test

```sh
cargo test
```
