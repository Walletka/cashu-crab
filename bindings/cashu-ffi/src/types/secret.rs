use std::{ops::Deref, str::FromStr};
use crate::error::Result;
use cashu::secret::Secret as SecretSdk;

pub struct Secret {
    inner: SecretSdk,
}

impl Deref for Secret {
    type Target = SecretSdk;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for Secret {
    fn default() -> Self {
        Self::new()
    }
}

impl Secret {
    pub fn new() -> Self {
        Self {
            inner: SecretSdk::new(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }

    pub fn from_string(secret: String) -> Result<Self> {
        Ok(Self {
            inner: SecretSdk::from_str(&secret).unwrap(),
        })
    }
}

impl From<SecretSdk> for Secret {
    fn from(inner: SecretSdk) -> Secret {
        Secret { inner }
    }
}

impl From<Secret> for SecretSdk {
    fn from(secret: Secret) -> SecretSdk {
        secret.inner
    }
}
