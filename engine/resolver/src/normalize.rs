use pkgseal_domain::PackageName;
use regex::Regex;
use std::sync::LazyLock;

static SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[-_\s.]+").unwrap());
static VENDOR_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:lib|gnu|kde|qt|gtk|libre|open|free)-").unwrap());
static VERSION_SUFFIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[-_\s]?(?:git|svn|hg|bzr|cvs|nightly|daily|stable|beta|alpha|rc\d*|\d+(?:\.\d+)+)(?:-\w+)?$").unwrap()
});
static ARCH_SUFFIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[-_\s]?(?:bin|git|debug|static|shared|dbg|dev|doc|common|data|headers|libs?)(?:-\w+)?$",
    )
    .unwrap()
});

/// Normalize a product name for comparison
/// - lowercase
/// - split on separators (-, _, space, .)
/// - remove vendor prefixes (lib, gnu, kde, qt, etc.)
/// - remove version suffixes
/// - remove architecture/build suffixes (bin, git, debug, etc.)
pub fn normalize_product_name(name: &str) -> String {
    let mut s = name.to_lowercase();

    // Remove vendor prefix conservatively: only a single leading prefix + hyphen,
    // applied once. Guard against empty remainder and known short products like
    // open-vpn where stripping `open-` would distort a real product name.
    let replaced = VENDOR_PREFIX_RE.replace(&s, "").to_string();
    if !replaced.trim().is_empty() {
        // Keep original for open-vpn (and variants) — "open" is part of product, not vendor.
        let is_open_vpn_case = s.starts_with("open-") && replaced.starts_with("vpn");
        if !is_open_vpn_case {
            s = replaced;
        }
    }

    // Remove version suffixes
    s = VERSION_SUFFIX_RE.replace_all(&s, "").to_string();
    s = ARCH_SUFFIX_RE.replace_all(&s, "").to_string();

    // Split and filter parts
    let parts: Vec<String> = SPLIT_RE
        .split(&s)
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();

    parts.join(" ")
}

