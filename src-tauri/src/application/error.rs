use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub detail: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Network(String),
    #[error("{0}")]
    Storage(String),
    #[error("{0}")]
    System(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "TASK_CONFLICT",
            Self::Network(_) => "NETWORK_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::System(_) => "SYSTEM_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn detail(&self) -> String {
        self.to_string()
    }
}

impl From<AppError> for AppErrorDto {
    fn from(value: AppError) -> Self {
        Self {
            code: value.code().to_string(),
            message: value.to_string(),
            detail: value.detail(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Storage(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Storage(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type CommandResult<T> = Result<T, AppErrorDto>;

pub trait IntoCommandResult<T> {
    fn into_command_result(self) -> CommandResult<T>;
}

impl<T> IntoCommandResult<T> for AppResult<T> {
    fn into_command_result(self) -> CommandResult<T> {
        self.map_err(Into::into)
    }
}
