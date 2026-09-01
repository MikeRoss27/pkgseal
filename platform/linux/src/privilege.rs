use crate::error::PlatformError;
use pkgseal_domain::PackageName;
use serde::{Deserialize, Serialize};

/// Typed, narrow privileged operations.
///
/// The privileged helper only accepts these variants — never a raw shell
/// command, never `run_as_root(String)`.
///
/// Each variant is validated both in the unprivileged backend (`validate`)
/// and re-validated inside the helper (`revalidate_for_helper`) to prevent
/// confused-deputy attacks where a compromised WebView reuses the backend as
/// an oracle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivilegedRequest {
    InstallArch {
        packages: Vec<PackageName>,
    },
    RemoveArch {
        packages: Vec<PackageName>,
    },
    InstallFlatpak {
        app_id: FlatpakAppId,
        remote: Option<FlatpakRemote>,
    },
    RemoveFlatpak {
        app_id: FlatpakAppId,
    },
    UpdateFlatpak {
        app_ids: Vec<FlatpakAppId>,
    },
    EnableService {
        unit: SystemdUnit,
    },
    DisableService {
        unit: SystemdUnit,
    },
}

/// Validated Flatpak application ID: reverse-DNS, e.g. `com.brave.Browser`
/// or `org.mozilla.firefox`.
///
/// Rules (simplified freedesktop spec):
/// - 3+ dot-separated components or 2+ with at least one dot,
/// - each component starts with alphanumeric, contains alphanumeric, `-`, `_`,
/// - last component may contain more permissive chars but we keep strict,
/// - max 255 chars, no empty components, no shell metachars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlatpakAppId(String);

impl FlatpakAppId {
    pub fn new(s: impl AsRef<str>) -> Result<Self, PlatformError> {
        let s = s.as_ref();
        validate_flatpak_app_id(s)?;
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FlatpakAppId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_flatpak_app_id(s: &str) -> Result<(), PlatformError> {
    if s.is_empty() {
        return Err(PlatformError::InvalidFlatpakAppId(
            "app id cannot be empty".to_string(),
        ));
    }
    if s.len() > 255 {
        return Err(PlatformError::InvalidFlatpakAppId(
            "app id too long".to_string(),
        ));
    }
    if s.contains(';')
        || s.contains('|')
        || s.contains('&')
        || s.contains('$')
        || s.contains('`')
        || s.contains('\n')
        || s.contains('\r')
        || s.contains('\0')
        || s.contains(' ')
        || s.contains('\'')
        || s.contains('"')
        || s.contains('\\')
    {
        return Err(PlatformError::InvalidFlatpakAppId(
            "app id contains forbidden character".to_string(),
        ));
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 2 {
        return Err(PlatformError::InvalidFlatpakAppId(
            "app id must contain at least one dot".to_string(),
        ));
    }
    for part in &parts {
        if part.is_empty() {
            return Err(PlatformError::InvalidFlatpakAppId(
                "app id contains empty component".to_string(),
            ));
        }
        let mut chars = part.chars();
        let first = chars.next().expect("non-empty");
        if !first.is_ascii_alphanumeric() {
            return Err(PlatformError::InvalidFlatpakAppId(format!(
                "component {part:?} must start with alphanumeric"
            )));
        }
        if !part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(PlatformError::InvalidFlatpakAppId(format!(
                "component {part:?} contains invalid character"
            )));
        }
        if part.starts_with('-')
            || part.ends_with('-')
            || part.starts_with('_')
            || part.ends_with('_')
        {
            return Err(PlatformError::InvalidFlatpakAppId(format!(
                "component {part:?} cannot start or end with - or _"
            )));
        }
    }
    Ok(())
}

/// Flatpak remote name, e.g. `flathub`.
///
/// Strict: alphanumeric, `-`, `_`, `.` ; cannot start/end with separator.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlatpakRemote(String);

impl FlatpakRemote {
    pub fn new(s: impl AsRef<str>) -> Result<Self, PlatformError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(PlatformError::InvalidArgument(
                "flatpak remote cannot be empty".to_string(),
            ));
        }
        if s.len() > 64 {
            return Err(PlatformError::InvalidArgument(
                "flatpak remote too long".to_string(),
            ));
        }
        if !s.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.'
        }) {
            return Err(PlatformError::InvalidArgument(format!(
                "invalid flatpak remote {s:?}"
            )));
        }
        if s.starts_with(['-', '.', '_']) || s.ends_with(['-', '.', '_']) {
            return Err(PlatformError::InvalidArgument(format!(
                "remote {s:?} cannot start or end with separator"
            )));
        }
        // Reuse shell-metachar rejection for defence in depth.
        if s.contains(';') || s.contains('|') || s.contains('$') || s.contains('`') {
            return Err(PlatformError::InvalidArgument(format!(
                "remote {s:?} contains forbidden character"
            )));
        }
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FlatpakRemote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Systemd unit name, e.g. `bluetooth.service`.
///
/// Very strict allow-list: alphanumeric, `-`, `_`, `.`, `@` and must end
/// with a known suffix.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemdUnit(String);

