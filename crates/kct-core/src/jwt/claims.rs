use serde_json::Value;

/// How long a token has left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expiry {
    /// Seconds remaining.
    Valid(i64),
    /// Seconds since it lapsed.
    Expired(i64),
    /// No `exp` claim, or one that is not a number.
    Unknown,
}

impl Expiry {
    #[must_use]
    pub fn describe(self) -> String {
        match self {
            Self::Valid(seconds) => {
                format!("valid for {}", crate::catalog::format_seconds(seconds))
            }
            Self::Expired(seconds) => {
                format!("EXPIRED {} ago", crate::catalog::format_seconds(seconds))
            }
            Self::Unknown => "no expiry claim".to_owned(),
        }
    }

    #[must_use]
    pub const fn is_expired(self) -> bool {
        matches!(self, Self::Expired(_))
    }
}

/// The claims an operator looks at first when a token is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claims {
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub preferred_username: Option<String>,
    pub authorized_party: Option<String>,
    pub audience: Vec<String>,
    pub scope: Vec<String>,
    pub realm_roles: Vec<String>,
    pub expiry: Expiry,
}

impl Claims {
    #[must_use]
    pub fn from(payload: &Value) -> Self {
        Self {
            issuer: string(payload, "iss"),
            subject: string(payload, "sub"),
            preferred_username: string(payload, "preferred_username"),
            authorized_party: string(payload, "azp"),
            // `aud` is a string when there is one audience and an array when there are more.
            audience: strings(payload.get("aud")),
            scope: string(payload, "scope")
                .map(|scope| scope.split_whitespace().map(ToOwned::to_owned).collect())
                .unwrap_or_default(),
            realm_roles: strings(payload.pointer("/realm_access/roles")),
            expiry: expiry(payload),
        }
    }
}

fn expiry(payload: &Value) -> Expiry {
    let Some(exp) = payload.get("exp").and_then(Value::as_i64) else {
        return Expiry::Unknown;
    };
    let now = chrono::Utc::now().timestamp();

    if exp >= now {
        Expiry::Valid(exp - now)
    } else {
        Expiry::Expired(now - exp)
    }
}

fn string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(one)) => vec![one.clone()],
        Some(Value::Array(many)) => many
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}
