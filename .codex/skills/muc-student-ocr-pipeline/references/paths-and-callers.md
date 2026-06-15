# 资源路径

- OCR 资源在 `src-tauri/resources/ocr/`
- 实际路径由 `RuntimePaths` 提供
- `Backend::build` 负责把路径传给 provider

# 调用点

- `AuthPortalClient::verify_login_yii`
- `SelfServicePanelClient::login_yii_with_ocr`

改 OCR 逻辑时，两个调用点都要看。
