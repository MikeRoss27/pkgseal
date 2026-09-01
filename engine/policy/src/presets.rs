use serde::{Deserialize, Serialize};

/// User-selectable policy preset. Each preset re-weights the same atomic rules — there is
/// no hard-coded `Arch > Flatpak > Aur` universal ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyPreset {
    Balanced,
    NativeFirst,
    SandboxFirst,
    MaximumReview,
}

impl PolicyPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::NativeFirst => "native-first",
            Self::SandboxFirst => "sandbox-first",
            Self::MaximumReview => "maximum-review",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Balanced => {
                "Balances provenance, publisher support, sandboxing, permissions and maintenance"
            }
            Self::NativeFirst => {
                "Prefers native packages when trust and maintenance are comparable"
            }
            Self::SandboxFirst => {
                "Prefers sandboxed applications when their permissions remain reasonable"
            }
            Self::MaximumReview => {
                "Requires stronger review before accepting community or broadly privileged packages"
            }
        }
    }

    pub fn all() -> [Self; 4] {
        [
            Self::Balanced,
            Self::NativeFirst,
            Self::SandboxFirst,
            Self::MaximumReview,
        ]
    }
}

impl std::fmt::Display for PolicyPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tunable weights for the atomic rules. Kept intentionally explicit and `i32` so
/// scoring stays deterministic and explainable. Positive values are bonuses, negative are penalties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyWeights {
    pub official_repository: i32,
    pub community_penalty: i32,
    pub publisher_verified: i32,
    pub publisher_supported: i32,
    pub signature_present: i32,
    pub checksum_present: i32,
    pub checksum_validated_bonus: i32,
    pub sandboxed_narrow_bonus: i32,
    pub sandboxed_broad_penalty: i32,
    pub narrow_permissions_bonus: i32,
    pub broad_permissions_penalty: i32,
    pub excessive_permissions_penalty: i32,
    pub host_filesystem_penalty: i32,
    pub host_dbus_penalty: i32,
    pub network_penalty: i32,
    pub findings_penalty_per_finding: i32,
    pub install_script_penalty: i32,
    pub build_changed_penalty: i32,
    pub native_integration_bonus: i32,
}

impl PolicyWeights {
    pub fn balanced() -> Self {
        Self {
            official_repository: 30,
            community_penalty: -18,
            publisher_verified: 16,
            publisher_supported: 22,
            signature_present: 14,
            checksum_present: 8,
            checksum_validated_bonus: 6,
            sandboxed_narrow_bonus: 20,
            sandboxed_broad_penalty: -24,
            narrow_permissions_bonus: 10,
            broad_permissions_penalty: -16,
            excessive_permissions_penalty: -30,
            host_filesystem_penalty: -18,
            host_dbus_penalty: -16,
            network_penalty: -2,
            findings_penalty_per_finding: -12,
            install_script_penalty: -6,
            build_changed_penalty: -8,
            native_integration_bonus: 8,
        }
    }

    pub fn native_first() -> Self {
        Self {
            official_repository: 42,
            community_penalty: -22,
            publisher_verified: 12,
            publisher_supported: 26,
            signature_present: 14,
            checksum_present: 8,
            checksum_validated_bonus: 6,
            sandboxed_narrow_bonus: 10,
            sandboxed_broad_penalty: -28,
            narrow_permissions_bonus: 6,
            broad_permissions_penalty: -14,
            excessive_permissions_penalty: -28,
            host_filesystem_penalty: -16,
            host_dbus_penalty: -14,
            network_penalty: -1,
            findings_penalty_per_finding: -12,
            install_script_penalty: -6,
            build_changed_penalty: -8,
            native_integration_bonus: 16,
        }
    }

