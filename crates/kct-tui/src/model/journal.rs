use std::time::Instant;

use kct_core::Method;

use crate::msg::Notice;

/// One recorded API call, for the request log.
#[derive(Debug, Clone)]
pub struct LogLine {
    pub at: Instant,
    pub method: Method,
    pub path: String,
    pub outcome: Result<u16, String>,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub notice: Notice,
    pub at: Instant,
}
