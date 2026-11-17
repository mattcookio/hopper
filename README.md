# Hopper

Switch apps from the keyboard on macOS.

Hold the **right Command** key and press a letter. Hopper brings the app whose name starts with that letter to the front. Press again to cycle through apps with the same letter.

## Install

```sh
mise run install
```

This builds `Hopper.app`, puts it in `~/Applications` and starts it now and at each login. It runs in the background, with no Dock icon.

macOS then asks for Accessibility access. Allow Hopper in System Settings > Privacy & Security > Accessibility. Hopper starts working when you do.

Access stays after a reinstall. If Hopper is in the list but does not work, remove it from the list and allow it again.

Log: `~/Library/Logs/Hopper.log`. To remove Hopper: `mise run uninstall`.

## Run from the terminal

```sh
cargo run --release
```

This needs Accessibility access for your terminal app.

## Notes

- Only regular apps (with a Dock icon) count.
- Right Command + a letter or digit always goes to Hopper, even if no app matches. Apps never see it, so their shortcuts cannot fire from the right Command key.
- Left Command shortcuts are not changed.
