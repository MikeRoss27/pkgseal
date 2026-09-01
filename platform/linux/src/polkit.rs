use crate::error::PlatformError;
use crate::privilege::PrivilegedRequest;
use serde::{Deserialize, Serialize};

/// Polkit action identifiers used by PkgSeal.
///
/// These must match the `.policy` file installed alongside the privileged
/// helper (`/usr/share/polkit-1/actions/org.pkgseal.policy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolkitAction {
    InstallArch,
    RemoveArch,
    InstallFlatpak,
    RemoveFlatpak,
    UpdateFlatpak,
    EnableService,
    DisableService,
}

impl PolkitAction {
    #[must_use]
    pub fn action_id(&self) -> &'static str {
        match self {
            Self::InstallArch => "org.pkgseal.install-arch",
            Self::RemoveArch => "org.pkgseal.remove-arch",
            Self::InstallFlatpak => "org.pkgseal.install-flatpak",
            Self::RemoveFlatpak => "org.pkgseal.remove-flatpak",
            Self::UpdateFlatpak => "org.pkgseal.update-flatpak",
            Self::EnableService => "org.pkgseal.enable-service",
            Self::DisableService => "org.pkgseal.disable-service",
        }
    }

    #[must_use]
    pub fn from_privileged_request(req: &PrivilegedRequest) -> Self {
        match req {
            PrivilegedRequest::InstallArch { .. } => Self::InstallArch,
            PrivilegedRequest::RemoveArch { .. } => Self::RemoveArch,
            PrivilegedRequest::InstallFlatpak { .. } => Self::InstallFlatpak,
            PrivilegedRequest::RemoveFlatpak { .. } => Self::RemoveFlatpak,
            PrivilegedRequest::UpdateFlatpak { .. } => Self::UpdateFlatpak,
            PrivilegedRequest::EnableService { .. } => Self::EnableService,
            PrivilegedRequest::DisableService { .. } => Self::DisableService,
        }
    }
}

impl std::fmt::Display for PolkitAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.action_id())
    }
}

/// Subject requesting authorization (typically the calling process).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolkitSubject {
    /// PID of the client process.
    pub pid: u32,
    /// UID of the client user.
    pub uid: u32,
    /// Optional DBus bus name (`:1.42`). Validated to avoid injection.
    pub bus_name: Option<String>,
}

impl PolkitSubject {
    pub fn new(pid: u32, uid: u32) -> Self {
        Self {
            pid,
            uid,
            bus_name: None,
        }
    }

    pub fn with_bus_name(mut self, bus_name: impl Into<String>) -> Result<Self, PlatformError> {
        let name = bus_name.into();
        if name.contains(';')
            || name.contains('|')
            || name.contains('$')
            || name.contains('`')
            || name.contains('\n')
            || name.contains('\0')
        {
            return Err(PlatformError::polkit(format!("invalid bus name {name:?}")));
        }
        self.bus_name = Some(name);
        Ok(self)
    }

    /// Current process as subject (pid/uid from `std::process` and `nix`-free fallback).
    #[must_use]
    pub fn current() -> Self {
        Self {
            pid: std::process::id(),
            uid: current_uid(),
            bus_name: None,
        }
    }
}

fn current_uid() -> u32 {
    // Minimal without `libc` crate: rely on env or default 1000 for tests.
    // The privileged helper will re-resolve via real getuid().
    if let Ok(val) = std::env::var("UID")
        && let Ok(n) = val.parse::<u32>()
    {
        return n;
    }
    1000
}

/// Result of a Polkit authorization check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationResult {
    /// Caller is authorized without challenge.
    Authorized,
    /// Caller is not authorized and no challenge is possible.
    NotAuthorized,
    /// Caller must authenticate (password / dialog).
    ChallengeRequired,
    /// Polkit service is not available — treat as advisory, not fatal, in
    /// read-only paths.
    ServiceUnavailable,
}

impl AuthorizationResult {
    #[must_use]
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized)
    }
}

/// Abstraction over the system Polkit agent.
///
/// Real implementation would talk to `org.freedesktop.PolicyKit1` over DBus
/// (e.g. via `zbus`). This crate intentionally avoids a heavy DBus dependency
/// in the MVP — the trait allows swapping in a real client without changing
/// callers.
#[async_trait::async_trait]
pub trait PolkitClient: Send + Sync + std::fmt::Debug {
    async fn check_authorization(
        &self,
        subject: &PolkitSubject,
        action: PolkitAction,
        details: &PolkitDetails,
    ) -> Result<AuthorizationResult, PlatformError>;

    /// Convenience: check whether `subject` is authorized for `request`.
    async fn is_authorized_for(
        &self,
        subject: &PolkitSubject,
        request: &PrivilegedRequest,
    ) -> Result<bool, PlatformError> {
        let action = PolkitAction::from_privileged_request(request);
        let details = PolkitDetails::from_request(request);
        let result = self.check_authorization(subject, action, &details).await?;
        Ok(result.is_authorized())
    }
}

/// Extra details sent to Polkit for audit / policy rules.
///
/// Never contains raw shell commands; only typed, validated identifiers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolkitDetails {
    pub package_count: Option<usize>,
    pub app_id: Option<String>,
    pub unit: Option<String>,
}

