use crate::view::Character;

/// Who is in the corner.
///
/// [`Mascot::Custom`] indexes [`crate::view::Scan::found`] on the model; after a reload that
/// shortens it, [`Prefs::clamped`] brings the choice back in range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mascot {
    Builtin(Character),
    Custom(usize),
}

/// Things an operator can change or turn off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Prefs {
    /// `None` hides the panel entirely.
    pub mascot: Option<Mascot>,
}

impl Prefs {
    #[must_use]
    pub fn hidden() -> Self {
        Self { mascot: None }
    }

    #[must_use]
    pub fn showing(mascot: Mascot) -> Self {
        Self {
            mascot: Some(mascot),
        }
    }

    /// Cycle through the built-in designs, then whatever art was loaded, then hidden.
    #[must_use]
    pub fn next_mascot(self, customs: usize) -> Self {
        let builtins = Character::ALL.len();
        let position = match self.mascot {
            None => return Self::showing(Mascot::Builtin(Character::default())),
            Some(Mascot::Builtin(current)) => Character::ALL
                .iter()
                .position(|candidate| *candidate == current)
                .unwrap_or(0),
            Some(Mascot::Custom(index)) => builtins + index.min(customs.saturating_sub(1)),
        };

        match position + 1 {
            next if next < builtins => Self::showing(Mascot::Builtin(Character::ALL[next])),
            next if next < builtins + customs => Self::showing(Mascot::Custom(next - builtins)),
            _ => Self::hidden(),
        }
    }

    /// Keep the choice pointing at art that still exists after a reload.
    #[must_use]
    pub fn clamped(self, customs: usize) -> Self {
        match self.mascot {
            Some(Mascot::Custom(index)) if index >= customs => {
                Self::showing(Mascot::Builtin(Character::default()))
            }
            _ => self,
        }
    }
}

impl Default for Mascot {
    fn default() -> Self {
        Self::Builtin(Character::default())
    }
}
