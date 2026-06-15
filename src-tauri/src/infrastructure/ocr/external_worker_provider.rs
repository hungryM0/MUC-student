use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::ocr::provider::{normalize_captcha_text, OcrProvider};

pub struct ExternalWorkerOcrProvider {
    worker_path: PathBuf,
}

impl ExternalWorkerOcrProvider {
    pub fn new(worker_path: PathBuf) -> Self {
        Self { worker_path }
    }
}

#[async_trait]
impl OcrProvider for ExternalWorkerOcrProvider {
    fn name(&self) -> &'static str {
        "external-worker"
    }

    async fn recognize(&self, image_bytes: &[u8]) -> AppResult<String> {
        if !self.worker_path.exists() {
            return Err(AppError::Ocr(format!(
                "OCR worker 不存在：{}",
                self.worker_path.display()
            )));
        }
        let mut child = Command::new(&self.worker_path)
            .arg("--stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| AppError::Ocr(format!("启动 OCR worker 失败：{err}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(image_bytes)
                .await
                .map_err(|err| AppError::Ocr(format!("写入 OCR worker 失败：{err}")))?;
        }
        let output = child
            .wait_with_output()
            .await
            .map_err(|err| AppError::Ocr(format!("等待 OCR worker 失败：{err}")))?;
        if !output.status.success() {
            return Err(AppError::Ocr(format!(
                "OCR worker 返回失败：{}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(normalize_captcha_text(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }
}
