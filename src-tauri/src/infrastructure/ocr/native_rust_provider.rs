use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ddddocr::DdddOcr;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::ocr::provider::{normalize_captcha_text, OcrProvider};

pub struct NativeRustOcrProvider {
    model_path: PathBuf,
    engine: Arc<Mutex<Option<DdddOcr>>>,
}

impl NativeRustOcrProvider {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            engine: Arc::new(Mutex::new(None)),
        }
    }

    fn ensure_engine(&self) -> AppResult<()> {
        if self
            .engine
            .lock()
            .map_err(|_| AppError::Ocr("OCR 引擎锁损坏".to_string()))?
            .is_some()
        {
            return Ok(());
        }
        if !Path::new(&self.model_path).exists() {
            return Err(AppError::Ocr(format!(
                "ddddocr 旧版轻量模型缺失：{}",
                self.model_path.display()
            )));
        }
        let engine = DdddOcr::new(&self.model_path)
            .map_err(|err| AppError::Ocr(format!("加载 ddddocr 旧版轻量模型失败：{err}")))?;
        *self
            .engine
            .lock()
            .map_err(|_| AppError::Ocr("OCR 引擎锁损坏".to_string()))? = Some(engine);
        Ok(())
    }
}

#[async_trait]
impl OcrProvider for NativeRustOcrProvider {
    fn name(&self) -> &'static str {
        "native-rust"
    }

    async fn recognize(
        &self,
        image_bytes: &[u8],
        _expected_sum: Option<u32>,
    ) -> AppResult<String> {
        self.ensure_engine()?;
        let engine = Arc::clone(&self.engine);
        let image_bytes = image_bytes.to_vec();
        let raw_text = tokio::task::spawn_blocking(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|err| AppError::Ocr(format!("构建 OCR 运行时失败：{err}")))?;
            let mut guard = engine
                .lock()
                .map_err(|_| AppError::Ocr("OCR 引擎锁损坏".to_string()))?;
            let Some(ocr) = guard.as_mut() else {
                return Err(AppError::Ocr("ddddocr 未初始化".to_string()));
            };
            runtime
                .block_on(ocr.classification(&image_bytes))
                .map_err(|err| AppError::Ocr(format!("ddddocr 识别失败：{err}")))
        })
        .await
        .map_err(|err| AppError::Ocr(format!("ddddocr 线程失败：{err}")))??;
        Ok(normalize_captcha_text(&raw_text))
    }
}
