/// Failures while obtaining or renewing an access token.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials for realm '{realm}' (client '{client_id}')")]
    InvalidCredentials { realm: String, client_id: String },

    #[error("token endpoint rejected the request: {error}")]
    OAuth {
        error: String,
        description: Option<String>,
    },

    #[error("no usable credentials for context '{0}'")]
    MissingCredentials(String),

    #[error("session expired and could not be renewed: {0}")]
    RefreshFailed(String),

    #[error("the account must complete '{0}' before it can administer anything")]
    RequiredAction(String),

    #[error("device authorization is still pending")]
    AuthorizationPending,
}
