# Hopper

Switch apps from the keyboard on macOS.

Hold the **right Command** key and press a letter. Hopper brings the app whose name starts with that letter to the front. Press again to cycle through apps with the same letter.

## Run

```sh
cargo run --release
```

On first run, macOS asks for Accessibility access. Allow it for your terminal in System Settings > Privacy & Security > Accessibility, then run Hopper again.

## Notes

- Only regular apps (with a Dock icon) count.
- If no app matches the letter, the key press goes through as normal.
- Left Command shortcuts are not changed.
