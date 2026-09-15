use crate::version::Version;

use super::{ServerProfile, Support};

/// The individual checks behind [`ServerProfile::evaluate`].
impl ServerProfile {
    pub(super) fn check_min_version(&self, min: Version) -> Support {
        if self.version.is_unknown() {
            Support::Unknown(format!("needs Keycloak {min} or newer; version unknown"))
        } else if self.version >= min {
            Support::Available
        } else {
            Support::Unavailable(format!(
                "needs Keycloak {min} or newer (this server is {})",
                self.version
            ))
        }
    }

    pub(super) fn check_below_version(&self, max: Version) -> Support {
        if self.version.is_unknown() {
            Support::Unknown(format!("removed in Keycloak {max}; version unknown"))
        } else if self.version < max {
            Support::Available
        } else {
            Support::Unavailable(format!("removed in Keycloak {max}"))
        }
    }

    pub(super) fn check_feature(&self, name: &str) -> Support {
        match self.feature(name) {
            Some(flag) if flag.enabled => Support::Available,
            Some(_) => Support::Unavailable(format!(
                "feature '{name}' is disabled; start Keycloak with --features={name}"
            )),
            None if self.source.is_complete() => Support::Unavailable(format!(
                "feature '{name}' does not exist on this Keycloak version"
            )),
            None => Support::Unknown(format!(
                "feature '{name}' could not be checked ({})",
                self.source.describe()
            )),
        }
    }

    pub(super) fn check_provider(&self, spi: &str, id: &str) -> Support {
        if self.has_provider(spi, id) {
            Support::Available
        } else if self.source.is_complete() {
            Support::Unavailable(format!("no '{id}' provider installed for SPI '{spi}'"))
        } else {
            Support::Unknown(format!("provider '{spi}/{id}' could not be checked"))
        }
    }
}
