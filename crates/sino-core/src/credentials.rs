use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const CREDENTIALS_DIR: &str = "sino";
const CREDENTIALS_FILE: &str = "credentials.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum Credential {
    UserId {
        user_id: String,
    },
    OauthToken {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
    Password {
        username: String,
        password: String,
    },
}

impl Credential {
    pub fn display_summary(&self) -> String {
        match self {
            Credential::UserId { user_id } => {
                format!("user_id: {}", mask_value(user_id))
            }
            Credential::OauthToken {
                access_token,
                expires_at,
                ..
            } => {
                let masked = mask_value(access_token);
                match expires_at {
                    Some(exp) => {
                        let now = Utc::now();
                        if *exp < now {
                            format!("token: 已过期 {}, 请重新配置", exp.format("%Y-%m-%d"))
                        } else {
                            let remaining = *exp - now;
                            if remaining.num_hours() < 24 {
                                format!("token: {masked}, 将于明天过期")
                            } else {
                                format!(
                                    "token: {masked}, 有效至 {}",
                                    exp.format("%Y-%m-%d")
                                )
                            }
                        }
                    }
                    None => format!("token: {masked}"),
                }
            }
            Credential::Password { username, .. } => {
                format!("username: {}", mask_value(username))
            }
        }
    }

    pub fn is_expired(&self) -> bool {
        match self {
            Credential::OauthToken {
                expires_at: Some(exp),
                ..
            } => *exp < Utc::now(),
            _ => false,
        }
    }

    pub fn has_refresh_token(&self) -> bool {
        matches!(
            self,
            Credential::OauthToken {
                refresh_token: Some(_),
                ..
            }
        )
    }
}

fn mask_value(value: &str) -> String {
    let len = value.len();
    if len <= 4 {
        return "*".repeat(len);
    }
    let prefix = &value[..3];
    let suffix = &value[len - 2..];
    format!("{prefix}***{suffix}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStore {
    pub version: u32,
    pub sources: BTreeMap<String, Credential>,
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self {
            version: 1,
            sources: BTreeMap::new(),
        }
    }
}

pub struct CredentialManager {
    path: PathBuf,
    store: CredentialStore,
}

impl CredentialManager {
    pub fn load() -> Result<Self> {
        let path = Self::credentials_path()?;
        let store = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("无法读取凭证文件: {}", path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("凭证文件格式错误: {}", path.display()))?
        } else {
            CredentialStore::default()
        };
        Ok(Self { path, store })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建目录: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(&self.store)?;
        fs::write(&self.path, &content)
            .with_context(|| format!("无法写入凭证文件: {}", self.path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.path, perms)?;
        }

        Ok(())
    }

    pub fn get(&self, source: &str) -> Option<&Credential> {
        self.store.sources.get(source)
    }

    /// Returns the old credential if one existed (for upsert messaging).
    pub fn set(&mut self, source: &str, credential: Credential) -> Option<Credential> {
        self.store
            .sources
            .insert(source.to_string(), credential)
    }

    pub fn remove(&mut self, source: &str) -> Option<Credential> {
        self.store.sources.remove(source)
    }

    pub fn list(&self) -> &BTreeMap<String, Credential> {
        &self.store.sources
    }

    pub fn get_sinocare_user_id(&self) -> Option<&str> {
        match self.store.sources.get("sinocare") {
            Some(Credential::UserId { user_id }) => Some(user_id.as_str()),
            _ => None,
        }
    }

    fn credentials_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("无法确定配置目录路径"))?;
        Ok(config_dir.join(CREDENTIALS_DIR).join(CREDENTIALS_FILE))
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_value() {
        assert_eq!(mask_value("abc123"), "abc***23");
        assert_eq!(mask_value("ab"), "**");
        assert_eq!(mask_value("abcdefghij"), "abc***ij");
    }

    #[test]
    fn test_credential_serialization() {
        let cred = Credential::UserId {
            user_id: "test123".to_string(),
        };
        let json = serde_json::to_string(&cred).unwrap();
        let deserialized: Credential = serde_json::from_str(&json).unwrap();
        match deserialized {
            Credential::UserId { user_id } => assert_eq!(user_id, "test123"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_credential_store_default() {
        let store = CredentialStore::default();
        assert_eq!(store.version, 1);
        assert!(store.sources.is_empty());
    }
}
