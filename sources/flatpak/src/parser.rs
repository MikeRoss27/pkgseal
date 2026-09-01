use pkgseal_domain::PackageName;
use pkgseal_policy::decision::{DbusAccess, FilesystemAccess, PermissionLevel};
use pkgseal_source::dto::{InstalledPackage, PackageSummary};
use pkgseal_source::error::SourceResult;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct FlatpakInfo {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub branch: String,
    pub origin: String,
    pub description: Option<String>,
    pub arch: String,
    pub installed: bool,
    pub installed_size: u64,
    pub download_size: u64,
    pub developer_name: Option<String>,
    pub license: Option<String>,
    pub url: Option<String>,
    pub runtime: String,
    pub runtime_version: String,
    pub sdk: String,
    pub commit: String,
    pub ref_: String,
    pub verification: Option<String>,
    pub permissions: Vec<String>,
    pub parsed_permissions: Option<FlatpakPermissions>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct FlatpakPermissions {
    pub filesystems: Vec<String>,
    pub shared: Vec<String>,
    pub sockets: Vec<String>,
    pub devices: Vec<String>,
    pub features: Vec<String>,
    pub persistent: Vec<String>,
    pub session_bus_policy: HashMap<String, String>,
    pub system_bus_policy: HashMap<String, String>,
    pub unset_environment: Vec<String>,
}

pub fn parse_flatpak_search(output: &str) -> SourceResult<Vec<PackageSummary>> {
    let mut packages = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Name") || line.starts_with("---") {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 5 {
            let app_id = parts[1].trim();
            let version = parts[2].trim();
            let description = parts[3].trim();
            let origin = parts[4].trim();

            let sanitized = app_id.replace('.', "-").to_lowercase();
            let pkg_name = match PackageName::new(&sanitized) {
                Ok(n) => n,
                Err(_) => {
                    // Sanitize fallback failed (rare: empty app_id or boundary chars).
                    // Skip the package to avoid collisions on a generic "invalid" name.
                    tracing::warn!(
                        app_id = %app_id,
                        sanitized = %sanitized,
                        "flatpak search: sanitized app_id is not a valid PackageName, skipping"
                    );
                    continue;
                }
            };
            packages.push(PackageSummary {
                id: format!("flatpak/{}", app_id),
                name: pkg_name,
                version: version.to_string(),
                description: if description.is_empty() {
                    None
                } else {
                    Some(description.to_string())
                },
                source: pkgseal_domain::PackageSource::Flatpak,
                repository: Some(origin.to_string()),
                installed: false,
                download_size: None,
                installed_size: None,
            });
        }
    }

    Ok(packages)
}

pub fn parse_flatpak_info(output: &str) -> SourceResult<FlatpakInfo> {
    let mut info = FlatpakInfo::default();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => info.name = value.to_string(),
                "ID" | "Application ID" => info.application_id = value.to_string(),
                "Version" => info.version = value.to_string(),
                "Branch" => info.branch = value.to_string(),
                "Origin" => info.origin = value.to_string(),
                "Description" => {
                    info.description = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "Arch" => info.arch = value.to_string(),
                "Installed" => info.installed = value == "yes",
                "Installed size" => {
                    info.installed_size = parse_size_kib(value).unwrap_or(0);
                }
                "Download size" => {
                    info.download_size = parse_size_kib(value).unwrap_or(0);
                }
                "Developer" => {
                    info.developer_name = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "License" => {
                    info.license = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "URL" | "Homepage" => {
                    info.url = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "Runtime" => info.runtime = value.to_string(),
                "Runtime version" => info.runtime_version = value.to_string(),
                "Sdk" => info.sdk = value.to_string(),
                "Commit" => info.commit = value.to_string(),
                "Ref" => info.ref_ = value.to_string(),
                "Verification" => {
                    info.verification = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "Permissions" => info.permissions.push(value.to_string()),
                _ => {}
            }
        }
    }

    if info.application_id.is_empty() {
        return Err(pkgseal_source::error::SourceError::parse(
            "No application ID found".to_string(),
        ));
    }

    Ok(info)
}

pub fn parse_flatpak_list(output: &str) -> SourceResult<Vec<InstalledPackage>> {
    let mut packages = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Application ID") || line.starts_with("---") {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let app_id = parts[0].trim();
            let version = parts[1].trim();
            let origin = parts[2].trim();
            let _installation = parts[3].trim();

            if let Ok(pkg_name) = PackageName::new(app_id.replace('.', "-")) {
                packages.push(InstalledPackage {
                    name: pkg_name,
                    version: version.to_string(),
                    source: pkgseal_domain::PackageSource::Flatpak,
                    repository: Some(origin.to_string()),
                    install_date: None,
                    install_reason: Some("user".to_string()),
                    size: None,
                });
            }
        }
    }

    Ok(packages)
}

