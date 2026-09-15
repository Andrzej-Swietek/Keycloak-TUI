use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::decode::decode;
use super::{CustomMascot, Mood, Scan};

/// Overrides where custom art is read from, for testing and for keeping art beside a project.
pub const MASCOT_DIR_ENV: &str = "KCT_MASCOT_DIR";

/// Where drop-in art lives: `$KCT_MASCOT_DIR`, else the platform config directory.
#[must_use]
pub fn mascot_dir() -> Option<PathBuf> {
    if let Some(from_env) = std::env::var_os(MASCOT_DIR_ENV) {
        return Some(PathBuf::from(from_env));
    }
    directories::ProjectDirs::from("", "", "kct").map(|dirs| dirs.config_dir().join("mascots"))
}

/// Read every mascot under `root`.
///
/// Two layouts, both without a manifest: `name.png` is a character with one frame, and a
/// directory `name/` holding `default.png` plus any of `idle.png`, `busy.png`, `happy.png`,
/// `worried.png`, `alarmed.png`, `sly.png` is one that emotes.
#[must_use]
pub fn load_from(root: &Path) -> Scan {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Scan::default();
    };

    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();

    let mut found = Vec::new();
    let mut problems = Vec::new();
    for path in paths {
        let Some(name) = stem(&path) else { continue };
        match read_mascot(&path, &name) {
            Ok(Some(mascot)) => found.push(mascot),
            Ok(None) => {}
            Err(why) => problems.push(format!("{name}: {why}")),
        }
    }

    Scan {
        found: Arc::from(found),
        problems: Arc::from(problems),
    }
}

fn stem(path: &Path) -> Option<String> {
    let name = path.file_stem()?.to_str()?.trim().to_ascii_lowercase();
    (!name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
    .then_some(name)
}

fn read_mascot(path: &Path, name: &str) -> Result<Option<CustomMascot>, String> {
    if path.is_dir() {
        return read_folder(path, name).map(Some);
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("png") {
        return Ok(None);
    }
    Ok(Some(CustomMascot {
        name: Arc::from(name),
        default: decode(path)?,
        moods: Arc::from(Vec::new()),
    }))
}

fn read_folder(dir: &Path, name: &str) -> Result<CustomMascot, String> {
    let mut moods = Vec::new();
    for (mood, file) in MOOD_FILES {
        let candidate = dir.join(file);
        if candidate.is_file() {
            moods.push((*mood, decode(&candidate)?));
        }
    }

    let fallback = dir.join("default.png");
    let default = if fallback.is_file() {
        decode(&fallback)?
    } else {
        moods
            .first()
            .map(|(_, art)| art.clone())
            .ok_or_else(|| "no default.png and no mood frames".to_owned())?
    };

    Ok(CustomMascot {
        name: Arc::from(name),
        default,
        moods: Arc::from(moods),
    })
}

const MOOD_FILES: &[(Mood, &str)] = &[
    (Mood::Idle, "idle.png"),
    (Mood::Busy, "busy.png"),
    (Mood::Happy, "happy.png"),
    (Mood::Worried, "worried.png"),
    (Mood::Alarmed, "alarmed.png"),
    (Mood::Sly, "sly.png"),
];
