use std::fmt;

/// A resolved, ready-to-send API path, absolute from the server root.
///
/// [`ApiPath::new`] prepends `/` when the input lacks one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ApiPath(String);

impl ApiPath {
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        Self(if path.starts_with('/') {
            path
        } else {
            format!("/{path}")
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Append one identifier, percent-encoding it.
    #[must_use]
    pub fn child(&self, segment: &str) -> Self {
        Self(format!(
            "{}/{}",
            self.0.trim_end_matches('/'),
            encode_segment(segment)
        ))
    }

    /// Append a literal sub-path such as `"role-mappings/realm"`.
    #[must_use]
    pub fn sub(&self, literal: &str) -> Self {
        Self(format!(
            "{}/{}",
            self.0.trim_end_matches('/'),
            literal.trim_start_matches('/')
        ))
    }
}

impl fmt::Display for ApiPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Percent-encode one path segment.
///
/// Deliberately conservative: Keycloak identifiers include UUIDs, usernames, client ids,
/// group paths and LDAP DNs, and an unencoded `/`, `#` or space turns a correct request into
/// a 404 — or worse, a request against a different object.
#[must_use]
pub fn encode_segment(segment: &str) -> String {
    segment
        .bytes()
        .fold(String::with_capacity(segment.len()), |mut acc, byte| {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    acc.push(byte as char);
                }
                other => {
                    const HEX: &[u8; 16] = b"0123456789ABCDEF";
                    acc.push('%');
                    acc.push(HEX[(other >> 4) as usize] as char);
                    acc.push(HEX[(other & 0x0f) as usize] as char);
                }
            }
            acc
        })
}
