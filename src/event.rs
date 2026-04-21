use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

/// Poll for a terminal event with a timeout.
pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Helper to check for common quit keybindings.
pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(
        (key.code, key.modifiers),
        (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn quit_on_q() {
        assert!(is_quit(&key_event(KeyCode::Char('q'), KeyModifiers::NONE)));
    }

    #[test]
    fn quit_on_ctrl_c() {
        assert!(is_quit(&key_event(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        )));
    }

    #[test]
    fn not_quit_on_other_keys() {
        assert!(!is_quit(&key_event(KeyCode::Char('a'), KeyModifiers::NONE)));
        assert!(!is_quit(&key_event(KeyCode::Enter, KeyModifiers::NONE)));
        assert!(!is_quit(&key_event(KeyCode::Esc, KeyModifiers::NONE)));
    }

    #[test]
    fn not_quit_on_modified_q() {
        assert!(!is_quit(&key_event(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL
        )));
        assert!(!is_quit(&key_event(
            KeyCode::Char('q'),
            KeyModifiers::SHIFT
        )));
    }

    #[test]
    fn not_quit_on_plain_c() {
        assert!(!is_quit(&key_event(KeyCode::Char('c'), KeyModifiers::NONE)));
    }
}
