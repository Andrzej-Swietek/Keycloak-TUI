use std::fmt::Write as _;

use super::{Character, Mood, palette, sprite};

/// Every character in every mood, as ANSI text.
///
/// Iterating on sprite art through a running TUI is miserable — launch, log in, contrive the
/// mood, squint at a corner. This renders the whole sheet to stdout in one command, which is
/// the difference between guessing and looking.
#[must_use]
pub fn sheet() -> String {
    let moods = [
        Mood::Idle,
        Mood::Busy,
        Mood::Happy,
        Mood::Worried,
        Mood::Alarmed,
        Mood::Sly,
    ];

    Character::ALL
        .into_iter()
        .map(|character| {
            let strip: Vec<Vec<String>> = moods
                .iter()
                .map(|mood| ansi_rows(character, *mood, 0))
                .collect();

            let header = moods.iter().fold(String::new(), |mut acc, mood| {
                let _ = write!(acc, "{:^18}", mood.caption());
                acc
            });

            let body = (0..sprite::HEIGHT / 2)
                .map(|row| {
                    strip.iter().fold(String::new(), |mut acc, mood_rows| {
                        let _ = write!(acc, "  {}  ", mood_rows[row]);
                        acc
                    })
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!("{}\n{header}\n{body}\n", character.name())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The raw pixel grid, one character per palette key.
///
/// Colour is the last thing to get right; shape is the first, and shape is legible here even
/// where a terminal's colours are not.
#[must_use]
pub fn grid(character: Character) -> String {
    sprite::frame(character, Mood::Idle, 0).join("\n")
}

fn ansi_rows(character: Character, mood: Mood, tick: u64) -> Vec<String> {
    let pixels = sprite::frame(character, mood, tick);

    pixels
        .chunks(2)
        .map(|pair| {
            pair[0].chars().zip(pair[1].chars()).fold(
                String::new(),
                |mut acc, (top, bottom)| {
                    // Same glyph choice as `super::paint`: an unpainted pixel, not a
                    // half-block coloured with the terminal default.
                    let _ = match (palette::is_transparent(top), palette::is_transparent(bottom)) {
                        (true, true) => acc.write_char(' '),
                        (true, false) => write!(acc, "\x1b[38;5;{}m\u{2584}\x1b[0m", indexed(bottom)),
                        (false, true) => write!(acc, "\x1b[38;5;{}m\u{2580}\x1b[0m", indexed(top)),
                        (false, false) => write!(
                            acc,
                            "\x1b[38;5;{}m\x1b[48;5;{}m\u{2580}\x1b[0m",
                            indexed(top),
                            indexed(bottom)
                        ),
                    };
                    acc
                },
            )
        })
        .collect()
}

/// The palette's 256-colour index, or the terminal default for a transparent pixel.
fn indexed(key: char) -> String {
    match palette::colour(key) {
        ratatui::style::Color::Indexed(index) => index.to_string(),
        _ => "0".to_owned(),
    }
}
