use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::collections::HashMap;
use keyring::Entry;
use crate::error::AppError;
use crate::settings::SettingsManager;

const SERVICE_NAME: &str = "zyloflow";

pub struct SecureStorage;

impl SecureStorage {
    fn key_file_path() -> PathBuf {
        SettingsManager::get_app_dir().join(".vault.bin")
    }

    /// Machine-specific obfuscation key derived from environment
    fn get_machine_salt() -> Vec<u8> {
        let user = std::env::var("USER").unwrap_or_else(|_| "zyloflow_user".to_string());
        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "zyloflow_host".to_string());
        format!("zyloflow:{}:{}", user, host).into_bytes()
    }

    fn cipher(data: &[u8], salt: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &b)| b ^ salt[i % salt.len()])
            .collect()
    }

    pub fn save_key(provider: &str, key: &str) -> Result<(), AppError> {
        let entry_name = format!("api_key_{}", provider.to_lowercase());
        
        // 1. Try OS native keyring (Secret Service / D-Bus on Linux)
        if let Ok(entry) = Entry::new(SERVICE_NAME, &entry_name) {
            if entry.set_password(key).is_ok() {
                return Ok(());
            }
        }

        // 2. Fallback: Obfuscated user-locked vault for environments without D-Bus Secret Service
        let mut vault = Self::read_fallback_vault();
        vault.insert(provider.to_lowercase(), key.to_string());
        Self::write_fallback_vault(&vault)
    }

    pub fn get_key(provider: &str) -> Result<String, AppError> {
        let entry_name = format!("api_key_{}", provider.to_lowercase());

        // 1. Try OS native keyring
        if let Ok(entry) = Entry::new(SERVICE_NAME, &entry_name) {
            if let Ok(pass) = entry.get_password() {
                if !pass.trim().is_empty() {
                    return Ok(pass);
                }
            }
        }

        // 2. Try fallback vault
        let vault = Self::read_fallback_vault();
        if let Some(key) = vault.get(&provider.to_lowercase()) {
            if !key.trim().is_empty() {
                return Ok(key.clone());
            }
        }

        Err(AppError::SecureStorage(format!(
            "No API key found for provider '{}'",
            provider
        )))
    }

    pub fn has_key(provider: &str) -> bool {
        Self::get_key(provider).is_ok()
    }

    pub fn delete_key(provider: &str) -> Result<(), AppError> {
        let entry_name = format!("api_key_{}", provider.to_lowercase());

        if let Ok(entry) = Entry::new(SERVICE_NAME, &entry_name) {
            let _ = entry.delete_password();
        }

        let mut vault = Self::read_fallback_vault();
        vault.remove(&provider.to_lowercase());
        let _ = Self::write_fallback_vault(&vault);
        Ok(())
    }

    fn read_fallback_vault() -> HashMap<String, String> {
        let path = Self::key_file_path();
        if !path.exists() {
            return HashMap::new();
        }

        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return HashMap::new(),
        };

        let mut encrypted = Vec::new();
        if file.read_to_end(&mut encrypted).is_err() {
            return HashMap::new();
        }

        let salt = Self::get_machine_salt();
        let decrypted = Self::cipher(&encrypted, &salt);
        if let Ok(s) = String::from_utf8(decrypted) {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    fn write_fallback_vault(vault: &HashMap<String, String>) -> Result<(), AppError> {
        let path = Self::key_file_path();
        let json_str = serde_json::to_string(vault)?;
        let salt = Self::get_machine_salt();
        let encrypted = Self::cipher(json_str.as_bytes(), &salt);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }

        file.write_all(&encrypted)?;
        Ok(())
    }
}
