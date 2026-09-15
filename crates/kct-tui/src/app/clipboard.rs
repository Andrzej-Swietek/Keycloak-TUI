use std::io::Write;

use base64::Engine;

/// Copy via OSC 52.
///
/// Deliberately not a clipboard library: this tool is used over SSH more than locally, and an
/// escape sequence the terminal emulator interprets reaches the *operator's* clipboard rather
/// than the clipboard of whatever host it happens to be running on.
pub fn copy(text: &str) -> Result<(), String> {
    // Most emulators cap the sequence; a truncated paste is worse than a refused one.
    const LIMIT: usize = 100_000;
    if text.len() > LIMIT {
        return Err(format!("{} bytes is too large for OSC 52", text.len()));
    }

    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    let mut out = std::io::stdout();

    write!(out, "\x1b]52;c;{encoded}\x07").map_err(|error| error.to_string())?;
    out.flush().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn oversized_payloads_are_refused_rather_than_silently_truncated() {
        let huge = "x".repeat(200_000);
        assert!(super::copy(&huge).is_err());
    }
}
