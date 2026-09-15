use std::path::PathBuf;

use super::*;

/// Write a PNG of solid colour with one fully transparent column, to prove alpha is honoured.
fn write_png(path: &std::path::Path, width: u32, height: u32) {
    let file = std::fs::File::create(path).expect("scratch directory is writable");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("header");

    let mut data = Vec::new();
    for _ in 0..height {
        for x in 0..width {
            let opaque = u8::from(x != 0) * 255;
            data.extend_from_slice(&[10, 200, 180, opaque]);
        }
    }
    writer.write_image_data(&data).expect("pixels");
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kct-mascot-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

#[test]
fn a_loose_png_becomes_a_character_named_after_its_file() {
    let dir = scratch("loose");
    write_png(&dir.join("Nyan.png"), 6, 4);
    write_png(&dir.join("notes.txt.png"), 6, 4);

    let scan = load_from(&dir);
    let names: Vec<&str> = scan.found.iter().map(|art| &*art.name).collect();

    assert!(names.contains(&"nyan"), "got {names:?}");
    assert!(scan.problems.is_empty(), "{:?}", scan.problems);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn transparent_pixels_stay_unpainted() {
    let dir = scratch("alpha");
    write_png(&dir.join("ghosty.png"), 3, 2);

    let scan = load_from(&dir);
    let art = &scan.found[0].default;

    assert_eq!(art.width, 3);
    assert_eq!(art.height(), 2);
    assert_eq!(art.row(0)[0], None, "the first column was fully transparent");
    assert!(art.row(0)[1].is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_folder_supplies_one_frame_per_mood_and_falls_back_for_the_rest() {
    let dir = scratch("folder");
    let who = dir.join("hero");
    std::fs::create_dir_all(&who).expect("character directory");
    write_png(&who.join("default.png"), 4, 4);
    write_png(&who.join("alarmed.png"), 6, 4);

    let scan = load_from(&dir);
    let hero = &scan.found[0];

    assert_eq!(&*hero.name, "hero");
    assert_eq!(hero.frame(Mood::Alarmed).width, 6);
    assert_eq!(hero.frame(Mood::Happy).width, 4, "falls back to default");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn art_too_large_for_the_panel_is_reported_rather_than_ignored() {
    let dir = scratch("huge");
    write_png(&dir.join("giant.png"), (MAX_WIDTH + 1) as u32, 4);

    let scan = load_from(&dir);

    assert!(scan.found.is_empty());
    assert!(
        scan.problems.iter().any(|problem| problem.contains("giant")),
        "a mascot that silently fails to appear is worse than one that says why: {:?}",
        scan.problems
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn colours_survive_a_terminal_that_only_does_256() {
    // Not every SSH session advertises truecolor; the art still has to look like itself.
    for (rgb, expected) in [
        ((0, 0, 0), 16),
        ((255, 255, 255), 231),
        ((128, 128, 128), 244),
        ((255, 0, 0), 196),
        ((0, 255, 0), 46),
    ] {
        assert_eq!(
            super::load::quantise(rgb.0, rgb.1, rgb.2),
            ratatui::style::Color::Indexed(expected),
            "{rgb:?}"
        );
    }
}
