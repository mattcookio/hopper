//! Hopper: hold the right Command key and press a letter to bring the app
//! whose name starts with that letter to the front. Press again to cycle.

mod pick;
mod tap;

use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSApplicationActivationOptions, NSApplicationActivationPolicy, NSWorkspace};

use pick::{App, pick};

fn main() {
    if !tap::has_accessibility_access() {
        eprintln!(
            "Hopper needs Accessibility access.\n\
             Allow it in System Settings > Privacy & Security > Accessibility, then run Hopper again."
        );
        std::process::exit(1);
    }

    println!("Hopper is running. Hold right Command and press a letter.");
    if let Err(err) = tap::run(Box::new(switch_to)) {
        eprintln!("Hopper: {err}");
        std::process::exit(1);
    }
}

/// Brings the app for `letter` to the front. Returns false if no app matches.
fn switch_to(letter: char) -> bool {
    autoreleasepool(|_| {
        let workspace = NSWorkspace::sharedWorkspace();
        let running = workspace.runningApplications();

        let apps: Vec<App> = running
            .iter()
            .filter(|app| app.activationPolicy() == NSApplicationActivationPolicy::Regular)
            .filter_map(|app| {
                let name = app.localizedName()?.to_string();
                Some(App {
                    pid: app.processIdentifier(),
                    name,
                })
            })
            .collect();
        let frontmost = workspace
            .frontmostApplication()
            .map(|app| app.processIdentifier());

        let Some(target) = pick(&apps, letter, frontmost) else {
            return false;
        };
        if let Some(app) = running
            .iter()
            .find(|app| app.processIdentifier() == target.pid)
        {
            app.unhide();
            app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
        }
        true
    })
}
