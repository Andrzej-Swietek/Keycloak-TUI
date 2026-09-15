use super::{Design, expression};

/// The same character as `twintails`, drawn full length at chibi proportions.
///
/// The head is deliberately larger than the body: shrink it to life-like proportions and the
/// face — the only part that carries an expression — stops being legible at this size.
pub const DESIGN: Design = Design {
    name: "miku",
    body: [
        "..........kkkkk..........",
        ".kkkk...kkWWWWwkk...kkkk.",
        "kwwwwk.kwwWhhhhhwk.kwwwwk",
        "kccchHkwhhhhhhhhhhkwhccck",
        "kcccHkHhSSSSSSSSShHhhccck",
        "kwhHk.khssssssssshHhhhHk.",
        "kwhHk.kwssssssssshHhhhHk.",
        "kwhHk.kwsoosssooshHhhhHk.",
        "kwhHk.kwsssssssssHkhhhHk.",
        "kwhHk..kHsssssssHk.khhHk.",
        "kwhHk...kkssssskk..kwhHk.",
        "kwhHk.....kkkkk....kwhHk.",
        "kwhHk......kSk.....kwhHk.",
        "kwhHkkkkkkkwwwkkkkkkwhHk.",
        "kwhHkklllllwWwCllllkwhHk.",
        "kwhhkklllCCwWwCClllkwhhk.",
        "kwhhHklCCCCwWwCCCClkwhhHk",
        "kwhhHklCCCCwWwCCCCCkwhhHk",
        ".kHHklCCCCCCCCCCCCCckHHk.",
        "..kk.kCCCCCCCCCCCCCk.kk..",
    ],
    iris: 'e',
    eye_top: 4,
    face_at: 8,
    eyes: expression::small_eyes,
    mouth: expression::small_mouth,
};