    pub fn sandbox_first() -> Self {
        Self {
            official_repository: 18,
            community_penalty: -18,
            publisher_verified: 14,
            publisher_supported: 16,
            signature_present: 10,
            checksum_present: 6,
            checksum_validated_bonus: 4,
            sandboxed_narrow_bonus: 36,
            sandboxed_broad_penalty: -12,
            narrow_permissions_bonus: 18,
            broad_permissions_penalty: -22,
            excessive_permissions_penalty: -36,
            host_filesystem_penalty: -22,
            host_dbus_penalty: -20,
            network_penalty: -4,
            findings_penalty_per_finding: -12,
            install_script_penalty: -6,
            build_changed_penalty: -8,
            native_integration_bonus: 2,
        }
    }

    pub fn maximum_review() -> Self {
        Self {
            official_repository: 32,
            community_penalty: -36,
            publisher_verified: 10,
            publisher_supported: 14,
            signature_present: 14,
            checksum_present: 10,
            checksum_validated_bonus: 8,
            sandboxed_narrow_bonus: 16,
            sandboxed_broad_penalty: -32,
            narrow_permissions_bonus: 10,
            broad_permissions_penalty: -24,
            excessive_permissions_penalty: -40,
            host_filesystem_penalty: -28,
            host_dbus_penalty: -26,
            network_penalty: -6,
            findings_penalty_per_finding: -18,
            install_script_penalty: -12,
            build_changed_penalty: -14,
            native_integration_bonus: 6,
        }
    }
}

/// Complete policy: preset + resolved weights. `Policy` is immutable after construction and carries
/// no IO handles — the engine is pure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub preset: PolicyPreset,
    pub weights: PolicyWeights,
}

impl Policy {
    pub fn from_preset(preset: PolicyPreset) -> Self {
        let weights = match preset {
            PolicyPreset::Balanced => PolicyWeights::balanced(),
            PolicyPreset::NativeFirst => PolicyWeights::native_first(),
            PolicyPreset::SandboxFirst => PolicyWeights::sandbox_first(),
            PolicyPreset::MaximumReview => PolicyWeights::maximum_review(),
        };
        Self { preset, weights }
    }

    pub fn balanced() -> Self {
        Self::from_preset(PolicyPreset::Balanced)
    }

    pub fn native_first() -> Self {
        Self::from_preset(PolicyPreset::NativeFirst)
    }

    pub fn sandbox_first() -> Self {
        Self::from_preset(PolicyPreset::SandboxFirst)
    }

    pub fn maximum_review() -> Self {
        Self::from_preset(PolicyPreset::MaximumReview)
    }

    pub fn with_weights(mut self, weights: PolicyWeights) -> Self {
        self.weights = weights;
        self
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self::balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_distinct_weights() {
        let balanced = PolicyWeights::balanced();
        let native = PolicyWeights::native_first();
        let sandbox = PolicyWeights::sandbox_first();
        let review = PolicyWeights::maximum_review();

        // NativeFirst boosts official_repository and native_integration over Balanced.
        assert!(native.official_repository > balanced.official_repository);
        assert!(native.native_integration_bonus > balanced.native_integration_bonus);

        // SandboxFirst boosts sandboxed_narrow_bonus over Balanced/NativeFirst.
        assert!(sandbox.sandboxed_narrow_bonus > balanced.sandboxed_narrow_bonus);
        assert!(sandbox.sandboxed_narrow_bonus > native.sandboxed_narrow_bonus);

        // MaximumReview penalizes community more strongly than Balanced.
        assert!(review.community_penalty < balanced.community_penalty);
        assert!(review.findings_penalty_per_finding < balanced.findings_penalty_per_finding);
    }

    #[test]
    fn policy_from_preset_is_deterministic() {
        let a = Policy::from_preset(PolicyPreset::Balanced);
        let b = Policy::from_preset(PolicyPreset::Balanced);
        assert_eq!(a, b);
        assert_eq!(a.preset, PolicyPreset::Balanced);
    }

    #[test]
    fn all_presets_constructible() {
        for preset in PolicyPreset::all() {
            let p = Policy::from_preset(preset);
            assert_eq!(p.preset, preset);
        }
    }

    #[test]
    fn preset_display_roundtrip() {
        for preset in PolicyPreset::all() {
            let s = preset.as_str();
            let json = serde_json::to_string(&preset).unwrap();
            assert!(json.contains(s));
        }
    }
}