fn parse_size_kib(s: &str) -> Result<u64, &'static str> {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix(" KB").or_else(|| s.strip_suffix(" kB")) {
        let num: f64 = stripped.trim().parse().map_err(|_| "parse error")?;
        Ok((num * 1024.0) as u64)
    } else if let Some(num_str) = s.strip_suffix(" MB") {
        let num: f64 = num_str.trim().parse().map_err(|_| "parse error")?;
        Ok((num * 1024.0 * 1024.0) as u64)
    } else if let Some(num_str) = s.strip_suffix(" GB") {
        let num: f64 = num_str.trim().parse().map_err(|_| "parse error")?;
        Ok((num * 1024.0 * 1024.0 * 1024.0) as u64)
    } else {
        Err("unknown unit")
    }
}

/// Parses the output of `flatpak info --show-permissions <app-id>` which is in INI-like format.
/// Example input:
/// ```text
/// [Context]
/// shared=network;ipc;
/// sockets=x11;wayland;pulseaudio;fallback-x11;
/// devices=dri;all;
/// filesystems=home;host;host-os;/run/media;
/// ```
pub fn parse_flatpak_permissions(ini_content: &str) -> FlatpakPermissions {
    let mut perms = FlatpakPermissions::default();
    let mut current_section = String::new();

    for line in ini_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_string();
            continue;
        }

        // Key=value pair
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match current_section.as_str() {
                "Context" => {
                    parse_context_key(key, value, &mut perms);
                }
                "Session Bus Policy" => {
                    perms
                        .session_bus_policy
                        .insert(key.to_string(), value.to_string());
                }
                "System Bus Policy" => {
                    perms
                        .system_bus_policy
                        .insert(key.to_string(), value.to_string());
                }
                _ => {}
            }
        }
    }

    perms
}

fn parse_context_key(key: &str, value: &str, perms: &mut FlatpakPermissions) {
    let values: Vec<String> = value
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    match key {
        "shared" => perms.shared.extend(values),
        "sockets" => perms.sockets.extend(values),
        "devices" => perms.devices.extend(values),
        "filesystems" => perms.filesystems.extend(values),
        "features" => perms.features.extend(values),
        "persistent" => perms.persistent.extend(values),
        "unset-environment" => perms.unset_environment.extend(values),
        _ => {}
    }
}

