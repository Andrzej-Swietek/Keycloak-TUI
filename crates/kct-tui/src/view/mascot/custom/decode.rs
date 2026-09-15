//! Turning a PNG into pixels the panel can paint.

use std::path::Path;
use std::sync::Arc;

use ratatui::style::Color;

use super::{Art, MAX_HEIGHT, MAX_WIDTH};

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
    let pixels = to_pixels(
        &buffer[..frame.buffer_size()],
        frame.color_type,
        width * height,
    )?;
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

fn to_pixels(
    bytes: &[u8],
    colour: png::ColorType,
    count: usize,
) -> Result<Vec<Option<Color>>, String> {
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
