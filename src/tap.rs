//! Watches key presses through a macOS event tap.
//!
//! The tap sits at the hardware level, so it sees keys before apps, app
//! shortcuts and other event taps. Each key press and release goes to the
//! handler, which says whether to let it through, drop it or replace it.

use std::ffi::c_void;
use std::time::Duration;

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_foundation::string::{CFString, CFStringRef};

use crate::keys::{Action, Key};

type EventRef = *mut c_void;
type MachPortRef = *mut c_void;
type RunLoopSourceRef = *mut c_void;
type TapCallback = extern "C" fn(*mut c_void, u32, EventRef, *mut c_void) -> EventRef;

const HID_TAP: u32 = 0;
const HEAD_INSERT: u32 = 0;
const TAP_DEFAULT: u32 = 0;
const KEY_DOWN: u32 = 10;
const KEY_UP: u32 = 11;
const TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
const TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;
const FIELD_AUTOREPEAT: u32 = 8;
const FIELD_KEYCODE: u32 = 9;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events: u64,
        callback: TapCallback,
        user_info: *mut c_void,
    ) -> MachPortRef;
    fn CGEventTapEnable(tap: MachPortRef, enable: bool);
    fn CGEventGetFlags(event: EventRef) -> u64;
    fn CGEventSetFlags(event: EventRef, flags: u64);
    fn CGEventSetIntegerValueField(event: EventRef, field: u32, value: i64);
    fn CGEventCreateKeyboardEvent(source: *const c_void, code: u16, down: bool) -> EventRef;
    fn CGEventGetIntegerValueField(event: EventRef, field: u32) -> i64;
    fn CGEventKeyboardGetUnicodeString(
        event: EventRef,
        max_len: usize,
        actual_len: *mut usize,
        chars: *mut u16,
    );
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFMachPortCreateRunLoopSource(
        alloc: *const c_void,
        port: MachPortRef,
        order: isize,
    ) -> RunLoopSourceRef;
}

/// Called for each key press and release. Returns what to do with it.
pub type Handler = Box<dyn FnMut(&Key) -> Action>;

struct State {
    port: MachPortRef,
    handler: Handler,
}

/// Returns true if the app may use the event tap. If `prompt` is true and it
/// may not, shows the system prompt that asks for Accessibility access.
pub fn has_accessibility_access(prompt: bool) -> bool {
    let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let value = if prompt {
        CFBoolean::true_value()
    } else {
        CFBoolean::false_value()
    };
    let options = CFDictionary::from_CFType_pairs(&[(key, value)]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as *const c_void) }
}

/// Installs the tap on the current run loop and runs the loop forever.
///
/// macOS refuses the tap until the app has Accessibility access, so this
/// tries again each second until it may.
pub fn run(handler: Handler) {
    let state = Box::into_raw(Box::new(State {
        port: std::ptr::null_mut(),
        handler,
    }));
    unsafe {
        let port = loop {
            let port = CGEventTapCreate(
                HID_TAP,
                HEAD_INSERT,
                TAP_DEFAULT,
                (1 << KEY_DOWN) | (1 << KEY_UP),
                on_event,
                state as *mut c_void,
            );
            if !port.is_null() {
                break port;
            }
            std::thread::sleep(Duration::from_secs(1));
        };
        (*state).port = port;
        println!("Hopper is running. Hold right Command and press a letter.");

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), port, 0);
        let run_loop = CFRunLoop::get_current();
        core_foundation::runloop::CFRunLoopAddSource(
            run_loop.as_concrete_TypeRef(),
            source as _,
            kCFRunLoopCommonModes,
        );
        CGEventTapEnable(port, true);
    }
    CFRunLoop::run_current();
}

extern "C" fn on_event(
    _: *mut c_void,
    kind: u32,
    event: EventRef,
    user_info: *mut c_void,
) -> EventRef {
    let state = unsafe { &mut *(user_info as *mut State) };

    // macOS turns the tap off if a callback is slow. Turn it back on.
    if kind == TAP_DISABLED_BY_TIMEOUT || kind == TAP_DISABLED_BY_USER_INPUT {
        unsafe { CGEventTapEnable(state.port, true) };
        return event;
    }
    if kind != KEY_DOWN && kind != KEY_UP {
        return event;
    }

    let key = unsafe {
        Key {
            code: CGEventGetIntegerValueField(event, FIELD_KEYCODE) as u16,
            flags: CGEventGetFlags(event),
            down: kind == KEY_DOWN,
            repeat: CGEventGetIntegerValueField(event, FIELD_AUTOREPEAT) != 0,
            letter: letter_of(event),
        }
    };
    match (state.handler)(&key) {
        Action::Pass => event,
        Action::Swallow | Action::SwitchApp(_) => std::ptr::null_mut(),
        // macOS releases the returned event along with the original.
        Action::Send { code, flags } => unsafe {
            let new = CGEventCreateKeyboardEvent(std::ptr::null(), code, key.down);
            CGEventSetFlags(new, flags);
            CGEventSetIntegerValueField(new, FIELD_AUTOREPEAT, key.repeat as i64);
            new
        },
    }
}

fn letter_of(event: EventRef) -> Option<char> {
    let mut buf = [0u16; 4];
    let mut len = 0usize;
    unsafe { CGEventKeyboardGetUnicodeString(event, buf.len(), &mut len, buf.as_mut_ptr()) };
    let c = char::decode_utf16(buf[..len].iter().copied())
        .next()?
        .ok()?;
    c.is_alphanumeric()
        .then(|| c.to_lowercase().next())
        .flatten()
}
