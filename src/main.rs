//! Hopper: hold the right Command key and press a letter to bring the app
//! whose name starts with that letter to the front. Press again to cycle.
//! It also remaps a few keys; see `keys` and `caps_lock`.

mod caps_lock;
mod keys;
mod pick;
mod tap;

use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSApplicationActivationOptions, NSApplicationActivationPolicy, NSWorkspace};

use keys::{Action, Router};
use pick::{App, pick};

fn main() {
    if !tap::has_accessibility_access(true) {
        eprintln!(
            "Hopper needs Accessibility access.\n\
             Allow it in System Settings > Privacy & Security > Accessibility. Hopper starts when you do."
        );
    }
    caps_lock::map_to_control();

    let mut router = Router::default();
    tap::run(Box::new(move |key| {
        let action = router.route(key);
        if let Action::SwitchApp(letter) = action {
            switch_to(letter);
        }
        action
    }));
}

/// Brings the app for `letter` to the front, if one matches.
fn switch_to(letter: char) {
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
            return;
        };
        if let Some(app) = running
            .iter()
            .find(|app| app.processIdentifier() == target.pid)
        {
            app.unhide();
            app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
        }
    })
}
