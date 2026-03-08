use thiserror::Error;

#[derive(Debug, Error)]
pub enum RimuruError {
    #[error("state error: {0}")]
    State(String),

    #[error("adapter error: {0}")]
    Adapter(String),

    #[error("bridge error: {0}")]
    Bridge(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("plugin error: {0}")]
    Plugin(String),

    #[error("hook error: {0}")]
    Hook(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(String),

    #[error("command error: {0}")]
    Command(String),
}

impl From<iii_sdk::IIIError> for RimuruError {
    fn from(err: iii_sdk::IIIError) -> Self {
        Self::Bridge(err.to_string())
    }
}

impl RimuruError {
    pub fn to_json_error(&self) -> serde_json::Value {
        serde_json::json!({
            "error": true,
            "message": self.to_string()
        })
    }
}
