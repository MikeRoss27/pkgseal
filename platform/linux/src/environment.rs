use crate::error::PlatformError;
use std::collections::HashMap;

/// System environment snapshot (subset) — used to control child process env.
///
/// Mirrors `docs/architecture/overview.md §35` — `platform/linux` owns
/// how environment variables are forwarded to package manager helpers.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentSnapshot {
    vars: HashMap<String, String>,
}

impl EnvironmentSnapshot {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), PlatformError> {
        let k = key.into();
        let v = value.into();
        if k.is_empty() || k.contains('=') || k.contains('\0') || v.contains('\0') {
            return Err(PlatformError::environment(format!("invalid env var {k:?}")));
        }
        self.vars.insert(k, v);
        Ok(())
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Allow-list of env vars forwarded to pacman/flatpak helpers.
    #[must_use]
    pub fn allow_list() -> &'static [&'static str] {
        &[
            "HOME",
            "USER",
            "LOGNAME",
            "LANG",
            "LC_ALL",
            "LC_MESSAGES",
            "PATH",
            "XDG_CACHE_HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
        ]
    }

    /// Build from current process env, filtering by allow-list.
    #[must_use]
    pub fn capture_allowed() -> Self {
        let mut snap = Self::new();
        for &key in Self::allow_list() {
            if let Ok(val) = std::env::var(key) {
                let _ = snap.insert(key, val);
            }
        }
        snap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut env = EnvironmentSnapshot::new();
        env.insert("HOME", "/home/test").unwrap();
        assert_eq!(env.get("HOME"), Some(&"/home/test".to_string()));
    }

    #[test]
    fn rejects_null() {
        let mut env = EnvironmentSnapshot::new();
        assert!(env.insert("BAD\0", "val").is_err());
        assert!(env.insert("KEY", "val\0").is_err());
    }

    #[test]
    fn allow_list_non_empty() {
        assert!(!EnvironmentSnapshot::allow_list().is_empty());
    }
}
