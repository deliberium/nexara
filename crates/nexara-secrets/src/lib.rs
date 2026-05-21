use aes_gcm_siv::aead::{Aead, KeyInit};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretValue([redacted])")
    }
}

pub trait SecretStore: Send + Sync {
    fn get(&self, name: &str) -> anyhow::Result<Option<SecretValue>>;
    fn put(&self, name: &str, value: SecretValue) -> anyhow::Result<()>;
    fn delete(&self, name: &str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AsyncSecretStore: Send + Sync {
    async fn get(&self, name: &str) -> anyhow::Result<Option<SecretValue>>;
    async fn put(&self, name: &str, value: SecretValue) -> anyhow::Result<()>;
    async fn delete(&self, name: &str) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct MemorySecretStore {
    values: Arc<Mutex<HashMap<String, SecretValue>>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, name: &str) -> anyhow::Result<Option<SecretValue>> {
        Ok(self.values.lock().unwrap().get(name).cloned())
    }

    fn put(&self, name: &str, value: SecretValue) -> anyhow::Result<()> {
        self.values.lock().unwrap().insert(name.to_string(), value);
        Ok(())
    }

    fn delete(&self, name: &str) -> anyhow::Result<()> {
        self.values.lock().unwrap().remove(name);
        Ok(())
    }
}

#[async_trait]
impl AsyncSecretStore for MemorySecretStore {
    async fn get(&self, name: &str) -> anyhow::Result<Option<SecretValue>> {
        SecretStore::get(self, name)
    }

    async fn put(&self, name: &str, value: SecretValue) -> anyhow::Result<()> {
        SecretStore::put(self, name, value)
    }

    async fn delete(&self, name: &str) -> anyhow::Result<()> {
        SecretStore::delete(self, name)
    }
}

#[derive(Debug, Clone)]
pub struct EncryptedFileSecretStore {
    path: PathBuf,
    key_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct SecretEnvelope {
    nonce_b64: String,
    ciphertext_b64: String,
}

impl EncryptedFileSecretStore {
    pub fn new(path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            key_path: key_path.into(),
        }
    }

    fn load_all(&self) -> anyhow::Result<HashMap<String, String>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let envelope: SecretEnvelope = serde_json::from_slice(&fs::read(&self.path)?)?;
        let nonce_bytes = BASE64.decode(envelope.nonce_b64)?;
        let ciphertext = BASE64.decode(envelope.ciphertext_b64)?;
        let cipher = Aes256GcmSiv::new_from_slice(&self.load_or_create_key()?)?;
        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("failed decrypting Nexara secrets"))?;
        Ok(serde_json::from_slice(&plaintext)?)
    }

    fn save_all(&self, data: &HashMap<String, String>) -> anyhow::Result<()> {
        self.ensure_parent_dirs()?;
        let cipher = Aes256GcmSiv::new_from_slice(&self.load_or_create_key()?)?;
        let nonce_uuid = Uuid::new_v4();
        let nonce = &nonce_uuid.as_bytes()[..12];
        let plaintext = serde_json::to_vec(data)?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(nonce), plaintext.as_ref())
            .map_err(|_| anyhow::anyhow!("failed encrypting Nexara secrets"))?;
        let envelope = SecretEnvelope {
            nonce_b64: BASE64.encode(nonce),
            ciphertext_b64: BASE64.encode(ciphertext),
        };
        fs::write(&self.path, serde_json::to_vec(&envelope)?)?;
        Ok(())
    }

    fn load_or_create_key(&self) -> anyhow::Result<[u8; 32]> {
        self.ensure_parent_dirs()?;
        let key_bytes = if self.key_path.exists() {
            fs::read(&self.key_path)?
        } else {
            let seed = Uuid::new_v4();
            let digest = Sha256::digest(seed.as_bytes());
            fs::write(&self.key_path, digest.as_slice())?;
            digest.to_vec()
        };
        let mut key = [0_u8; 32];
        let digest = Sha256::digest(&key_bytes);
        key.copy_from_slice(&digest[..32]);
        Ok(key)
    }

    fn ensure_parent_dirs(&self) -> anyhow::Result<()> {
        ensure_parent_dir(&self.path)?;
        ensure_parent_dir(&self.key_path)?;
        Ok(())
    }
}

impl SecretStore for EncryptedFileSecretStore {
    fn get(&self, name: &str) -> anyhow::Result<Option<SecretValue>> {
        Ok(self.load_all()?.remove(name).map(SecretValue::new))
    }

    fn put(&self, name: &str, value: SecretValue) -> anyhow::Result<()> {
        let mut data = self.load_all()?;
        data.insert(name.to_string(), value.expose().to_string());
        self.save_all(&data)
    }

    fn delete(&self, name: &str) -> anyhow::Result<()> {
        let mut data = self.load_all()?;
        data.remove(name);
        self.save_all(&data)
    }
}

#[async_trait]
impl AsyncSecretStore for EncryptedFileSecretStore {
    async fn get(&self, name: &str) -> anyhow::Result<Option<SecretValue>> {
        SecretStore::get(self, name)
    }

    async fn put(&self, name: &str, value: SecretValue) -> anyhow::Result<()> {
        SecretStore::put(self, name, value)
    }

    async fn delete(&self, name: &str) -> anyhow::Result<()> {
        SecretStore::delete(self, name)
    }
}

fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
