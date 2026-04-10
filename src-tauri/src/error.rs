use std::sync::PoisonError;

pub type AppResult<T> = Result<T, AppError>;
pub type CommandResult<T> = Result<T, String>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    SchemeNotFound(String),
    #[error("{0}")]
    Network(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    State(String),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn state(message: impl Into<String>) -> Self {
        Self::State(message.into())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::Validation(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::config(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::network(value.to_string())
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(value: PoisonError<T>) -> Self {
        Self::state(format!("应用状态访问失败: {}", value))
    }
}

pub trait IntoCommandResult<T> {
    fn into_command_result(self) -> CommandResult<T>;
}

impl<T> IntoCommandResult<T> for AppResult<T> {
    fn into_command_result(self) -> CommandResult<T> {
        self.map_err(|error| error.to_string())
    }
}