/// Derives the aggregate permission level from parsed Flatpak permissions.
/// Heuristic (deterministic):
/// - Excessive: filesystem=host OR filesystem=host-root OR filesystem=host-etc OR filesystem=host-os (with rw)
/// - Broad: home:rw + (network OR devices=all OR system-bus socket)
/// - Moderate: home (ro or rw) + network OR devices (limited like dri) OR session-bus socket
/// - Narrow: only limited filesystems (xdg-*, ~/, home:ro) + wayland/pulseaudio only + no network
pub fn derive_permission_level(perms: &FlatpakPermissions) -> PermissionLevel {
    let has_host_fs = perms.filesystems.iter().any(|fs| {
        let fs_lower = fs.to_lowercase();
        fs_lower == "host"
            || fs_lower == "host-root"
            || fs_lower == "host-etc"
            || (fs_lower.starts_with("host-os") && fs_lower.contains(":rw"))
    });

    if has_host_fs {
        return PermissionLevel::Excessive;
    }

    let has_home_rw = perms.filesystems.iter().any(|fs| {
        let fs_lower = fs.to_lowercase();
        fs_lower == "home" || fs_lower == "home:rw" || fs_lower == "~" || fs_lower == "~/"
    });

    let has_network = perms
        .shared
        .iter()
        .any(|s| s.eq_ignore_ascii_case("network"));
    let has_devices_all = perms.devices.iter().any(|d| d.eq_ignore_ascii_case("all"));
    let has_system_bus = perms
        .sockets
        .iter()
        .any(|s| s.eq_ignore_ascii_case("system-bus"));
    let has_session_bus_full = perms
        .sockets
        .iter()
        .any(|s| s.eq_ignore_ascii_case("session-bus"));

    let broad_indicators = [
        has_home_rw,
        has_network,
        has_devices_all,
        has_system_bus,
        has_session_bus_full,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if broad_indicators >= 2 {
        return PermissionLevel::Broad;
    }

    let has_home_ro = perms.filesystems.iter().any(|fs| {
        let fs_lower = fs.to_lowercase();
        fs_lower == "home:ro"
    });

    let has_limited_fs = perms.filesystems.iter().any(|fs| {
        let fs_lower = fs.to_lowercase();
        fs_lower.starts_with("xdg-") || fs_lower.starts_with("~/") || fs_lower.starts_with("home/")
    });

    let has_dri_only = perms.devices.iter().all(|d| {
        let d_lower = d.to_lowercase();
        d_lower == "dri" || d_lower == "shm" || d_lower == "kvm"
    });

    let has_wayland_only = perms.sockets.iter().all(|s| {
        let s_lower = s.to_lowercase();
        s_lower == "wayland"
            || s_lower == "pulseaudio"
            || s_lower == "fallback-x11"
            || s_lower == "x11"
    });

    let has_no_network = !has_network;

    // Narrow: only limited filesystems (xdg-*, home:ro, ~/) + wayland/pulseaudio only + no network
    // home:rw alone (even with only dri/wayland) is at least Moderate
    let is_narrow = (has_home_ro || has_limited_fs)
        && !has_home_rw
        && has_dri_only
        && has_wayland_only
        && has_no_network;

    if is_narrow {
        PermissionLevel::Narrow
    } else {
        PermissionLevel::Moderate
    }
}

/// Derives filesystem access level from Flatpak permissions.
pub fn derive_filesystem_access(perms: &FlatpakPermissions) -> FilesystemAccess {
    for fs in &perms.filesystems {
        let fs_lower = fs.to_lowercase();
        if fs_lower == "host" || fs_lower == "host-root" {
            return FilesystemAccess::Host;
        }
        if fs_lower == "host-etc" || (fs_lower.starts_with("host-os") && fs_lower.contains(":rw")) {
            return FilesystemAccess::Host;
        }
        if fs_lower == "home" || fs_lower == "home:rw" || fs_lower == "~" || fs_lower == "~/" {
            return FilesystemAccess::HomeRw;
        }
        if fs_lower == "home:ro" {
            return FilesystemAccess::HomeRo;
        }
    }

    let has_limited = perms.filesystems.iter().any(|fs| {
        let fs_lower = fs.to_lowercase();
        fs_lower.starts_with("xdg-") || fs_lower.starts_with("~/") || fs_lower.starts_with("home/")
    });

    if has_limited {
        FilesystemAccess::Limited
    } else {
        FilesystemAccess::None
    }
}

/// Derives D-Bus access level from Flatpak permissions.
pub fn derive_dbus_access(perms: &FlatpakPermissions) -> DbusAccess {
    // Check for explicit system bus socket
    if perms
        .sockets
        .iter()
        .any(|s| s.eq_ignore_ascii_case("system-bus"))
    {
        // Check if system bus policy allows broad access
        let has_broad_system = perms
            .system_bus_policy
            .values()
            .any(|v| v.eq_ignore_ascii_case("talk") || v.eq_ignore_ascii_case("own"));
        if has_broad_system {
            return DbusAccess::System;
        }
        return DbusAccess::Host; // System bus available but policy restricts
    }

    // Check for session bus socket
    if perms
        .sockets
        .iter()
        .any(|s| s.eq_ignore_ascii_case("session-bus"))
    {
        // Check if session bus policy is restrictive (default policy is narrow)
        let has_broad_session = perms
            .session_bus_policy
            .values()
            .any(|v| v.eq_ignore_ascii_case("talk") || v.eq_ignore_ascii_case("own"));
        if has_broad_session {
            return DbusAccess::SessionFull;
        }
        return DbusAccess::SessionLimited;
    }

    // Check for dbus-related sockets without full bus access
    if perms.sockets.iter().any(|s| {
        let s_lower = s.to_lowercase();
        s_lower == "ssh-auth" || s_lower == "pcsc" || s_lower == "cups" || s_lower == "gpg-agent"
    }) {
        return DbusAccess::SessionLimited;
    }

    DbusAccess::None
}

/// Returns true if network access is granted via shared=network
pub fn derive_network_access(perms: &FlatpakPermissions) -> bool {
    perms
        .shared
        .iter()
        .any(|s| s.eq_ignore_ascii_case("network"))
        || perms
            .sockets
            .iter()
            .any(|s| s.eq_ignore_ascii_case("ssh-auth"))
}

/// Returns true if device access is granted beyond basic dri/shm
pub fn derive_device_access(perms: &FlatpakPermissions) -> bool {
    perms.devices.iter().any(|d| {
        let d_lower = d.to_lowercase();
        d_lower == "all"
            || d_lower == "usb"
            || d_lower == "input"
            || d_lower == "kvm"
            || d_lower == "bluetooth"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_policy::decision::{DbusAccess, FilesystemAccess, PermissionLevel};

    #[test]
    fn parse_permissions_empty() {
        let perms = parse_flatpak_permissions("");
        assert_eq!(perms.filesystems, Vec::<String>::new());
        assert_eq!(perms.shared, Vec::<String>::new());
        assert_eq!(perms.sockets, Vec::<String>::new());
        assert_eq!(perms.devices, Vec::<String>::new());
    }

    #[test]
    fn parse_permissions_basic() {
        let ini = r#"
[Context]
shared=network;ipc;
sockets=x11;wayland;pulseaudio;fallback-x11;
devices=dri;all;
filesystems=home;host;host-os;/run/media;
"#;
        let perms = parse_flatpak_permissions(ini);
        assert_eq!(perms.shared, vec!["network", "ipc"]);
        assert_eq!(
            perms.sockets,
            vec!["x11", "wayland", "pulseaudio", "fallback-x11"]
        );
        assert_eq!(perms.devices, vec!["dri", "all"]);
        assert_eq!(
            perms.filesystems,
            vec!["home", "host", "host-os", "/run/media"]
        );
    }

    #[test]
    fn parse_permissions_with_session_bus_policy() {
        let ini = r#"
[Context]
shared=network;
sockets=wayland;session-bus;
devices=dri;
filesystems=home:ro;

[Session Bus Policy]
org.freedesktop.portal.*=talk
org.gnome.SessionManager=see

[System Bus Policy]
org.freedesktop.login1=talk
"#;
        let perms = parse_flatpak_permissions(ini);
        assert_eq!(perms.shared, vec!["network"]);
        assert_eq!(perms.sockets, vec!["wayland", "session-bus"]);
        assert_eq!(perms.devices, vec!["dri"]);
        assert_eq!(perms.filesystems, vec!["home:ro"]);
        assert_eq!(
            perms.session_bus_policy.get("org.freedesktop.portal.*"),
            Some(&"talk".to_string())
        );
        assert_eq!(
            perms.system_bus_policy.get("org.freedesktop.login1"),
            Some(&"talk".to_string())
        );
    }

    #[test]
    fn derive_permission_level_excessive_host_fs() {
        let perms = FlatpakPermissions {
            filesystems: vec!["host".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Excessive);
    }

    #[test]
    fn derive_permission_level_excessive_host_root() {
        let perms = FlatpakPermissions {
            filesystems: vec!["host-root".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Excessive);
    }

    #[test]
    fn derive_permission_level_broad_home_rw_network() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home".to_string()],
            shared: vec!["network".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Broad);
    }

    #[test]
    fn derive_permission_level_broad_devices_all() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home".to_string()],
            devices: vec!["all".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Broad);
    }

    #[test]
    fn derive_permission_level_broad_system_bus() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home".to_string()],
            sockets: vec!["system-bus".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Broad);
    }

    #[test]
    fn derive_permission_level_narrow_home_ro() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home:ro".to_string()],
            sockets: vec!["wayland".to_string(), "pulseaudio".to_string()],
            devices: vec!["dri".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Narrow);
    }

    #[test]
    fn derive_permission_level_narrow_xdg_dirs() {
        let perms = FlatpakPermissions {
            filesystems: vec!["xdg-download".to_string(), "xdg-documents".to_string()],
            sockets: vec!["wayland".to_string()],
            devices: vec!["dri".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Narrow);
    }

    #[test]
    fn derive_permission_level_moderate_home_rw_only() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home".to_string()],
            sockets: vec!["wayland".to_string()],
            devices: vec!["dri".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_permission_level(&perms), PermissionLevel::Moderate);
    }

    #[test]
    fn derive_filesystem_access_host() {
        let perms = FlatpakPermissions {
            filesystems: vec!["host".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_filesystem_access(&perms), FilesystemAccess::Host);
    }

    #[test]
    fn derive_filesystem_access_home_rw() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_filesystem_access(&perms), FilesystemAccess::HomeRw);
    }

    #[test]
    fn derive_filesystem_access_home_ro() {
        let perms = FlatpakPermissions {
            filesystems: vec!["home:ro".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_filesystem_access(&perms), FilesystemAccess::HomeRo);
    }

    #[test]
    fn derive_filesystem_access_limited_xdg() {
        let perms = FlatpakPermissions {
            filesystems: vec!["xdg-download".to_string(), "xdg-documents".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_filesystem_access(&perms), FilesystemAccess::Limited);
    }

    #[test]
    fn derive_filesystem_access_none() {
        let perms = FlatpakPermissions::default();
        assert_eq!(derive_filesystem_access(&perms), FilesystemAccess::None);
    }

    #[test]
    fn derive_dbus_access_system_full() {
        let perms = FlatpakPermissions {
            sockets: vec!["system-bus".to_string()],
            system_bus_policy: HashMap::from([(
                "org.freedesktop.login1".to_string(),
                "talk".to_string(),
            )]),
            ..Default::default()
        };
        assert_eq!(derive_dbus_access(&perms), DbusAccess::System);
    }

    #[test]
    fn derive_dbus_access_host() {
        let perms = FlatpakPermissions {
            sockets: vec!["system-bus".to_string()],
            system_bus_policy: HashMap::from([(
                "org.freedesktop.login1".to_string(),
                "see".to_string(),
            )]),
            ..Default::default()
        };
        assert_eq!(derive_dbus_access(&perms), DbusAccess::Host);
    }

    #[test]
    fn derive_dbus_access_session_full() {
        let perms = FlatpakPermissions {
            sockets: vec!["session-bus".to_string()],
            session_bus_policy: HashMap::from([(
                "org.freedesktop.portal.*".to_string(),
                "talk".to_string(),
            )]),
            ..Default::default()
        };
        assert_eq!(derive_dbus_access(&perms), DbusAccess::SessionFull);
    }

    #[test]
    fn derive_dbus_access_session_limited() {
        let perms = FlatpakPermissions {
            sockets: vec!["session-bus".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_dbus_access(&perms), DbusAccess::SessionLimited);
    }

    #[test]
    fn derive_dbus_access_none() {
        let perms = FlatpakPermissions {
            sockets: vec!["wayland".to_string(), "pulseaudio".to_string()],
            ..Default::default()
        };
        assert_eq!(derive_dbus_access(&perms), DbusAccess::None);
    }

    #[test]
    fn derive_network_access_true() {
        let perms = FlatpakPermissions {
            shared: vec!["network".to_string()],
            ..Default::default()
        };
        assert!(derive_network_access(&perms));

        let perms2 = FlatpakPermissions {
            sockets: vec!["ssh-auth".to_string()],
            ..Default::default()
        };
        assert!(derive_network_access(&perms2));
    }

    #[test]
    fn derive_network_access_false() {
        let perms = FlatpakPermissions {
            shared: vec!["ipc".to_string()],
            sockets: vec!["wayland".to_string()],
            ..Default::default()
        };
        assert!(!derive_network_access(&perms));
    }

    #[test]
    fn derive_device_access_true() {
        let perms = FlatpakPermissions {
            devices: vec!["all".to_string()],
            ..Default::default()
        };
        assert!(derive_device_access(&perms));

        let perms2 = FlatpakPermissions {
            devices: vec!["usb".to_string()],
            ..Default::default()
        };
        assert!(derive_device_access(&perms2));
    }

    #[test]
    fn derive_device_access_false() {
        let perms = FlatpakPermissions {
            devices: vec!["dri".to_string(), "shm".to_string()],
            ..Default::default()
        };
        assert!(!derive_device_access(&perms));
    }
}
