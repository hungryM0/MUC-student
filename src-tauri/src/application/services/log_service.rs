use chrono::Local;

use crate::application::dto::LogItemDto;

#[derive(Clone, Default)]
pub struct LogService;

impl LogService {
    pub fn entry(level: impl Into<String>, message: impl Into<String>) -> LogItemDto {
        LogItemDto {
            timestamp: Local::now(),
            level: level.into(),
            message: message.into(),
        }
    }
}
