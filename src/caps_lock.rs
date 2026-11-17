//! Turns Caps Lock into Control for the whole system.
//!
//! macOS forgets this mapping on restart. Hopper starts at login, so it sets
//! it again each time.

use std::process::Command;

const MAPPING: &str = r#"{"UserKeyMapping":[{"HIDKeyboardModifierMappingSrc":0x700000039,"HIDKeyboardModifierMappingDst":0x7000000E0}]}"#;

pub fn map_to_control() {
    let result = Command::new("/usr/bin/hidutil")
        .args(["property", "--set", MAPPING])
        .output();
    match result {
        Ok(out) if out.status.success() => {}
        Ok(out) => eprintln!(
            "Hopper: could not map Caps Lock to Control: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(err) => eprintln!("Hopper: could not map Caps Lock to Control: {err}"),
    }
}