impl PolkitDetails {
    #[must_use]
    pub fn from_request(req: &PrivilegedRequest) -> Self {
        match req {
            PrivilegedRequest::InstallArch { packages }
            | PrivilegedRequest::RemoveArch { packages } => Self {
                package_count: Some(packages.len()),
                ..Default::default()
            },
            PrivilegedRequest::InstallFlatpak { app_id, .. }
            | PrivilegedRequest::RemoveFlatpak { app_id } => Self {
                app_id: Some(app_id.as_str().to_owned()),
                ..Default::default()
            },
            PrivilegedRequest::UpdateFlatpak { app_ids } => Self {
                package_count: Some(app_ids.len()),
                ..Default::default()
            },
            PrivilegedRequest::EnableService { unit }
            | PrivilegedRequest::DisableService { unit } => Self {
                unit: Some(unit.as_str().to_owned()),
                ..Default::default()
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Stub
// ---------------------------------------------------------------------------

/// Stub Polkit client that always returns a configured result.
///
/// Useful for tests and for read-only development without a running Polkit
/// daemon. Never grants unintended authorizations in production — the
/// privileged helper must use a real client.
#[derive(Debug, Clone)]
pub struct StubPolkitClient {
    result: AuthorizationResult,
}

impl StubPolkitClient {
    #[must_use]
    pub fn new(result: AuthorizationResult) -> Self {
        Self { result }
    }

    #[must_use]
    pub fn authorized() -> Self {
        Self::new(AuthorizationResult::Authorized)
    }

    #[must_use]
    pub fn not_authorized() -> Self {
        Self::new(AuthorizationResult::NotAuthorized)
    }

    #[must_use]
    pub fn challenge_required() -> Self {
        Self::new(AuthorizationResult::ChallengeRequired)
    }

    #[must_use]
    pub fn unavailable() -> Self {
        Self::new(AuthorizationResult::ServiceUnavailable)
    }
}

#[async_trait::async_trait]
impl PolkitClient for StubPolkitClient {
    async fn check_authorization(
        &self,
        _subject: &PolkitSubject,
        _action: PolkitAction,
        _details: &PolkitDetails,
    ) -> Result<AuthorizationResult, PlatformError> {
        Ok(self.result)
    }
}

/// Check authorization synchronously (blocking) using the stub.
/// Real daemon integration will be async over DBus.
pub fn check_authorization_blocking(
    client: &dyn PolkitClient,
    subject: &PolkitSubject,
    action: PolkitAction,
    details: &PolkitDetails,
) -> Result<AuthorizationResult, PlatformError> {
    // Build a minimal runtime for blocking callers.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| PlatformError::Internal(format!("runtime error: {e}")))?;
    rt.block_on(client.check_authorization(subject, action, details))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::privilege::FlatpakAppId;
    use pkgseal_domain::PackageName;

    #[test]
    fn polkit_action_ids_are_org_pkgseal() {
        for action in [
            PolkitAction::InstallArch,
            PolkitAction::RemoveArch,
            PolkitAction::InstallFlatpak,
            PolkitAction::RemoveFlatpak,
            PolkitAction::UpdateFlatpak,
            PolkitAction::EnableService,
            PolkitAction::DisableService,
        ] {
            assert!(
                action.action_id().starts_with("org.pkgseal."),
                "{}",
                action.action_id()
            );
        }
    }

    #[test]
    fn subject_current_has_pid() {
        let s = PolkitSubject::current();
        assert!(s.pid > 0);
    }

    #[test]
    fn subject_rejects_injection() {
        let s = PolkitSubject::new(1, 1000);
        assert!(s.clone().with_bus_name(":1.42").is_ok());
        assert!(s.clone().with_bus_name("bad;rm").is_err());
        assert!(s.with_bus_name("bad\nname").is_err());
    }

    #[tokio::test]
    async fn stub_client_returns_configured() {
        let client = StubPolkitClient::authorized();
        let subject = PolkitSubject::new(1, 1000);
        let result = client
            .check_authorization(
                &subject,
                PolkitAction::InstallArch,
                &PolkitDetails::default(),
            )
            .await
            .unwrap();
        assert_eq!(result, AuthorizationResult::Authorized);

        let client2 = StubPolkitClient::not_authorized();
        let result2 = client2
            .check_authorization(
                &subject,
                PolkitAction::InstallArch,
                &PolkitDetails::default(),
            )
            .await
            .unwrap();
        assert!(!result2.is_authorized());
    }

    #[tokio::test]
    async fn is_authorized_for_maps_request() {
        let client = StubPolkitClient::authorized();
        let subject = PolkitSubject::current();
        let req = PrivilegedRequest::InstallArch {
            packages: vec![PackageName::new("brave-bin").unwrap()],
        };
        assert!(client.is_authorized_for(&subject, &req).await.unwrap());
    }

    #[test]
    fn polkit_details_from_request() {
        let req = PrivilegedRequest::InstallArch {
            packages: vec![
                PackageName::new("a").unwrap(),
                PackageName::new("b").unwrap(),
            ],
        };
        let details = PolkitDetails::from_request(&req);
        assert_eq!(details.package_count, Some(2));

        let req2 = PrivilegedRequest::InstallFlatpak {
            app_id: FlatpakAppId::new("com.example.App").unwrap(),
            remote: None,
        };
        let details2 = PolkitDetails::from_request(&req2);
        assert_eq!(details2.app_id.as_deref(), Some("com.example.App"));
    }

    #[test]
    fn check_blocking() {
        let client = StubPolkitClient::challenge_required();
        let subject = PolkitSubject::new(1, 1000);
        let result = check_authorization_blocking(
            &client,
            &subject,
            PolkitAction::InstallArch,
            &PolkitDetails::default(),
        )
        .unwrap();
        assert_eq!(result, AuthorizationResult::ChallengeRequired);
    }
}