impl SystemdUnit {
    const ALLOWED_SUFFIXES: &'static [&'static str] =
        &[".service", ".socket", ".timer", ".target", ".mount"];

    pub fn new(s: impl AsRef<str>) -> Result<Self, PlatformError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(PlatformError::InvalidArgument(
                "systemd unit cannot be empty".to_string(),
            ));
        }
        if s.len() > 256 {
            return Err(PlatformError::InvalidArgument(
                "systemd unit too long".to_string(),
            ));
        }
        if s.contains('/')
            || s.contains(';')
            || s.contains('|')
            || s.contains('&')
            || s.contains('$')
            || s.contains('`')
            || s.contains('\n')
            || s.contains('\0')
            || s.contains(' ')
        {
            return Err(PlatformError::InvalidArgument(format!(
                "unit {s:?} contains forbidden character"
            )));
        }
        let has_suffix = Self::ALLOWED_SUFFIXES.iter().any(|suf| s.ends_with(suf));
        if !has_suffix {
            return Err(PlatformError::InvalidArgument(format!(
                "unit {s:?} must end with one of {:?}",
                Self::ALLOWED_SUFFIXES
            )));
        }
        // Validate name part before suffix
        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@')
        {
            return Err(PlatformError::InvalidArgument(format!(
                "unit {s:?} contains invalid character"
            )));
        }
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SystemdUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PrivilegedRequest {
    /// Validate the request in the unprivileged backend before it is sent to
    /// the helper. Checks package list bounds, duplicates, names, etc.
    pub fn validate(&self) -> Result<(), PlatformError> {
        match self {
            Self::InstallArch { packages } | Self::RemoveArch { packages } => {
                Self::validate_package_list(packages)
            }
            Self::InstallFlatpak { app_id: _, remote } => {
                if let Some(r) = remote {
                    // remote already validated on construction, but re-check
                    FlatpakRemote::new(r.as_str()).map(|_| ())?;
                }
                Ok(())
            }
            Self::RemoveFlatpak { .. } => Ok(()),
            Self::UpdateFlatpak { app_ids } => {
                if app_ids.is_empty() {
                    return Err(PlatformError::invalid_privileged_request(
                        "update requires at least one app id",
                    ));
                }
                if app_ids.len() > 64 {
                    return Err(PlatformError::invalid_privileged_request(
                        "too many flatpak app ids (max 64)",
                    ));
                }
                // Deduplicate check
                let mut seen = std::collections::HashSet::new();
                for id in app_ids {
                    if !seen.insert(id.as_str()) {
                        return Err(PlatformError::invalid_privileged_request(format!(
                            "duplicate app id {id}"
                        )));
                    }
                }
                Ok(())
            }
            Self::EnableService { unit } | Self::DisableService { unit } => {
                // unit already validated on construction
                SystemdUnit::new(unit.as_str()).map(|_| ())?;
                Ok(())
            }
        }
    }

    fn validate_package_list(packages: &[PackageName]) -> Result<(), PlatformError> {
        if packages.is_empty() {
            return Err(PlatformError::invalid_privileged_request(
                "package list cannot be empty",
            ));
        }
        if packages.len() > 128 {
            return Err(PlatformError::invalid_privileged_request(
                "too many packages (max 128)",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for pkg in packages {
            if !seen.insert(pkg.as_str().to_owned()) {
                return Err(PlatformError::invalid_privileged_request(format!(
                    "duplicate package {}",
                    pkg.as_str()
                )));
            }
        }
        Ok(())
    }

    /// Re-validate inside the privileged helper before any mutation.
    ///
    /// This is intentionally stricter: it re-runs `validate` and additionally
    /// checks that package names contain no path traversal and are not on a
    /// deny-list (e.g. empty, reserved).
    pub fn revalidate_for_helper(&self) -> Result<ValidatedPrivilegedRequest, PlatformError> {
        self.validate()?;

        // Extra helper-side checks.
        match self {
            Self::InstallArch { packages } | Self::RemoveArch { packages } => {
                for pkg in packages {
                    let name = pkg.as_str();
                    if name.contains('/') || name.contains('\0') {
                        return Err(PlatformError::invalid_privileged_request(format!(
                            "package {name:?} contains path separator"
                        )));
                    }
                    // Deny-list for obviously dangerous names (defence in depth).
                    const DENY: &[&str] = &["", ".", ".."];
                    if DENY.contains(&name) {
                        return Err(PlatformError::invalid_privileged_request(format!(
                            "package {name:?} is denied"
                        )));
                    }
                }
            }
            Self::InstallFlatpak { .. }
            | Self::RemoveFlatpak { .. }
            | Self::UpdateFlatpak { .. } => {
                // FlatpakAppId already validated; helper re-checks via new()
                // to ensure deserialization didn't bypass validation (e.g. if
                // serde feature is abused). We do it explicitly.
                match self {
                    Self::InstallFlatpak { app_id, .. } | Self::RemoveFlatpak { app_id } => {
                        FlatpakAppId::new(app_id.as_str()).map(|_| ())?;
                    }
                    Self::UpdateFlatpak { app_ids } => {
                        for id in app_ids {
                            FlatpakAppId::new(id.as_str()).map(|_| ())?;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Self::EnableService { unit } | Self::DisableService { unit } => {
                SystemdUnit::new(unit.as_str()).map(|_| ())?;
            }
        }

        Ok(ValidatedPrivilegedRequest {
            inner: self.clone(),
        })
    }

    /// Stable string identifier for Polkit action mapping.
    #[must_use]
    pub fn action_id(&self) -> &'static str {
        match self {
            Self::InstallArch { .. } => "org.pkgseal.install-arch",
            Self::RemoveArch { .. } => "org.pkgseal.remove-arch",
            Self::InstallFlatpak { .. } => "org.pkgseal.install-flatpak",
            Self::RemoveFlatpak { .. } => "org.pkgseal.remove-flatpak",
            Self::UpdateFlatpak { .. } => "org.pkgseal.update-flatpak",
            Self::EnableService { .. } => "org.pkgseal.enable-service",
            Self::DisableService { .. } => "org.pkgseal.disable-service",
        }
    }

    /// Human-readable description for confirmation UI and audit logs.
    /// Never includes raw shell snippets.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::InstallArch { packages } => format!(
                "Install Arch packages: {}",
                packages
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::RemoveArch { packages } => format!(
                "Remove Arch packages: {}",
                packages
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::InstallFlatpak { app_id, remote } => {
                if let Some(r) = remote {
                    format!("Install Flatpak {app_id} from {r}")
                } else {
                    format!("Install Flatpak {app_id}")
                }
            }
            Self::RemoveFlatpak { app_id } => format!("Remove Flatpak {app_id}"),
            Self::UpdateFlatpak { app_ids } => format!(
                "Update Flatpak: {}",
                app_ids
                    .iter()
                    .map(|a| a.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::EnableService { unit } => format!("Enable service {unit}"),
            Self::DisableService { unit } => format!("Disable service {unit}"),
        }
    }
}

/// Wrapper returned by `revalidate_for_helper` — proves the request was
/// checked inside the helper before execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPrivilegedRequest {
    inner: PrivilegedRequest,
}

impl ValidatedPrivilegedRequest {
    #[must_use]
    pub fn inner(&self) -> &PrivilegedRequest {
        &self.inner
    }

    #[must_use]
    pub fn into_inner(self) -> PrivilegedRequest {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(s: &str) -> PackageName {
        PackageName::new(s).unwrap()
    }

    #[test]
    fn valid_install_arch() {
        let req = PrivilegedRequest::InstallArch {
            packages: vec![pkg("brave-bin"), pkg("firefox")],
        };
        assert!(req.validate().is_ok());
        assert!(req.revalidate_for_helper().is_ok());
        assert_eq!(req.action_id(), "org.pkgseal.install-arch");
    }

    #[test]
    fn install_arch_rejects_empty() {
        let req = PrivilegedRequest::InstallArch { packages: vec![] };
        assert!(req.validate().is_err());
    }

    #[test]
    fn install_arch_rejects_duplicate() {
        let req = PrivilegedRequest::InstallArch {
            packages: vec![pkg("brave-bin"), pkg("brave-bin")],
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn install_arch_rejects_too_many() {
        let pkgs = (0..129)
            .map(|i| pkg(&format!("pkg{i}")))
            .collect::<Vec<_>>();
        let req = PrivilegedRequest::InstallArch { packages: pkgs };
        assert!(req.validate().is_err());
    }

    #[test]
    fn flatpak_app_id_valid() {
        assert!(FlatpakAppId::new("com.brave.Browser").is_ok());
        assert!(FlatpakAppId::new("org.mozilla.firefox").is_ok());
        assert!(FlatpakAppId::new("com.example.App_1").is_ok());
    }

    #[test]
    fn flatpak_app_id_invalid() {
        assert!(FlatpakAppId::new("").is_err());
        assert!(FlatpakAppId::new("noslash").is_err());
        assert!(FlatpakAppId::new("com..Browser").is_err());
        assert!(FlatpakAppId::new("com.brave.Browser; rm").is_err());
        assert!(FlatpakAppId::new("com.brave.Browser$").is_err());
        assert!(FlatpakAppId::new(".com.brave").is_err());
        let long = "a".repeat(300) + ".b.c";
        assert!(FlatpakAppId::new(long).is_err());
    }

    #[test]
    fn flatpak_remote_valid() {
        assert!(FlatpakRemote::new("flathub").is_ok());
        assert!(FlatpakRemote::new("my-remote_1").is_ok());
    }

    #[test]
    fn flatpak_remote_invalid() {
        assert!(FlatpakRemote::new("").is_err());
        assert!(FlatpakRemote::new("-bad").is_err());
        assert!(FlatpakRemote::new("bad-").is_err());
        assert!(FlatpakRemote::new("bad;remote").is_err());
        assert!(FlatpakRemote::new("BAD").is_err());
    }

    #[test]
    fn systemd_unit_valid() {
        assert!(SystemdUnit::new("bluetooth.service").is_ok());
        assert!(SystemdUnit::new("my-app.socket").is_ok());
        assert!(SystemdUnit::new("foo@bar.service").is_ok());
    }

    #[test]
    fn systemd_unit_invalid() {
        assert!(SystemdUnit::new("bad").is_err());
        assert!(SystemdUnit::new("bad;service.service").is_err());
        assert!(SystemdUnit::new("path/to.service").is_err());
        assert!(SystemdUnit::new("evil.service ").is_err());
    }

    #[test]
    fn install_flatpak_validate() {
        let req = PrivilegedRequest::InstallFlatpak {
            app_id: FlatpakAppId::new("com.brave.Browser").unwrap(),
            remote: Some(FlatpakRemote::new("flathub").unwrap()),
        };
        assert!(req.validate().is_ok());
        assert!(req.revalidate_for_helper().is_ok());
    }

    #[test]
    fn update_flatpak_rejects_empty_and_duplicates() {
        let req = PrivilegedRequest::UpdateFlatpak { app_ids: vec![] };
        assert!(req.validate().is_err());
        let req2 = PrivilegedRequest::UpdateFlatpak {
            app_ids: vec![
                FlatpakAppId::new("com.example.App").unwrap(),
                FlatpakAppId::new("com.example.App").unwrap(),
            ],
        };
        assert!(req2.validate().is_err());
    }

    #[test]
    fn describe_does_not_contain_shell() {
        let req = PrivilegedRequest::InstallArch {
            packages: vec![pkg("brave-bin")],
        };
        let desc = req.describe();
        assert!(!desc.contains(';'));
        assert!(!desc.contains("$("));
    }

    #[test]
    fn serialization_roundtrip() {
        let req = PrivilegedRequest::InstallArch {
            packages: vec![pkg("brave-bin")],
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: PrivilegedRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }
}
