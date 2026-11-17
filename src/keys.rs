//! Decides what each key press does. Knows nothing about macOS events, so
//! every rule can be tested.
//!
//! - Right Command + letter or digit: switch to an app.
//! - Control + Command + H / L: move one Space left / right.
//! - Control + H / J / K / L: arrow keys. Shift, Option and Command still apply.

use std::collections::HashMap;

/// Modifier bits as macOS reports them in event flags.
pub mod flag {
    #[cfg(test)]
    pub const SHIFT: u64 = 0x0002_0000;
    pub const CONTROL: u64 = 0x0004_0000;
    #[cfg(test)]
    pub const OPTION: u64 = 0x0008_0000;
    pub const COMMAND: u64 = 0x0010_0000;
    pub const NUMPAD: u64 = 0x0020_0000;
    pub const FN: u64 = 0x0080_0000;
    pub const LEFT_CONTROL: u64 = 0x0001;
    pub const RIGHT_CONTROL: u64 = 0x2000;
    pub const LEFT_COMMAND: u64 = 0x0008;
    pub const RIGHT_COMMAND: u64 = 0x0010;
    /// Set on every real key event.
    pub const NON_COALESCED: u64 = 0x0100;
}

/// Key codes for physical key positions.
pub mod code {
    pub const H: u16 = 4;
    pub const J: u16 = 38;
    pub const K: u16 = 40;
    pub const L: u16 = 37;
    pub const LEFT: u16 = 123;
    pub const RIGHT: u16 = 124;
    pub const DOWN: u16 = 125;
    pub const UP: u16 = 126;
}

/// One key press or release.
#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: u16,
    pub flags: u64,
    pub down: bool,
    pub repeat: bool,
    /// The lowercase letter or digit the key types, if any.
    pub letter: Option<char>,
}

/// What to do with a key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Let the event through unchanged.
    Pass,
    /// Drop the event.
    Swallow,
    /// Drop the event and switch to the app for this letter.
    SwitchApp(char),
    /// Replace the event with this key, pressed or released as the original was.
    Send { code: u16, flags: u64 },
}

/// Turns key events into actions. Remembers what each held key became, so its
/// release matches its press even if modifiers changed in between.
#[derive(Default)]
pub struct Router {
    held: HashMap<u16, Action>,
}

impl Router {
    pub fn route(&mut self, key: &Key) -> Action {
        if !key.down {
            return match self.held.remove(&key.code) {
                Some(Action::Send { code, flags }) => Action::Send { code, flags },
                Some(_) => Action::Swallow,
                None => Action::Pass,
            };
        }
        let action = press(key);
        if action != Action::Pass {
            self.held.insert(key.code, action);
        }
        action
    }
}

fn press(key: &Key) -> Action {
    let control = key.flags & flag::CONTROL != 0;
    let command = key.flags & flag::COMMAND != 0;

    if control && command {
        let space = match key.code {
            code::H => Some(code::LEFT),
            code::L => Some(code::RIGHT),
            _ => None,
        };
        if let Some(code) = space {
            // macOS moves between Spaces on Control + Left / Right arrow.
            let flags = flag::CONTROL | flag::LEFT_CONTROL | flag::FN | flag::NUMPAD;
            return Action::Send {
                code,
                flags: flags | flag::NON_COALESCED,
            };
        }
    }

    if control && !command {
        let arrow = match key.code {
            code::H => Some(code::LEFT),
            code::J => Some(code::DOWN),
            code::K => Some(code::UP),
            code::L => Some(code::RIGHT),
            _ => None,
        };
        if let Some(code) = arrow {
            let without_control =
                key.flags & !(flag::CONTROL | flag::LEFT_CONTROL | flag::RIGHT_CONTROL);
            return Action::Send {
                code,
                flags: without_control | flag::FN | flag::NUMPAD,
            };
        }
    }

    let right_command_only =
        key.flags & flag::RIGHT_COMMAND != 0 && key.flags & flag::LEFT_COMMAND == 0;
    if right_command_only && let Some(letter) = key.letter {
        return if key.repeat {
            Action::Swallow
        } else {
            Action::SwitchApp(letter)
        };
    }

    Action::Pass
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTRL: u64 = flag::CONTROL | flag::LEFT_CONTROL;
    const RCMD: u64 = flag::COMMAND | flag::RIGHT_COMMAND;
    const LCMD: u64 = flag::COMMAND | flag::LEFT_COMMAND;
    const ARROW: u64 = flag::FN | flag::NUMPAD;

    fn down(code: u16, flags: u64, letter: Option<char>) -> Key {
        Key {
            code,
            flags,
            down: true,
            repeat: false,
            letter,
        }
    }

    fn up(code: u16, flags: u64) -> Key {
        Key {
            code,
            flags,
            down: false,
            repeat: false,
            letter: None,
        }
    }

    fn route(key: Key) -> Action {
        Router::default().route(&key)
    }

    #[test]
    fn control_hjkl_become_arrows() {
        for (from, to) in [
            (code::H, code::LEFT),
            (code::J, code::DOWN),
            (code::K, code::UP),
            (code::L, code::RIGHT),
        ] {
            assert_eq!(
                route(down(from, CTRL, None)),
                Action::Send {
                    code: to,
                    flags: ARROW
                }
            );
        }
    }

    #[test]
    fn arrows_keep_shift_and_option() {
        let flags = CTRL | flag::SHIFT | flag::OPTION;
        assert_eq!(
            route(down(code::L, flags, None)),
            Action::Send {
                code: code::RIGHT,
                flags: flag::SHIFT | flag::OPTION | ARROW
            }
        );
    }

    #[test]
    fn control_command_h_and_l_move_between_spaces() {
        let flags = CTRL | LCMD;
        let space = CTRL | ARROW | flag::NON_COALESCED;
        assert_eq!(
            route(down(code::H, flags, None)),
            Action::Send {
                code: code::LEFT,
                flags: space
            }
        );
        assert_eq!(
            route(down(code::L, flags, None)),
            Action::Send {
                code: code::RIGHT,
                flags: space
            }
        );
    }

    #[test]
    fn control_command_j_passes_through() {
        assert_eq!(route(down(code::J, CTRL | LCMD, None)), Action::Pass);
    }

    #[test]
    fn right_command_letter_switches_app() {
        assert_eq!(route(down(0, RCMD, Some('a'))), Action::SwitchApp('a'));
    }

    #[test]
    fn right_command_held_letter_does_not_switch_again() {
        let mut key = down(0, RCMD, Some('a'));
        key.repeat = true;
        assert_eq!(route(key), Action::Swallow);
    }

    #[test]
    fn left_command_letter_passes_through() {
        assert_eq!(route(down(0, LCMD, Some('a'))), Action::Pass);
    }

    #[test]
    fn plain_letters_pass_through() {
        assert_eq!(route(down(code::H, 0, Some('h'))), Action::Pass);
    }

    #[test]
    fn release_matches_press_after_control_is_let_go() {
        let mut router = Router::default();
        router.route(&down(code::H, CTRL, None));
        assert_eq!(
            router.route(&up(code::H, 0)),
            Action::Send {
                code: code::LEFT,
                flags: ARROW
            }
        );
        assert_eq!(router.route(&up(code::H, 0)), Action::Pass);
    }

    #[test]
    fn release_of_app_switch_key_is_swallowed() {
        let mut router = Router::default();
        router.route(&down(0, RCMD, Some('a')));
        assert_eq!(router.route(&up(0, 0)), Action::Swallow);
    }
}
