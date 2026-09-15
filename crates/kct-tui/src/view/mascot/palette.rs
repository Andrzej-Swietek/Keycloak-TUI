use ratatui::style::Color;

pub const TRANSPARENT: char = '.';

/// The colour one palette key paints with. [`Color::Reset`] for a key no ramp defines.
///
/// Three tones per material, lit from the upper left, over a dark outline. Colours are
/// 256-colour indexes, independent of the terminal's own sixteen.
#[must_use]
pub fn colour(key: char) -> Color {
    Color::Indexed(match key {
        // Outline and lashes.
        'k' => 233,
        // Hair: shadow, base, light, shine.
        'H' => 23,
        'h' => 30,
        'w' => 44,
        'W' => 87,
        // Skin: shadow, base, light.
        'S' => 180,
        's' => 223,
        'L' => 230,
        // Eyes: iris, a teal iris for the one who wants it, highlight.
        'e' => 25,
        'i' => 38,
        'E' => 231,
        // Mouth.
        'm' => 168,
        // Outfit: shadow, base, sleeve.
        'c' => 235,
        'C' => 60,
        'l' => 250,
        // Accents: blush, ears, antenna.
        'o' => 217,
        'O' => 210,
        // Ghost body: shadow, base, light.
        'G' => 146,
        'g' => 189,
        'p' => 225,
        _ => return Color::Reset,
    })
}

#[must_use]
pub const fn is_transparent(key: char) -> bool {
    key == TRANSPARENT
}

/// The colour a pixel paints with, or `None` where the sprite is transparent.
#[must_use]
pub fn ink(key: char) -> Option<Color> {
    (!is_transparent(key)).then(|| colour(key))
}
