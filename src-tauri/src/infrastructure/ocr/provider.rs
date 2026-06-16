use async_trait::async_trait;

use crate::application::error::AppResult;

#[async_trait]
pub trait OcrProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn recognize(&self, image_bytes: &[u8], expected_sum: Option<u32>) -> AppResult<String>;
}

pub struct OcrProviderChain {
    native: std::sync::Arc<dyn OcrProvider>,
    worker: std::sync::Arc<dyn OcrProvider>,
    native_attempts: usize,
}

impl OcrProviderChain {
    pub fn new(
        native: std::sync::Arc<dyn OcrProvider>,
        worker: std::sync::Arc<dyn OcrProvider>,
    ) -> Self {
        Self {
            native,
            worker,
            native_attempts: 3,
        }
    }

    pub async fn recognize_for_login(
        &self,
        image_bytes: &[u8],
        expected_sum: Option<u32>,
    ) -> AppResult<String> {
        let mut last_error = None;
        for _ in 0..self.native_attempts {
            match self.native.recognize(image_bytes, expected_sum).await {
                Ok(text) if !text.trim().is_empty() => return Ok(text),
                Ok(_) => last_error = Some("原生 OCR 返回空结果".to_string()),
                Err(err) => last_error = Some(err.to_string()),
            }
        }
        match self.worker.recognize(image_bytes, expected_sum).await {
            Ok(text) if !text.trim().is_empty() => Ok(text),
            Ok(_) => Err(crate::application::error::AppError::Ocr(format!(
                "OCR 兜底 worker 返回空结果，原生错误={}",
                last_error.unwrap_or_else(|| "unknown".to_string())
            ))),
            Err(err) => Err(crate::application::error::AppError::Ocr(format!(
                "OCR 双 provider 均失败：native={}, worker={}",
                last_error.unwrap_or_else(|| "unknown".to_string()),
                err
            ))),
        }
    }
}

pub fn normalize_captcha_text(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(4)
        .collect()
}
