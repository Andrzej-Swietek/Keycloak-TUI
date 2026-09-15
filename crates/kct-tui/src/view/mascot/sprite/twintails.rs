use super::{Design, expression};

/// A chibi bust with twin bunches gathered at ribbons.
///
/// The bunches overlap the head's sides so they read as attached hair and hang below it, so
/// the silhouette stays unmistakable at a glance — which is the whole job of a corner sprite.
pub const DESIGN: Design = Design {
    name: "twintails",
    body: [
        ".........................",
        ".........................",
        ".........................",
        ".........................",
        ".........................",
        "........kkkkkkkkk........",
        ".......kWWWWWwwwwk.......",
        "..cck.kwhhhhhhhhhhk.kcc..",
        ".cccHkwhShhhhhhhShhkHccc.",
        "kwhHHhhSsSSSSSSSsShhHHhHk",
        ".khHHhhssssssssssshhHHHk.",
        ".kwHHhhssssssssssshhHHHk.",
        ".kwHHhhssssssssssshhHHHk.",
        ".kwHHHkhoosssssooHkhHHHk.",
        ".kwHHk.kHsssssssHk.kHHhk.",
        "kwhHHHk.kkkkkkkkk.kwHHhHk",
        ".khHHk.............kHHHk.",
        ".kwhHk.............kwhHk.",
        ".kwHHk.............kwHHk.",
        ".kkkkk.............kkkkk.",
    ],
    iris: 'e',
    eye_top: 9,
    face_at: 7,
    eyes: expression::eyes,
    mouth: expression::mouth,
};
