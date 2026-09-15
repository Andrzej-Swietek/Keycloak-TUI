use crate::error::{Error, Result};

use super::api_path::{ApiPath, encode_segment};
use super::params::PathParams;

/// A path with `{slot}` placeholders, e.g. `/admin/realms/{realm}/clients/{client}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Template(pub &'static str);

impl Template {
    /// Slot names this template needs, in order of appearance.
    #[must_use]
    pub fn slots(&self) -> Vec<&'static str> {
        self.0
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect()
    }

    /// Substitute every placeholder, percent-encoding the values.
    ///
    /// A missing slot is a programming error, reported as a normal error so a mis-declared
    /// catalog entry surfaces as a message rather than a panic mid-session.
    pub fn resolve(&self, params: &PathParams) -> Result<ApiPath> {
        let resolved = self.0.split('{').enumerate().try_fold(
            String::with_capacity(self.0.len()),
            |mut acc, (index, chunk)| {
                if index == 0 {
                    acc.push_str(chunk);
                    return Ok(acc);
                }
                let Some((slot, rest)) = chunk.split_once('}') else {
                    return Err(Error::config(format!(
                        "malformed path template '{}': unbalanced '{{'",
                        self.0
                    )));
                };
                let value = params.get(slot).ok_or_else(|| {
                    Error::config(format!(
                        "path template '{}' needs '{slot}', but only {:?} were supplied",
                        self.0,
                        params.names()
                    ))
                })?;
                acc.push_str(&encode_segment(value));
                acc.push_str(rest);
                Ok(acc)
            },
        )?;

        Ok(ApiPath::new(resolved))
    }
}