/// Normalize a vendor/publisher name
pub fn normalize_vendor_name(name: &str) -> String {
    let s = name.to_lowercase();
    let s = s.replace("inc.", "").replace("inc", "");
    let s = s.replace("corp.", "").replace("corp", "");
    let s = s.replace("corporation", "");
    let s = s.replace("company", "");
    let s = s.replace("ltd.", "").replace("ltd", "");
    let s = s.replace("limited", "");
    let s = s.replace("gmbh", "");
    let s = s.replace("llc", "");
    let s = s.replace("foundation", "");
    let s = s.replace("project", "");
    let s = s.replace("team", "");
    s.split_whitespace()
        .map(|w| w.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract reverse-domain ID from various formats
/// e.g., "com.brave.Browser" -> "com.brave.browser"
/// "org.mozilla.firefox" -> "org.mozilla.firefox"
pub fn extract_reverse_domain_id(s: &str) -> Option<String> {
    let s = s.trim();
    // Must have at least two dots and look like reverse domain
    if s.chars().filter(|&c| c == '.').count() >= 2 {
        let parts: Vec<&str> = s.split('.').collect();
        // Check if first part looks like TLD (com, org, net, io, etc.)
        let tlds = [
            "com", "org", "net", "io", "app", "dev", "co", "eu", "fr", "de", "jp", "cn", "uk",
        ];
        if tlds.contains(&parts[0].to_lowercase().as_str()) {
            return Some(s.to_lowercase());
        }
    }
    None
}

/// Normalize a homepage URL for comparison
/// - remove scheme
/// - remove www.
/// - remove trailing slash
/// - lowercase
pub fn normalize_homepage(url: &str) -> String {
    let mut s = url.to_lowercase();
    s = s.replace("https://", "").replace("http://", "");
    s = s.replace("www.", "");
    s = s.trim_end_matches('/').to_string();
    s
}

/// Normalize package name for comparison (stricter than product name)
pub fn normalize_package_name(name: &PackageName) -> String {
    name.as_str().to_lowercase()
}

/// Extract product name from package name
/// e.g., "brave-bin" -> "brave"
/// "libreoffice-fresh" -> "libreoffice"
/// "code" -> "code"
pub fn extract_product_name_from_package(name: &PackageName) -> String {
    let s = name.as_str();
    // Remove common suffixes
    let suffixes = [
        "-bin", "-git", "-stable", "-beta", "-dev", "-debug", "-static", "-shared",
    ];
    let mut result = s.to_string();
    for suffix in suffixes {
        if let Some(stripped) = result.strip_suffix(suffix) {
            result = stripped.to_string();
            break;
        }
    }
    // Remove version-like suffixes
    if let Some(stripped) = result.strip_suffix("-") {
        if stripped
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            // Keep the dash if it's not version-like
        } else {
            result = stripped.to_string();
        }
    }
    normalize_product_name(&result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageName;

    #[test]
    fn test_normalize_product_name() {
        assert_eq!(normalize_product_name("Brave Browser"), "brave browser");
        assert_eq!(
            normalize_product_name("libreoffice-fresh"),
            "libreoffice fresh"
        );
        assert_eq!(
            normalize_product_name("visual-studio-code"),
            "visual studio code"
        );
        assert_eq!(normalize_product_name("Google Chrome"), "google chrome");
        assert_eq!(normalize_product_name("Mozilla Firefox"), "mozilla firefox");
    }

    #[test]
    fn test_normalize_product_name_removes_suffixes() {
        assert_eq!(normalize_product_name("brave-bin"), "brave");
        assert_eq!(normalize_product_name("vscode-git"), "vscode");
        assert_eq!(normalize_product_name("firefox-nightly"), "firefox");
        assert_eq!(normalize_product_name("code-stable"), "code");
    }

    #[test]
    fn test_normalize_product_name_removes_vendor_prefixes() {
        assert_eq!(normalize_product_name("lib-gtk-4"), "gtk 4");
        assert_eq!(normalize_product_name("gnu-emacs"), "emacs");
        assert_eq!(normalize_product_name("kde-konsole"), "konsole");
        assert_eq!(normalize_product_name("qt-creator"), "creator");
    }

    #[test]
    fn test_normalize_vendor_name() {
        assert_eq!(normalize_vendor_name("Google Inc."), "google");
        assert_eq!(normalize_vendor_name("Mozilla Foundation"), "mozilla");
        assert_eq!(
            normalize_vendor_name("The Document Foundation"),
            "the document"
        );
        assert_eq!(normalize_vendor_name("VideoLAN"), "videolan");
    }

    #[test]
    fn test_extract_reverse_domain_id() {
        assert_eq!(
            extract_reverse_domain_id("com.brave.Browser"),
            Some("com.brave.browser".to_string())
        );
        assert_eq!(
            extract_reverse_domain_id("org.mozilla.firefox"),
            Some("org.mozilla.firefox".to_string())
        );
        assert_eq!(
            extract_reverse_domain_id("io.github.user.app"),
            Some("io.github.user.app".to_string())
        );
        assert_eq!(extract_reverse_domain_id("brave-browser"), None);
        assert_eq!(extract_reverse_domain_id("firefox"), None);
    }

    #[test]
    fn test_normalize_homepage() {
        assert_eq!(normalize_homepage("https://brave.com"), "brave.com");
        assert_eq!(
            normalize_homepage("https://www.mozilla.org/en-US/"),
            "mozilla.org/en-us"
        );
        assert_eq!(normalize_homepage("http://example.com/"), "example.com");
    }

    #[test]
    fn test_extract_product_name_from_package() {
        let name = PackageName::new("brave-bin").unwrap();
        assert_eq!(extract_product_name_from_package(&name), "brave");

        let name = PackageName::new("visual-studio-code-bin").unwrap();
        assert_eq!(
            extract_product_name_from_package(&name),
            "visual studio code"
        );

        let name = PackageName::new("libreoffice-fresh").unwrap();
        assert_eq!(
            extract_product_name_from_package(&name),
            "libreoffice fresh"
        );

        let name = PackageName::new("code").unwrap();
        assert_eq!(extract_product_name_from_package(&name), "code");
    }
}
