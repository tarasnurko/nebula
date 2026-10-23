//! Shortcuts typed on a Cyrillic layout, read as the Latin key they share.
//!
//! With Ctrl, Alt or Cmd held the host reports the letter the active layout
//! puts on the key — `⌘с` for ⌘C on Ukrainian — while every shortcut is
//! spelled in Latin: nebula's own (`ctrl+q`) and the session's (Claude Code
//! copies on `⌘c`, a shell interrupts on `^C`). The kitty protocol's base
//! layout key would say which key it was, but crossterm drops that field, so
//! the JCUKEN letters are folded back to their QWERTY keys here.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Modifiers under which a key is a shortcut, never text.
const SHORTCUT_MODS: KeyModifiers = KeyModifiers::CONTROL
    .union(KeyModifiers::ALT)
    .union(KeyModifiers::SUPER)
    .union(KeyModifiers::HYPER)
    .union(KeyModifiers::META);

/// Each Ukrainian / Russian letter and the US key it sits on.
const CYRILLIC_TO_QWERTY: &[(char, char)] = &[
    ('й', 'q'),
    ('ц', 'w'),
    ('у', 'e'),
    ('к', 'r'),
    ('е', 't'),
    ('н', 'y'),
    ('г', 'u'),
    ('ш', 'i'),
    ('щ', 'o'),
    ('з', 'p'),
    ('х', '['),
    ('ї', ']'),
    ('ъ', ']'),
    ('ф', 'a'),
    ('і', 's'),
    ('ы', 's'),
    ('в', 'd'),
    ('а', 'f'),
    ('п', 'g'),
    ('р', 'h'),
    ('о', 'j'),
    ('л', 'k'),
    ('д', 'l'),
    ('ж', ';'),
    ('є', '\''),
    ('э', '\''),
    ('я', 'z'),
    ('ч', 'x'),
    ('с', 'c'),
    ('м', 'v'),
    ('и', 'b'),
    ('т', 'n'),
    ('ь', 'm'),
    ('б', ','),
    ('ю', '.'),
];

/// `key` with a Cyrillic letter under Ctrl/Alt/Cmd swapped for its QWERTY key.
pub fn latin_shortcut(key: KeyEvent) -> KeyEvent {
    if !key.modifiers.intersects(SHORTCUT_MODS) {
        return key;
    }
    let KeyCode::Char(typed) = key.code else {
        return key;
    };
    let Some(latin) = qwerty_key_of(typed) else {
        return key;
    };
    KeyEvent {
        code: KeyCode::Char(latin),
        ..key
    }
}

fn qwerty_key_of(cyrillic: char) -> Option<char> {
    let lower = cyrillic.to_lowercase().next()?;
    let (_, latin) = CYRILLIC_TO_QWERTY.iter().find(|(c, _)| *c == lower)?;
    Some(if cyrillic.is_uppercase() {
        latin.to_ascii_uppercase()
    } else {
        *latin
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn latin(c: char, mods: KeyModifiers) -> KeyEvent {
        latin_shortcut(KeyEvent::new(KeyCode::Char(c), mods))
    }

    #[test]
    fn cmd_c_on_ukrainian_is_cmd_c() {
        assert_eq!(latin('с', KeyModifiers::SUPER).code, KeyCode::Char('c'));
        assert_eq!(
            latin('с', KeyModifiers::SUPER).modifiers,
            KeyModifiers::SUPER
        );
    }

    #[test]
    fn ctrl_and_alt_shortcuts_fold_too() {
        assert_eq!(latin('с', KeyModifiers::CONTROL).code, KeyCode::Char('c'));
        assert_eq!(latin('и', KeyModifiers::ALT).code, KeyCode::Char('b'));
    }

    #[test]
    fn typed_text_stays_cyrillic() {
        assert_eq!(latin('с', KeyModifiers::NONE).code, KeyCode::Char('с'));
        assert_eq!(latin('С', KeyModifiers::SHIFT).code, KeyCode::Char('С'));
    }

    #[test]
    fn a_capital_folds_to_a_capital() {
        let shifted = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
        assert_eq!(latin('С', shifted).code, KeyCode::Char('C'));
        assert_eq!(latin('Ї', shifted).code, KeyCode::Char(']'));
    }

    #[test]
    fn every_row_lands_on_its_own_key() {
        let cases = [
            ('й', 'q'),
            ('з', 'p'),
            ('х', '['),
            ('ї', ']'),
            ('ъ', ']'),
            ('ф', 'a'),
            ('і', 's'),
            ('ы', 's'),
            ('д', 'l'),
            ('ж', ';'),
            ('є', '\''),
            ('э', '\''),
            ('я', 'z'),
            ('ь', 'm'),
            ('б', ','),
            ('ю', '.'),
        ];
        for (cyrillic, qwerty) in cases {
            assert_eq!(
                latin(cyrillic, KeyModifiers::CONTROL).code,
                KeyCode::Char(qwerty),
                "{cyrillic}"
            );
        }
    }

    #[test]
    fn latin_and_non_letter_keys_pass_untouched() {
        assert_eq!(latin('c', KeyModifiers::SUPER).code, KeyCode::Char('c'));
        assert_eq!(latin('5', KeyModifiers::CONTROL).code, KeyCode::Char('5'));
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
        assert_eq!(latin_shortcut(enter), enter);
    }

    #[test]
    fn the_press_kind_and_state_survive() {
        let release = KeyEvent::new_with_kind_and_state(
            KeyCode::Char('с'),
            KeyModifiers::SUPER,
            KeyEventKind::Release,
            KeyEventState::KEYPAD,
        );
        let folded = latin_shortcut(release);
        assert_eq!(folded.kind, KeyEventKind::Release);
        assert_eq!(folded.state, KeyEventState::KEYPAD);
    }
}
