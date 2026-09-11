//! One shared keyboard-to-text helper for the castle's typed modals.
//!
//! Encounters, the manifestation conduit, and the Architect workshop all capture the same
//! printable keys. Two private copies of this table had already drifted apart — one accepted
//! punctuation the other did not, one capped by bytes and the other by characters — so this is
//! now the single source of truth. Capitals are deliberately not supported: the existing modals
//! were lowercase-only, and inventing shift handling here would change their behavior silently.

use bevy::prelude::*;

const TYPED_KEYS: [(KeyCode, char); 42] = [
    (KeyCode::KeyA, 'a'), (KeyCode::KeyB, 'b'), (KeyCode::KeyC, 'c'), (KeyCode::KeyD, 'd'),
    (KeyCode::KeyE, 'e'), (KeyCode::KeyF, 'f'), (KeyCode::KeyG, 'g'), (KeyCode::KeyH, 'h'),
    (KeyCode::KeyI, 'i'), (KeyCode::KeyJ, 'j'), (KeyCode::KeyK, 'k'), (KeyCode::KeyL, 'l'),
    (KeyCode::KeyM, 'm'), (KeyCode::KeyN, 'n'), (KeyCode::KeyO, 'o'), (KeyCode::KeyP, 'p'),
    (KeyCode::KeyQ, 'q'), (KeyCode::KeyR, 'r'), (KeyCode::KeyS, 's'), (KeyCode::KeyT, 't'),
    (KeyCode::KeyU, 'u'), (KeyCode::KeyV, 'v'), (KeyCode::KeyW, 'w'), (KeyCode::KeyX, 'x'),
    (KeyCode::KeyY, 'y'), (KeyCode::KeyZ, 'z'),
    (KeyCode::Space, ' '),
    (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'), (KeyCode::Digit3, '3'),
    (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'), (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'),
    (KeyCode::Digit8, '8'), (KeyCode::Digit9, '9'),
    (KeyCode::Comma, ','), (KeyCode::Period, '.'), (KeyCode::Quote, '\''), (KeyCode::Minus, '-'),
    (KeyCode::Slash, '/'),
];

/// Applies this frame's printable keypresses and backspace to `buffer`.
///
/// The cap counts characters, never bytes, so a cap can never split a multi-byte character.
pub fn apply_typed_keys(keyboard: &ButtonInput<KeyCode>, buffer: &mut String, max_chars: usize) {
    if keyboard.just_pressed(KeyCode::Backspace) {
        buffer.pop();
    }
    for (key, character) in TYPED_KEYS {
        if keyboard.just_pressed(key) && buffer.chars().count() < max_chars {
            buffer.push(character);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(keys: &[KeyCode]) -> ButtonInput<KeyCode> {
        let mut input = ButtonInput::default();
        for key in keys {
            input.press(*key);
        }
        input
    }

    #[test]
    fn typed_keys_append_in_table_order() {
        let mut buffer = String::new();
        apply_typed_keys(&press(&[KeyCode::KeyH]), &mut buffer, 16);
        apply_typed_keys(&press(&[KeyCode::KeyI]), &mut buffer, 16);
        assert_eq!(buffer, "hi");
    }

    #[test]
    fn backspace_removes_one_character() {
        let mut buffer = "plan".to_owned();
        apply_typed_keys(&press(&[KeyCode::Backspace]), &mut buffer, 16);
        assert_eq!(buffer, "pla");
    }

    #[test]
    fn the_cap_stops_further_input_without_truncating_what_is_there() {
        let mut buffer = "abc".to_owned();
        apply_typed_keys(&press(&[KeyCode::KeyD]), &mut buffer, 3);
        assert_eq!(buffer, "abc");
    }

    #[test]
    fn every_table_key_is_unique() {
        let mut keys: Vec<KeyCode> = TYPED_KEYS.iter().map(|(key, _)| *key).collect();
        let before = keys.len();
        keys.sort_by_key(|key| format!("{key:?}"));
        keys.dedup_by_key(|key| format!("{key:?}"));
        assert_eq!(keys.len(), before);
    }
}
