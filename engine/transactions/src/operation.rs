use pkgseal_domain::PackageName;
use serde::{Deserialize, Serialize};

/// Narrow, typed transaction operations.
///
/// No generic `run_as_root(command)` — every operation is explicit and
/// auditable. The privileged helper (future `platform/linux`) will only
/// accept these variants, never an arbitrary shell string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum TransactionOperation {
    /// Install a native Arch package (official repository or AUR build result).
    InstallPackage { name: PackageName, version: String },
    /// Remove a native Arch package.
    RemovePackage { name: PackageName },
    /// Upgrade a native Arch package from one version to another.
    UpgradePackage {
        name: PackageName,
        from_version: String,
        to_version: String,
    },
    /// Install a Flatpak application from a remote.
    InstallFlatpak {
        /// Flatpak application ID, e.g. `com.brave.Browser`.
        app_id: String,
        /// Optional version or commit to pin.
        version: Option<String>,
    },
    /// Remove a Flatpak application.
    RemoveFlatpak { app_id: String },
    /// Update a Flatpak application.
    UpdateFlatpak {
        app_id: String,
        from_version: Option<String>,
        to_version: String,
    },
}

impl TransactionOperation {
    /// Human-readable summary without secrets.
    pub fn summary(&self) -> String {
        match self {
            Self::InstallPackage { name, version } => format!("install {name} {version}"),
            Self::RemovePackage { name } => format!("remove {name}"),
            Self::UpgradePackage {
                name,
                from_version,
                to_version,
            } => format!("upgrade {name} {from_version} -> {to_version}"),
            Self::InstallFlatpak { app_id, version } => {
                if let Some(v) = version {
                    format!("install flatpak {app_id} {v}")
                } else {
                    format!("install flatpak {app_id}")
                }
            }
            Self::RemoveFlatpak { app_id } => format!("remove flatpak {app_id}"),
            Self::UpdateFlatpak {
                app_id,
                from_version,
                to_version,
            } => {
                if let Some(from) = from_version {
                    format!("update flatpak {app_id} {from} -> {to_version}")
                } else {
                    format!("update flatpak {app_id} -> {to_version}")
                }
            }
        }
    }

    /// Whether this operation will require elevated privileges on Arch.
    ///
    /// Flatpak user installations may not require privileges; Arch
    /// package operations always do. This is conservative: Flatpak
    /// operations report `false` here and the `TransactionPlan`
    /// decides the aggregate `privileges_required`.
    pub fn requires_privileges(&self) -> bool {
        matches!(
            self,
            Self::InstallPackage { .. } | Self::RemovePackage { .. } | Self::UpgradePackage { .. }
        )
    }

    /// Validates the operation fields (lightweight, no IO).
    pub fn validate(&self) -> Result<(), crate::error::TransactionError> {
        match self {
            Self::InstallPackage { name: _, version } => {
                if version.trim().is_empty() {
                    return Err(crate::error::TransactionError::validation(
                        "InstallPackage version cannot be empty",
                    ));
                }
                Ok(())
            }
            Self::UpgradePackage {
                name: _,
                from_version,
                to_version,
            } => {
                if from_version.trim().is_empty() || to_version.trim().is_empty() {
                    return Err(crate::error::TransactionError::validation(
                        "UpgradePackage versions cannot be empty",
                    ));
                }
                Ok(())
            }
            Self::InstallFlatpak { app_id, version: _ }
            | Self::RemoveFlatpak { app_id }
            | Self::UpdateFlatpak { app_id, .. } => {
                validate_flatpak_app_id(app_id)?;
                Ok(())
            }
            Self::RemovePackage { name: _ } => Ok(()),
        }
    }
}

fn validate_flatpak_app_id(app_id: &str) -> Result<(), crate::error::TransactionError> {
    if app_id.trim().is_empty() {
        return Err(crate::error::TransactionError::validation(
            "Flatpak app_id cannot be empty",
        ));
    }
    // Minimal reverse-DNS check: at least two dot-separated components, allowed chars a-zA-Z0-9_-
    let parts: Vec<&str> = app_id.split('.').collect();
    if parts.len() < 2 {
        return Err(crate::error::TransactionError::validation(format!(
            "Flatpak app_id must be reverse-DNS, got '{app_id}'"
        )));
    }
    for part in parts {
        if part.is_empty() {
            return Err(crate::error::TransactionError::validation(format!(
                "Flatpak app_id has empty component: '{app_id}'"
            )));
        }
        if !part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::error::TransactionError::validation(format!(
                "Flatpak app_id contains invalid character: '{app_id}'"
            )));
        }
        if part.starts_with('-') || part.starts_with('_') {
            return Err(crate::error::TransactionError::validation(format!(
                "Flatpak app_id component cannot start with '-' or '_': '{app_id}'"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageName;

    #[test]
    fn operation_summary_install() {
        let op = TransactionOperation::InstallPackage {
            name: PackageName::new("brave-bin").unwrap(),
            version: "1.70.0-1".to_string(),
        };
        assert_eq!(op.summary(), "install brave-bin 1.70.0-1");
    }

    #[test]
    fn operation_requires_privileges() {
        let arch = TransactionOperation::InstallPackage {
            name: PackageName::new("brave").unwrap(),
            version: "1.0".to_string(),
        };
        assert!(arch.requires_privileges());
        let flatpak = TransactionOperation::InstallFlatpak {
            app_id: "com.brave.Browser".to_string(),
            version: None,
        };
        assert!(!flatpak.requires_privileges());
    }

    #[test]
    fn flatpak_app_id_validation_ok() {
        let op = TransactionOperation::InstallFlatpak {
            app_id: "com.brave.Browser".to_string(),
            version: None,
        };
        assert!(op.validate().is_ok());
    }

    #[test]
    fn flatpak_app_id_validation_rejects_plain_name() {
        let op = TransactionOperation::InstallFlatpak {
            app_id: "brave".to_string(),
            version: None,
        };
        assert!(op.validate().is_err());
    }

    #[test]
    fn operation_serde_roundtrip() {
        let op = TransactionOperation::RemovePackage {
            name: PackageName::new("firefox").unwrap(),
        };
        let json = serde_json::to_string(&op).unwrap();
        let parsed: TransactionOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, parsed);
    }
}
