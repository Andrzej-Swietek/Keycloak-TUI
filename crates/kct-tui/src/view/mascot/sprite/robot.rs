use super::{Design, expression};

/// A boxy shell with a dark visor, on which the shared expressions read as glowing eyes.
///
/// The antenna stalk is one pixel wide and drawn lit; [`super::super::outline`] would consume
/// it.
pub const DESIGN: Design = Design {
    name: "robot",
    body: [
        ".........................",
        ".........................",
        ".........................",
        ".........................",
        "...........OOO...........",
        "...........OOO...........",
        "...........ooo...........",
        "............W............",
        "....kkkkkkkkWkkkkkkkk....",
        "...kWWWWWWWWWWWWWWWWWk...",
        "...kWCkkkkkkkkkkkkkCck...",
        "...kWkccccccccccccckCk...",
        "kkkWCkccccccccccccckCCkkk",
        "kWWCCkccccccccccccckCCCWk",
        "kWCCCkccccccccccccckCCCck",
        "kWCCCkccccccccccccckCCcck",
        "kkkCCkccccccccccccckCckkk",
        "...kCCkkkkkkkkkkkkkCck...",
        "...kWcccccccccccccccck...",
        "...kkkkckkckkckkckkkkk...",
    ],
    iris: 'e',
    eye_top: 11,
    face_at: 7,
    eyes: expression::eyes,
    mouth: expression::mouth,
};
