use super::*;

const MOODS: [Mood; 6] = [
    Mood::Idle,
    Mood::Busy,
    Mood::Happy,
    Mood::Worried,
    Mood::Alarmed,
    Mood::Sly,
];

#[test]
fn every_row_of_every_design_is_exactly_the_sprite_width() {
    for character in Character::ALL {
        for (index, row) in character.design().body.iter().enumerate() {
            assert_eq!(
                row.chars().count(),
                WIDTH,
                "{} body row {index}: {row}",
                character.name()
            );
        }
    }
}

#[test]
fn every_expression_is_one_width_that_fits_inside_the_sprite() {
    // Designs differ in how wide a face they wear; what must hold is that one design's eyes,
    // mouth and every mood agree, and that none of them runs off the canvas.
    for character in Character::ALL {
        let design = character.design();
        let width = (design.mouth)(Mood::Idle).chars().count();
        assert!(
            design.face_at + width <= WIDTH,
            "{} face runs past the sprite",
            design.name
        );
        for mood in MOODS {
            for tick in 0..8 {
                for band in (design.eyes)(mood, tick) {
                    assert_eq!(band.chars().count(), width, "{} {mood:?}", design.name);
                }
            }
            assert_eq!(
                (design.mouth)(mood).chars().count(),
                width,
                "{} {mood:?} mouth",
                design.name
            );
        }
    }
}

#[test]
fn the_face_window_always_lands_on_the_body() {
    // An eye drawn over a transparent pixel floats beside the character instead of on it.
    for character in Character::ALL {
        let design = character.design();
        let body = design.body;
        let width = (design.mouth)(Mood::Idle).chars().count();
        for row in [
            design.eye_top,
            design.eye_top + 1,
            design.eye_top + 2,
            design.mouth_row(),
        ] {
            let window: String = body[row]
                .chars()
                .skip(design.face_at)
                .take(width)
                .filter(|pixel| *pixel == super::super::palette::TRANSPARENT)
                .collect();
            assert!(
                window.is_empty(),
                "{} row {row} has {} transparent pixels under the face",
                character.name(),
                window.len()
            );
        }
    }
}

#[test]
fn patching_a_face_never_changes_the_silhouette() {
    for character in Character::ALL {
        for mood in MOODS {
            for row in frame(character, mood, 0) {
                assert_eq!(row.chars().count(), WIDTH, "{}", character.name());
            }
        }
    }
}

#[test]
fn an_alarmed_face_is_never_smiling() {
    // It fires one keypress from a delete.
    for character in Character::ALL {
        let mouth = character.design().mouth_row();
        let happy = frame(character, Mood::Happy, 0);
        let alarmed = frame(character, Mood::Alarmed, 0);
        assert_ne!(happy[mouth], alarmed[mouth], "{}", character.name());
    }
}

#[test]
fn a_panel_is_only_as_tall_as_the_character_in_it() {
    // A bust must not reserve the rows a full figure needs, or choosing it costs listing space
    // for nothing.
    let bust = Character::Cat.design().top();
    let figure = Character::Miku.design().top();
    assert!(bust > figure, "a bust should start lower than a full figure");
    assert_eq!(bust % 2, 0, "an odd start would pair pixels with the wrong neighbour");
    assert_eq!(figure % 2, 0);
}

#[test]
fn every_design_blinks_within_one_cycle() {
    for character in Character::ALL {
        let seen: std::collections::BTreeSet<String> = (0..8)
            .map(|tick| frame(character, Mood::Idle, tick)[character.design().eye_top].clone())
            .collect();
        assert!(seen.len() > 1, "{} looks hung", character.name());
    }
}

#[test]
fn characters_are_addressable_by_name_and_cycle() {
    assert_eq!(Character::parse("cat"), Some(Character::Cat));
    assert_eq!(Character::parse(" Robot "), Some(Character::Robot));
    assert_eq!(Character::parse("nobody"), None);

    let cycled: Vec<&str> = std::iter::successors(Some(Character::default()), |c| Some(c.next()))
        .take(Character::ALL.len() + 1)
        .map(Character::name)
        .collect();
    assert_eq!(
        cycled.first(),
        cycled.last(),
        "cycling returns to the start"
    );
    assert_eq!(
        cycled[..Character::ALL.len()]
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        Character::ALL.len(),
        "cycling visits every character once"
    );
}
