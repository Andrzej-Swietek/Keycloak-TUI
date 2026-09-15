//! One expression alphabet for every character.
//!
//! A space means "keep whatever the body already has there", so the same eyes land correctly
//! on skin, on a robot's dark visor and on a ghost, and adding a character costs no
//! expression work at all.

use super::{EYE_ROWS, Mood};

const OPEN: [&str; EYE_ROWS] = ["kkkk   kkkk", "Eeee   Eeee", "keek   keek"];

const BLINK: [&str; EYE_ROWS] = ["           ", "kkkk   kkkk", "           "];

const SMILE: [&str; EYE_ROWS] = ["           ", " kk     kk ", "k  k   k  k"];

const WIDE: [&str; EYE_ROWS] = ["kkkk   kkkk", "EEee   eeEE", "keek   keek"];

const WORRY: [&str; EYE_ROWS] = ["k         k", "kkkk   kkkk", " Eee   eeE "];

const WINK: [&str; EYE_ROWS] = ["kkkk       ", "Eeee    kk ", "keek   k  k"];

const DOT: &str = "     m     ";
const SMIRK: &str = "    kmk    ";
const GRIN: &str = "   kmmmk   ";
const AGAPE: &str = "    mmm    ";
const FLAT: &str = "    kkk    ";

const SMALL_OPEN: [&str; EYE_ROWS] = ["kkk   kkk", "Eee   Eee", "kek   kek"];
const SMALL_BLINK: [&str; EYE_ROWS] = ["         ", "kkk   kkk", "         "];
const SMALL_SMILE: [&str; EYE_ROWS] = ["         ", " k     k ", "k k   k k"];
const SMALL_WIDE: [&str; EYE_ROWS] = ["kkk   kkk", "EEe   eEE", "kek   kek"];
const SMALL_WORRY: [&str; EYE_ROWS] = ["k       k", "kkk   kkk", " Ee   eE "];
const SMALL_WINK: [&str; EYE_ROWS] = ["kkk      ", "Eee    k ", "kek   k k"];

const SMALL_DOT: &str = "    m    ";
const SMALL_SMIRK: &str = "   kmk   ";
const SMALL_GRIN: &str = "  kmmmk  ";
const SMALL_AGAPE: &str = "   mmm   ";
const SMALL_FLAT: &str = "   kkk   ";

/// Three rows: a lash, an iris with its highlight, and the lower lid.
pub fn eyes(mood: Mood, tick: u64) -> [&'static str; EYE_ROWS] {
    let beat = tick % 8;
    match mood {
        Mood::Alarmed => WIDE,
        Mood::Worried => WORRY,
        Mood::Happy => SMILE,
        Mood::Sly if beat < 6 => WINK,
        Mood::Busy if beat % 4 == 3 => BLINK,
        Mood::Idle if beat == 7 => BLINK,
        Mood::Busy | Mood::Sly | Mood::Idle => OPEN,
    }
}

pub fn mouth(mood: Mood) -> &'static str {
    match mood {
        Mood::Alarmed => AGAPE,
        Mood::Happy => GRIN,
        Mood::Sly => SMIRK,
        Mood::Worried => FLAT,
        Mood::Busy | Mood::Idle => DOT,
    }
}

/// The same expressions at nine pixels, for a head that has to stay round to read.
pub fn small_eyes(mood: Mood, tick: u64) -> [&'static str; EYE_ROWS] {
    let beat = tick % 8;
    match mood {
        Mood::Alarmed => SMALL_WIDE,
        Mood::Worried => SMALL_WORRY,
        Mood::Happy => SMALL_SMILE,
        Mood::Sly if beat < 6 => SMALL_WINK,
        Mood::Busy if beat % 4 == 3 => SMALL_BLINK,
        Mood::Idle if beat == 7 => SMALL_BLINK,
        Mood::Busy | Mood::Sly | Mood::Idle => SMALL_OPEN,
    }
}

pub fn small_mouth(mood: Mood) -> &'static str {
    match mood {
        Mood::Alarmed => SMALL_AGAPE,
        Mood::Happy => SMALL_GRIN,
        Mood::Sly => SMALL_SMIRK,
        Mood::Worried => SMALL_FLAT,
        Mood::Busy | Mood::Idle => SMALL_DOT,
    }
}
