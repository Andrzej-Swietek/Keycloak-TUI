use super::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn typed(text: &str) -> TextInput {
    text.chars()
        .fold(TextInput::default(), super::TextInput::insert)
}

#[test]
fn typing_and_backspace_behave() {
    let input = typed("alice").apply(key(KeyCode::Backspace));
    assert_eq!(input.value(), "alic");
    assert_eq!(input.cursor(), 4);
}

#[test]
fn the_cursor_counts_characters_not_bytes() {
    // A byte cursor would slice these in half and panic.
    let input = typed("zażółć").apply(key(KeyCode::Left)).insert('X');
    assert_eq!(input.value(), "zażółX ć".replace(' ', "").as_str());
}

#[test]
fn readline_shortcuts_work_the_way_muscle_memory_expects() {
    assert_eq!(typed("one two three").apply(ctrl('w')).value(), "one two ");
    assert_eq!(typed("anything").apply(ctrl('u')).value(), "");

    let at_start = typed("hello").apply(ctrl('a'));
    assert_eq!(at_start.cursor(), 0);
    assert_eq!(at_start.apply(ctrl('k')).value(), "");
}

#[test]
fn movement_never_runs_off_either_end() {
    let input = typed("ab")
        .apply(key(KeyCode::Left))
        .apply(key(KeyCode::Left))
        .apply(key(KeyCode::Left));
    assert_eq!(input.cursor(), 0);
    assert_eq!(input.apply(key(KeyCode::Backspace)).value(), "ab");

    let input = typed("ab")
        .apply(key(KeyCode::Right))
        .apply(key(KeyCode::Right));
    assert_eq!(input.cursor(), 2);
    assert_eq!(input.apply(key(KeyCode::Delete)).value(), "ab");
}

#[test]
fn unknown_keys_leave_the_value_alone() {
    let before = typed("stable");
    assert_eq!(before.clone().apply(key(KeyCode::F(5))), before);
}
