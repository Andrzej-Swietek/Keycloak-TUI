use super::{Design, expression};

/// Round head with ears that break the top of the silhouette.
///
/// The ears are drawn as their own outlined layer in front of the head: a shape this small
/// loses anything that merely shares a colour with what it sits on.
pub const DESIGN: Design = Design {
    name: "cat",
    body: [
        ".........................",
        ".........................",
        ".........................",
        ".........................",
        ".....k.............k.....",
        "....kk............kk.....",
        "...khhk..kkkkkkk.khhk....",
        "..khOOOkkWWWWwwwkkOOOk...",
        ".khhOOkWWWWWWhhhhwkOOhk..",
        ".kkkkkwhhhhhhhhhhhHkkkkk.",
        ".....kwhSSSSSSSSShhk.....",
        "....kwhSsssssssssShHk....",
        "....kwhssssssssssshHk....",
        "....kwhssssssssssshHk....",
        "....kwhssssssssssshHk..kk",
        ".....khhoosssssoohHk..kkk",
        ".....kwhhssssssshhHk..k..",
        "......kHhhssssshHHk...k..",
        ".......kkHHHHHHHkk...khkk",
        ".........kkkkkkk.....kkkk",
    ],
    iris: 'e',
    eye_top: 11,
    face_at: 7,
    eyes: expression::eyes,
    mouth: expression::mouth,
};
