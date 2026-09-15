//! Choosing and previewing who sits in the corner of the TUI.

use anyhow::Result;
use kct_tui::view::mascot_art;

/// Resolve `--mascot` against the built-in designs and then whatever art was dropped in.
pub fn choose_mascot(
    wanted: &str,
    mascots: &kct_tui::view::Scan,
) -> Result<Option<kct_tui::model::Mascot>> {
    use kct_tui::model::Mascot;
    use kct_tui::view::Character;

    let name = wanted.trim().to_ascii_lowercase();
    if matches!(name.as_str(), "none" | "off" | "") {
        return Ok(None);
    }
    if let Some(character) = Character::parse(&name) {
        return Ok(Some(Mascot::Builtin(character)));
    }
    if let Some(index) = mascots.found.iter().position(|art| *art.name == name) {
        return Ok(Some(Mascot::Custom(index)));
    }

    let known: Vec<&str> = Character::ALL
        .iter()
        .map(|character| character.name())
        .chain(mascots.found.iter().map(|art| &*art.name))
        .collect();
    anyhow::bail!(
        "no mascot called '{name}'; try {} or none",
        known.join(", ")
    )
}

/// Print the built-in designs, then say where to put art of one's own and what was found.
pub fn show(grid: bool) {
    use kct_tui::view::{Character, mascot_preview};

    if grid {
        for character in Character::ALL {
            println!(
                "{}\n{}\n",
                character.name(),
                mascot_preview::grid(character)
            );
        }
    } else {
        println!("{}", mascot_preview::sheet());
    }

    let Some(dir) = mascot_art::mascot_dir() else {
        println!("(no config directory on this platform)");
        return;
    };

    let scan = mascot_art::scan();
    println!("drop your own art in {}", dir.display());
    println!(
        "  a PNG is one character; a folder with default.png plus idle/busy/happy/worried/\
         alarmed/sly.png is one that emotes (max {}x{})",
        mascot_art::MAX_WIDTH,
        mascot_art::MAX_HEIGHT
    );
    for art in scan.found.iter() {
        println!(
            "  {:<16} {}x{}, {} mood frames",
            art.name,
            art.default.width,
            art.default.height(),
            art.moods.len()
        );
    }
    for problem in scan.problems.iter() {
        println!("  skipped {problem}");
    }
}
