use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::ocr::provider::{normalize_captcha_text, OcrProvider};

pub struct NativeRustOcrProvider {
    detection_model_path: PathBuf,
    recognition_model_path: PathBuf,
    engine: Mutex<Option<OcrEngine>>,
}

impl NativeRustOcrProvider {
    pub fn new(detection_model_path: PathBuf, recognition_model_path: PathBuf) -> Self {
        Self {
            detection_model_path,
            recognition_model_path,
            engine: Mutex::new(None),
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
        if !Path::new(&self.detection_model_path).exists()
            || !Path::new(&self.recognition_model_path).exists()
        {
            return Err(AppError::Ocr("原生 OCR 模型文件缺失".to_string()));
        }
        let detection_model = Model::load_file(&self.detection_model_path)
            .map_err(|err| AppError::Ocr(format!("加载 OCR 检测模型失败：{err}")))?;
        let recognition_model = Model::load_file(&self.recognition_model_path)
            .map_err(|err| AppError::Ocr(format!("加载 OCR 识别模型失败：{err}")))?;
        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            allowed_chars: Some(
                "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".to_string(),
            ),
            ..Default::default()
        })
        .map_err(|err| AppError::Ocr(format!("初始化原生 OCR 失败：{err}")))?;
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

    async fn recognize(&self, image_bytes: &[u8]) -> AppResult<String> {
        self.ensure_engine()?;
        let image_bytes = image_bytes.to_vec();
        let image_data =
            tokio::task::spawn_blocking(move || -> AppResult<(Vec<u8>, (u32, u32))> {
                let img = image::load_from_memory(&image_bytes)
                    .map_err(|err| AppError::Ocr(format!("读取验证码图片失败：{err}")))?
                    .into_rgb8();
                Ok((img.as_raw().clone(), img.dimensions()))
            })
            .await
            .map_err(|err| AppError::Ocr(format!("原生 OCR 线程失败：{err}")))??;

        let guard = self
            .engine
            .lock()
            .map_err(|_| AppError::Ocr("OCR 引擎锁损坏".to_string()))?;
        let Some(engine) = guard.as_ref() else {
            return Err(AppError::Ocr("原生 OCR 未初始化".to_string()));
        };
        let (pixels, dimensions) = image_data;
        let source = ImageSource::from_bytes(&pixels, dimensions)
            .map_err(|err| AppError::Ocr(format!("验证码图片格式不支持：{err}")))?;
        let input = engine
            .prepare_input(source)
            .map_err(|err| AppError::Ocr(format!("原生 OCR 预处理失败：{err}")))?;
        let raw_text = engine
            .get_text(&input)
            .map_err(|err| AppError::Ocr(format!("原生 OCR 识别失败：{err}")))?;
        Ok(normalize_captcha_text(&raw_text))
    }
}
