use std::io::Write;

use anyhow::{Result, bail};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::terminal;

/// Read a secret from the terminal with echo off.
///
/// Restores the terminal's original mode before returning, including on error.
pub fn password(label: &str) -> Result<String> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        bail!("no terminal to prompt on; pass --password or set KCT_PASSWORD");
    }

    print!("{label}: ");
    std::io::stdout().flush()?;

    terminal::enable_raw_mode()?;
    let entered = read_line();
    terminal::disable_raw_mode()?;
    println!();

    entered
}

fn read_line() -> Result<String> {
    let mut typed = String::new();

    loop {
        let Event::Key(key) = crossterm::event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Enter => return Ok(typed),
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                bail!("cancelled")
            }
            KeyCode::Char(character) => typed.push(character),
            KeyCode::Backspace => {
                typed.pop();
            }
            KeyCode::Esc => bail!("cancelled"),
            _ => {}
        }
    }
}
