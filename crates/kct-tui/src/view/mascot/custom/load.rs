use std::path::{Path, PathBuf};
use std::sync::Arc;

use ratatui::style::Color;

use super::{Art, CustomMascot, MAX_HEIGHT, MAX_WIDTH, Mood, Scan};

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
    (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
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

/// Decode one PNG.
///
/// Palette, grayscale, RGB and RGBA at 8 or 16 bits are accepted. Pixels under half alpha
/// become `None`. Errs on an image wider than [`MAX_WIDTH`] or taller than [`MAX_HEIGHT`].
pub(super) fn decode(path: &Path) -> Result<Art, String> {
    let file = std::fs::File::open(path).map_err(|why| why.to_string())?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    // Expand palette and grayscale to RGB, and 16-bit down to 8.
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|why| why.to_string())?;

    let info = reader.info();
    let (width, height) = (info.width as usize, info.height as usize);
    if width == 0 || height == 0 {
        return Err("empty image".to_owned());
    }
    if width > MAX_WIDTH || height > MAX_HEIGHT {
        return Err(format!(
            "{width}x{height} is larger than the {MAX_WIDTH}x{MAX_HEIGHT} a corner panel can show"
        ));
    }

    let mut buffer = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let frame = reader
        .next_frame(&mut buffer)
        .map_err(|why| why.to_string())?;
    let pixels = to_pixels(&buffer[..frame.buffer_size()], frame.color_type, width * height)?;
    Ok(Art {
        width,
        pixels: Arc::from(pixels),
    })
}

/// Whether the terminal advertises 24-bit colour.
///
/// Art is authored in RGB, but a terminal that only does 256 renders 24-bit escapes as
/// nothing useful. Quantising once at load time costs nothing per frame and means a drop-in
/// PNG still looks like itself over an old SSH session.
fn truecolor() -> bool {
    std::env::var("COLORTERM").is_ok_and(|value| {
        let value = value.to_ascii_lowercase();
        value.contains("truecolor") || value.contains("24bit")
    })
}

/// Nearest entry in the xterm 6x6x6 cube, or its greyscale ramp when the pixel is neutral.
pub(super) fn quantise(red: u8, green: u8, blue: u8) -> Color {
    let spread = u32::from(red.max(green).max(blue)) - u32::from(red.min(green).min(blue));
    if spread < 16 {
        let level = (u32::from(red) + u32::from(green) + u32::from(blue)) / 3;
        if level < 8 {
            return Color::Indexed(16);
        }
        if level > 238 {
            return Color::Indexed(231);
        }
        let step = u8::try_from((level - 8) / 10).unwrap_or(23).min(23);
        return Color::Indexed(232 + step);
    }

    let axis = |value: u8| u16::from(value).saturating_add(25) / 51;
    let index = 16 + 36 * axis(red) + 6 * axis(green) + axis(blue);
    Color::Indexed(u8::try_from(index).unwrap_or(231))
}

fn to_pixels(bytes: &[u8], colour: png::ColorType, count: usize) -> Result<Vec<Option<Color>>, String> {
    let stride = match colour {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        other => return Err(format!("{other:?} is not supported; save as RGB or RGBA")),
    };

    let rich = truecolor();
    Ok((0..count)
        .map(|index| {
            let at = index * stride;
            let alpha = if stride == 4 { bytes[at + 3] } else { 255 };
            (alpha >= 128).then(|| {
                let (red, green, blue) = (bytes[at], bytes[at + 1], bytes[at + 2]);
                if rich {
                    Color::Rgb(red, green, blue)
                } else {
                    quantise(red, green, blue)
                }
            })
        })
        .collect())
}
