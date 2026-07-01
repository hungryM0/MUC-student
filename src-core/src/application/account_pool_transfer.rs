use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::application::error::{AppError, AppResult};
use crate::domain::models::CachedTrafficSnapshot;

const CODE_PREFIX: &str = "MUCPOOL1.";
const ASSOCIATED_DATA: &[u8] = b"MUC-student account pool v1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const MIN_PAYLOAD_LEN: usize = SALT_LEN + NONCE_LEN + 16;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountPoolState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_online_username: Option<String>,
    #[serde(default)]
    pub status_card_order_usernames: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountPoolPlaintext {
    pub version: u8,
    pub accounts: Vec<AccountPoolEntry>,
    #[serde(default)]
    pub state: AccountPoolState,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountPoolEntry {
    pub remark_name: String,
    pub username: String,
    pub password: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_traffic_snapshot: Option<CachedTrafficSnapshot>,
}

pub fn encode_account_pool(
    accounts: Vec<AccountPoolEntry>,
    state: AccountPoolState,
    passphrase: &str,
) -> AppResult<String> {
    require_passphrase(passphrase)?;

    let plaintext = AccountPoolPlaintext {
        version: 1,
        accounts,
        state,
    };
    let plaintext = serde_json::to_vec(&plaintext)
        .map_err(|err| AppError::Internal(format!("序列化号池失败：{err}")))?;

    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|err| AppError::Internal(format!("初始化号池加密失败：{err}")))?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: ASSOCIATED_DATA,
            },
        )
        .map_err(|_| AppError::Internal("加密号池失败".to_string()))?;

    let mut payload = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);

    Ok(format!("{CODE_PREFIX}{}", URL_SAFE_NO_PAD.encode(payload)))
}

pub fn decode_account_pool(code: &str, passphrase: &str) -> AppResult<AccountPoolPlaintext> {
    require_passphrase(passphrase)?;

    let body = code
        .trim()
        .strip_prefix(CODE_PREFIX)
        .ok_or_else(|| AppError::Validation("号池码格式不对".to_string()))?;
    let payload = URL_SAFE_NO_PAD
        .decode(body)
        .map_err(|_| AppError::Validation("号池码格式不对".to_string()))?;
    if payload.len() < MIN_PAYLOAD_LEN {
        return Err(AppError::Validation("号池码不完整".to_string()));
    }

    let salt = &payload[..SALT_LEN];
    let nonce = &payload[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &payload[SALT_LEN + NONCE_LEN..];
    let key = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|err| AppError::Internal(format!("初始化号池解密失败：{err}")))?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: ASSOCIATED_DATA,
            },
        )
        .map_err(|_| AppError::Validation("号池码或口令不对".to_string()))?;

    let decoded: AccountPoolPlaintext = serde_json::from_slice(&plaintext)
        .map_err(|_| AppError::Validation("号池码内容不兼容".to_string()))?;
    if decoded.version != 1 {
        return Err(AppError::Validation("不支持这个号池码版本".to_string()));
    }
    Ok(decoded)
}

fn require_passphrase(passphrase: &str) -> AppResult<()> {
    if passphrase.trim().is_empty() {
        Err(AppError::Validation("加密令牌不能为空".to_string()))
    } else {
        Ok(())
    }
}

fn derive_key(passphrase: &str, salt: &[u8]) -> AppResult<[u8; KEY_LEN]> {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase.trim().as_bytes(), salt, &mut key)
        .map_err(|err| AppError::Internal(format!("派生号池密钥失败：{err}")))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::{decode_account_pool, encode_account_pool, AccountPoolEntry};

    #[test]
    fn roundtrips_encrypted_pool_code() {
        let accounts = vec![AccountPoolEntry {
            remark_name: "主号".to_string(),
            username: "20260001".to_string(),
            password: "secret-1".to_string(),
            cached_traffic_snapshot: None,
        }];

        let code =
            encode_account_pool(accounts.clone(), Default::default(), "share-pass").expect("encode");
        assert!(code.starts_with("MUCPOOL1."));
        assert!(!code.contains("20260001"));
        assert!(!code.contains("secret-1"));

        let decoded = decode_account_pool(&code, "share-pass").expect("decode");
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.accounts, accounts);
        assert_eq!(decoded.state, Default::default());
    }

    #[test]
    fn rejects_wrong_passphrase() {
        let code = encode_account_pool(
            vec![AccountPoolEntry {
                remark_name: "主号".to_string(),
                username: "20260001".to_string(),
                password: "secret-1".to_string(),
                cached_traffic_snapshot: None,
            }],
            Default::default(),
            "share-pass",
        )
        .expect("encode");

        assert!(decode_account_pool(&code, "wrong-pass").is_err());
    }
}
