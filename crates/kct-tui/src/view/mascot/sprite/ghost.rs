use super::{Design, expression};

/// A teardrop body with a wavy hem and no face plate.
///
/// The expression lands straight on the body: with `KEEP` in the expression alphabet the same
/// face works on skin, on a visor and here.
pub const DESIGN: Design = Design {
    name: "ghost",
    body: [
        ".........................",
        ".........................",
        ".........................",
        ".........................",
        "............k............",
        "........kkkkpkkkk........",
        ".......kpppppppppk.......",
        "......kpppppppggggk......",
        ".....kpppppppggggggk.....",
        "....kpgggggggggggggGk....",
        "....kpgggggggggggggGk....",
        "....kpgggggggggggggGk....",
        "....kpgggggggggggggGk....",
        "....kpgggggggggggggGk....",
        "....kpgggggggggggggGk....",
        "....kpggoogggggooggGk....",
        "....kpgggGGggggGGggGk....",
        "....kkggGkkkggGkkkgGk....",
        "......kGk...kGk...kGk....",
        "......kkk...kkk...kkk....",
    ],
    iris: 'e',
    eye_top: 11,
    face_at: 7,
    eyes: expression::eyes,
    mouth: expression::mouth,
};
