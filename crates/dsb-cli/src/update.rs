//! `deepseek-build update` / `dsb update` — update the install in place.
//!
//! The running binary's install channel is detected first, the product feeds
//! are asked for the latest published version, and the update runs only when
//! the channel supports it. Feed coordinates and the "never downgrade" policy
//! mirror `npm/lib/version-align.js` and the vendored updater under
//! `third_party/grok-build/crates/codegen/xai-grok-update` (reference only —
//! this module is the product's own implementation).
//!
//! Exit codes: `0` when the install is up to date or was updated, `1` when
//! nothing was installed (unrecognized channel, source install, refused
//! downgrade, unreadable feed).

use std::cmp::Ordering;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// npm package that ships this product.
pub const NPM_PACKAGE: &str = "@innocarpe/deepseek-build";
const NPM_LATEST_URL: &str = "https://registry.npmjs.org/@innocarpe/deepseek-build/latest";
/// Release feed of `innocarpe/deepseek-build`, used when the registry is down.
const GH_LATEST_URL: &str = "https://api.github.com/repos/innocarpe/deepseek-build/releases/latest";
/// Documented escape hatch for replacing a newer install with an older one
/// (`npm/lib/version-align.js`).
pub const ALLOW_DOWNGRADE_ENV: &str = "DEEPSEEK_BUILD_ALLOW_DOWNGRADE";
/// Installer classification, set to `npm` by the npm shim
/// (`npm/lib/run-native.js`).
const ENV_INSTALLER: &str = "GROK_INSTALLER";
const ENV_MANAGED_BY_NPM: &str = "GROK_MANAGED_BY_NPM";
const ENV_NPM_USER_AGENT: &str = "npm_config_user_agent";
/// Explicit native-binary override honoured by the npm shim.
const ENV_BIN: &str = "DEEPSEEK_BUILD_BIN";
/// Install prefix of `scripts/install.sh` (`--prefix` writes `<prefix>/bin`).
const ENV_INSTALL_PREFIX: &str = "DEEPSEEK_BUILD_INSTALL_PREFIX";
const CURL_TIMEOUT: &str = "20";

/// Which install layout the running binary came from.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Channel {
    /// Global npm package: `update` runs `npm install -g …@latest`.
    Npm,
    /// A checkout install (`./scripts/install.sh`, `cargo install`): `update`
    /// prints the pull + reinstall commands. `checkout` names the product
    /// checkout the binary sits in, when one could be identified.
    Source { checkout: Option<PathBuf> },
    /// No clue matched: refuse rather than guess.
    Unknown,
}

impl Channel {
    fn label(&self) -> &'static str {
        match self {
            Channel::Npm => "npm global install",
            Channel::Source { .. } => "source install",
            Channel::Unknown => "unrecognized install",
        }
    }
}

/// Where the latest version came from, for the printed line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feed {
    NpmRegistry,
    GitHubRelease,
    NpmCli,
}

impl Feed {
    fn label(self) -> &'static str {
        match self {
            Feed::NpmRegistry => "npm registry",
            Feed::GitHubRelease => "GitHub Releases",
            Feed::NpmCli => "npm CLI",
        }
    }
}

/// What `update` should do about the two versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Plan {
    UpToDate,
    Update,
    Downgrade,
}

/// Env facts and paths that decide the install channel.
#[derive(Debug, Default, Clone)]
struct InstallFacts {
    /// `current_exe()` as reported, plus its resolved path.
    exe: Option<PathBuf>,
    exe_real: Option<PathBuf>,
    /// `DEEPSEEK_BUILD_BIN`, the npm shim's explicit binary override.
    bin_override: Option<PathBuf>,
    installer: Option<String>,
    managed_by_npm: bool,
    npm_user_agent: bool,
    /// `<DEEPSEEK_BUILD_INSTALL_PREFIX>/bin` (`scripts/install.sh --prefix`).
    prefix_bin: Option<PathBuf>,
    /// `<product home>/bin` — written by the npm prebuilt install *and* by
    /// `scripts/install.sh`, so it decides nothing on its own.
    home_bin: Option<PathBuf>,
    /// `$CARGO_HOME/bin`, else `~/.cargo/bin` (`cargo install --path`).
    cargo_bin: Option<PathBuf>,
}

impl InstallFacts {
    fn from_env() -> Self {
        let exe = env::current_exe().ok();
        let exe_real = exe.as_deref().and_then(|p| p.canonicalize().ok());
        let home = dsb_config::BuildHome::resolve();
        let home_dir = home.path().to_path_buf();
        let prefix_bin = env::var(ENV_INSTALL_PREFIX)
            .ok()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .map(|p| PathBuf::from(p).join("bin"));
        let cargo_bin = env::var("CARGO_HOME")
            .ok()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .map(|p| PathBuf::from(p).join("bin"))
            .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".cargo").join("bin")));
        Self {
            installer: env::var(ENV_INSTALLER).ok(),
            managed_by_npm: env::var_os(ENV_MANAGED_BY_NPM).is_some(),
            npm_user_agent: env::var_os(ENV_NPM_USER_AGENT).is_some(),
            bin_override: env::var(ENV_BIN)
                .ok()
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .map(PathBuf::from),
            prefix_bin,
            home_bin: Some(home_dir.join("bin")),
            cargo_bin,
            exe,
            exe_real,
        }
    }

    /// Paths that stand for "the binary being updated".
    fn paths(&self) -> impl Iterator<Item = &Path> {
        [
            self.exe.as_deref(),
            self.exe_real.as_deref(),
            self.bin_override.as_deref(),
        ]
        .into_iter()
        .flatten()
    }

    /// The first candidate path that sits under `dir`.
    fn under(&self, dir: &Path) -> bool {
        self.paths().any(|p| p.starts_with(dir))
    }
}

/// Classify the running install. `npm_global` is the result of
/// [`npm_global_package_present`].
fn classify(facts: &InstallFacts, npm_global: bool) -> Channel {
    let source = || Channel::Source {
        checkout: checkout_root(facts),
    };
    // The npm shim exports GROK_INSTALLER=npm; the vendored updater's other
    // spellings mean a release/managed install, which the registry cannot
    // replace.
    if let Some(value) = facts.installer.as_deref() {
        match value.to_ascii_lowercase().as_str() {
            "npm" => return Channel::Npm,
            "internal" | "gh-release" | "gh" => return source(),
            // An unrecognized value (e.g. `brew`) states nothing; fall through
            // to the remaining clues.
            _ => {}
        }
    }
    if facts.managed_by_npm || facts.npm_user_agent {
        return Channel::Npm;
    }
    if facts.paths().any(|p| has_component(p, "node_modules")) {
        return Channel::Npm;
    }
    // `cargo install` never writes into the npm package or the product home.
    if let Some(dir) = facts.cargo_bin.as_deref()
        && facts.under(dir)
    {
        return source();
    }
    for dir in [facts.home_bin.as_deref(), facts.prefix_bin.as_deref()]
        .into_iter()
        .flatten()
    {
        if facts.under(dir) {
            // The npm prebuilt postinstall unpacks into `<home>/bin` too, so
            // the path alone cannot separate the two writers. Ask npm.
            return if npm_global { Channel::Npm } else { source() };
        }
    }
    match checkout_root(facts) {
        Some(_) => source(),
        None => Channel::Unknown,
    }
}

/// True when `path` has `name` as a whole component.
fn has_component(path: &Path, name: &str) -> bool {
    path.components().any(|c| c.as_os_str() == name)
}

/// The product checkout a candidate path sits in, walking up to the filesystem
/// root. A directory counts only with the source-checkout trio plus the
/// workspace crate, so an unrelated parent repository is not mistaken for one.
fn checkout_root(facts: &InstallFacts) -> Option<PathBuf> {
    facts.paths().find_map(|p| {
        let mut dir = p.parent()?;
        loop {
            if is_source_checkout(dir) {
                return Some(dir.to_path_buf());
            }
            dir = dir.parent()?;
        }
    })
}

fn is_source_checkout(dir: &Path) -> bool {
    dir.join(".git").exists()
        && dir.join("Cargo.toml").is_file()
        && dir.join("scripts").join("install.sh").is_file()
        && dir.join("crates").join("dsb-cli").is_dir()
}

/// True when this package is installed globally for npm (`npm root -g`).
/// A missing or broken npm reads as "not installed".
fn npm_global_package_present() -> bool {
    let Ok(root) = run_capture("npm", &["root", "-g"]) else {
        return false;
    };
    if root.is_empty() {
        return false;
    }
    Path::new(&root)
        .join(NPM_PACKAGE)
        .join("package.json")
        .is_file()
}

/// `npm install -g` for this product, with the npm 12 script opt-in
/// (`npm/lib/run-native.js`). The spec is repeated because npm 12 ignores a
/// bare `--allow-scripts=<name>` without a package argument.
fn npm_install_command() -> String {
    format!("npm install -g --allow-scripts={NPM_PACKAGE} {NPM_PACKAGE}@latest")
}

fn npm_install_argv() -> Vec<String> {
    vec![
        "install".to_string(),
        "-g".to_string(),
        format!("--allow-scripts={NPM_PACKAGE}"),
        format!("{NPM_PACKAGE}@latest"),
    ]
}

/// SemVer with the pre-release list kept separate, mirroring `parseSemver` in
/// `npm/lib/version-align.js`. Build metadata (`+…`) is not accepted, so a
/// version carrying it is not mistaken for a release.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Semver {
    core: [u64; 3],
    pre: Vec<String>,
}

impl Ord for Semver {
    fn cmp(&self, other: &Self) -> Ordering {
        self.core
            .cmp(&other.core)
            .then_with(|| compare_pre(&self.pre, &other.pre))
    }
}

impl PartialOrd for Semver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_semver(input: &str) -> Option<Semver> {
    let s = input.trim();
    let (core, pre) = match s.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (s, None),
    };
    let mut nums = [0u64; 3];
    let mut parts = core.split('.');
    for slot in &mut nums {
        let part = parts.next()?;
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = part.parse().ok()?;
    }
    if parts.next().is_some() {
        return None;
    }
    let pre = match pre {
        None => Vec::new(),
        Some(pre) => {
            if pre.is_empty() || !pre.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.') {
                return None;
            }
            pre.split('.').map(str::to_string).collect()
        }
    };
    Some(Semver { core: nums, pre })
}

/// A missing pre-release list outranks a present one (release > pre-release);
/// numeric identifiers rank below alphanumeric ones.
fn compare_pre(a: &[String], b: &[String]) -> Ordering {
    if a.is_empty() && b.is_empty() {
        return Ordering::Equal;
    }
    if a.is_empty() {
        return Ordering::Greater;
    }
    if b.is_empty() {
        return Ordering::Less;
    }
    for i in 0..a.len().max(b.len()) {
        let (Some(x), Some(y)) = (a.get(i), b.get(i)) else {
            return if a.get(i).is_none() {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        };
        let ord = match (is_numeric_ident(x), is_numeric_ident(y)) {
            (true, true) => cmp_numeric_ident(x, y),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => x.cmp(y),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

fn is_numeric_ident(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Numeric identifiers compare as numbers, so leading zeros do not count.
fn cmp_numeric_ident(a: &str, b: &str) -> Ordering {
    let a = a.trim_start_matches('0');
    let b = b.trim_start_matches('0');
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// `Some(Ordering)` when both sides are SemVer, mirroring `compareSemver`.
#[cfg(test)]
fn compare_semver(a: &str, b: &str) -> Option<Ordering> {
    Some(parse_semver(a)?.cmp(&parse_semver(b)?))
}

fn allow_downgrade() -> bool {
    env::var(ALLOW_DOWNGRADE_ENV).is_ok_and(|v| v == "1")
}

/// Decide between "nothing to do", "update", and "refuse" for two versions.
fn plan_update(current: &str, latest: &str, allow_downgrade: bool) -> Result<Plan> {
    let installed = parse_semver(current).with_context(|| {
        format!("installed version `{current}` is not MAJOR.MINOR.PATCH SemVer")
    })?;
    let published = parse_semver(latest)
        .with_context(|| format!("published version `{latest}` is not MAJOR.MINOR.PATCH SemVer"))?;
    match published.cmp(&installed) {
        Ordering::Greater => Ok(Plan::Update),
        Ordering::Equal => Ok(Plan::UpToDate),
        Ordering::Less if allow_downgrade => Ok(Plan::Update),
        Ordering::Less => Ok(Plan::Downgrade),
    }
}

/// Ask the product feeds in turn for the latest published version.
fn latest_version() -> Result<(String, Feed)> {
    let mut problems: Vec<String> = Vec::new();
    let mut attempt = |feed: Feed, result: Result<String, String>| match result {
        Ok(text) => match feed {
            Feed::NpmRegistry => match parse_npm_latest(&text) {
                Some(v) => Some((v, feed)),
                None => {
                    problems.push(format!("{}: no `version` field", feed.label()));
                    None
                }
            },
            Feed::GitHubRelease => match parse_gh_tag(&text) {
                Some(v) => Some((v, feed)),
                None => {
                    problems.push(format!("{}: no `tag_name` field", feed.label()));
                    None
                }
            },
            Feed::NpmCli => {
                let v = text.trim().to_string();
                if v.is_empty() {
                    problems.push(format!("{}: empty output", feed.label()));
                    None
                } else {
                    Some((v, feed))
                }
            }
        },
        Err(e) => {
            problems.push(format!("{}: {e}", feed.label()));
            None
        }
    };
    if let Some(found) = attempt(
        Feed::NpmRegistry,
        run_capture(
            "curl",
            &["-fsSL", "--max-time", CURL_TIMEOUT, NPM_LATEST_URL],
        ),
    ) {
        return Ok(found);
    }
    if let Some(found) = attempt(
        Feed::GitHubRelease,
        run_capture(
            "curl",
            &["-fsSL", "--max-time", CURL_TIMEOUT, GH_LATEST_URL],
        ),
    ) {
        return Ok(found);
    }
    if let Some(found) = attempt(
        Feed::NpmCli,
        run_capture("npm", &["view", NPM_PACKAGE, "version"]),
    ) {
        return Ok(found);
    }
    bail!(
        "could not read the latest published version:\n  {}",
        problems.join("\n  ")
    )
}

/// Trimmed stdout of a successful command.
fn run_capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{program} {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_npm_latest(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    json_string(&value, "version")
}

fn parse_gh_tag(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let tag = json_string(&value, "tag_name")?;
    Some(tag.trim_start_matches(['v', 'V']).to_string())
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    let s = value.get(key)?.as_str()?.trim();
    if s.is_empty() {
        return None;
    }
    Some(s.to_string())
}

/// `deepseek-build update` / `dsb update`.
pub fn run_update() -> Result<()> {
    let inv = crate::invocation_name();
    let current = env!("CARGO_PKG_VERSION");
    let facts = InstallFacts::from_env();
    let channel = classify(&facts, npm_global_package_present());

    if channel == Channel::Unknown {
        bail!("{}", unknown_channel_message(inv, current, &facts));
    }

    println!("installed {current} ({})", channel.label());
    let (latest, feed) = latest_version()
        .with_context(|| format!("{inv} update: could not read the latest published version"))?;
    println!("latest    {latest} ({})", feed.label());

    match plan_update(current, &latest, allow_downgrade())? {
        Plan::UpToDate => {
            println!("\nAlready up to date ({current}).");
            Ok(())
        }
        Plan::Downgrade => bail!("{}", downgrade_refusal(inv, current, &latest)),
        Plan::Update => match channel {
            Channel::Npm => {
                println!("\nUpdating {NPM_PACKAGE} {current} → {latest} …");
                run_npm_install()?;
                println!("\nUpdated to {latest}. Re-run `{inv}` to use it.");
                Ok(())
            }
            Channel::Source { checkout } => bail!(
                "{}",
                source_guidance(inv, current, &latest, checkout.as_deref())
            ),
            Channel::Unknown => unreachable!("unknown channel returns before the feed lookup"),
        },
    }
}

/// Run the npm update with npm's own output on the terminal.
fn run_npm_install() -> Result<()> {
    let command = npm_install_command();
    let status = Command::new("npm")
        .args(npm_install_argv())
        .status()
        .with_context(|| format!("could not run `{command}` (is npm on PATH?)"))?;
    if !status.success() {
        bail!(
            "`{command}` failed ({status}).\n\
             The install was left as it was — re-run the command above to see the full npm output."
        );
    }
    Ok(())
}

fn unknown_channel_message(inv: &str, current: &str, facts: &InstallFacts) -> String {
    let exe = facts
        .exe
        .as_deref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown path".to_string());
    format!(
        "{inv} update: install channel not recognized for {exe}\n\
         current version: {current}\n\
         Update it the way it was installed:\n\
         \x20 npm global:  {}\n\
         \x20 source:      cd <checkout> && git pull --ff-only && ./scripts/install.sh\n\
         State the channel with GROK_INSTALLER=npm (or gh-release), or point at the\n\
         binary with {ENV_BIN}=<path>.",
        npm_install_command()
    )
}

fn downgrade_refusal(inv: &str, current: &str, latest: &str) -> String {
    format!(
        "{inv} update: refusing to replace {current} with the older published {latest}.\n\
         The install was left unchanged.\n\
         Install it anyway: {ALLOW_DOWNGRADE_ENV}=1 {inv} update"
    )
}

fn source_guidance(inv: &str, current: &str, latest: &str, checkout: Option<&Path>) -> String {
    let mut out = format!(
        "{inv} update: source install ({current}) — the npm registry cannot replace it \
         (latest published is {latest}).\n"
    );
    match checkout {
        Some(root) => {
            let root = root.display();
            out.push_str(&format!(
                "Update from the checkout:\n  git -C {root} pull --ff-only\n  cd {root} && ./scripts/install.sh\n"
            ));
        }
        None => out
            .push_str("Update from your checkout:\n  git pull --ff-only\n  ./scripts/install.sh\n"),
    }
    out.push_str(&format!(
        "The npm channel updates in place: {}",
        npm_install_command()
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> InstallFacts {
        InstallFacts::default()
    }

    fn facts_at(exe: &str) -> InstallFacts {
        InstallFacts {
            exe: Some(PathBuf::from(exe)),
            ..InstallFacts::default()
        }
    }

    #[test]
    fn semver_ordering() {
        assert_eq!(compare_semver("6.9.0", "6.9.1"), Some(Ordering::Less));
        assert_eq!(compare_semver("6.10.0", "6.9.9"), Some(Ordering::Greater));
        assert_eq!(compare_semver("6.9.0", "6.9.0"), Some(Ordering::Equal));
        assert_eq!(compare_semver(" 6.9.0 ", "6.9.0"), Some(Ordering::Equal));
        // Leading zeros read as the same number.
        assert_eq!(compare_semver("06.9.0", "6.9.0"), Some(Ordering::Equal));
        // A pre-release ranks below its release.
        assert_eq!(compare_semver("6.9.0-rc.1", "6.9.0"), Some(Ordering::Less));
        assert_eq!(
            compare_semver("6.9.0", "6.9.0-rc.1"),
            Some(Ordering::Greater)
        );
        // Numeric pre-release identifiers compare as numbers, not strings.
        assert_eq!(
            compare_semver("6.9.0-rc.2", "6.9.0-rc.10"),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_semver("6.9.0-rc.02", "6.9.0-rc.2"),
            Some(Ordering::Equal)
        );
        // Numeric identifiers rank below alphanumeric ones.
        assert_eq!(
            compare_semver("6.9.0-1", "6.9.0-alpha"),
            Some(Ordering::Less)
        );
        // A longer pre-release list wins when the shared part is equal.
        assert_eq!(
            compare_semver("6.9.0-alpha.1", "6.9.0-alpha"),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn semver_rejects_non_semver() {
        for bad in [
            "6.9",
            "v6.9.0",
            "6.9.0+meta",
            "6.9.0-",
            "6.9.0-rc_1",
            "6.9.0.1",
            "",
            "latest",
        ] {
            assert_eq!(compare_semver(bad, "6.9.0"), None, "{bad}");
        }
    }

    #[test]
    fn plan_prefers_newer_and_refuses_downgrade() {
        assert_eq!(
            plan_update("6.9.0", "6.9.0", false).unwrap(),
            Plan::UpToDate
        );
        assert_eq!(plan_update("6.9.0", "6.9.1", false).unwrap(), Plan::Update);
        assert_eq!(plan_update("6.9.0", "7.0.0", false).unwrap(), Plan::Update);
        assert_eq!(
            plan_update("6.9.0", "6.8.0", false).unwrap(),
            Plan::Downgrade
        );
        // A pre-release of the installed core ranks below that release, so a
        // pre-release published as `latest` does not displace it.
        assert_eq!(
            plan_update("6.9.0", "6.9.0-rc.1", false).unwrap(),
            Plan::Downgrade
        );
        // A pre-release of a higher core is still newer than the install.
        assert_eq!(
            plan_update("6.9.0", "6.10.0-rc.1", false).unwrap(),
            Plan::Update
        );
        // The documented escape hatch.
        assert_eq!(plan_update("6.9.0", "6.8.0", true).unwrap(), Plan::Update);
        assert!(plan_update("not-a-version", "6.9.0", false).is_err());
        assert!(plan_update("6.9.0", "not-a-version", false).is_err());
    }

    #[test]
    fn classify_reads_the_installer_env() {
        let npm = InstallFacts {
            installer: Some("npm".into()),
            ..facts()
        };
        assert_eq!(classify(&npm, false), Channel::Npm);
        let upper = InstallFacts {
            installer: Some("NPM".into()),
            ..facts()
        };
        assert_eq!(classify(&upper, false), Channel::Npm);
        for value in ["internal", "gh-release", "gh", "Gh-Release"] {
            let release = InstallFacts {
                installer: Some(value.into()),
                ..facts()
            };
            assert_eq!(
                classify(&release, false),
                Channel::Source { checkout: None },
                "{value}"
            );
        }
        // An unknown value states nothing; the remaining clues decide.
        let unknown_value = InstallFacts {
            installer: Some("brew".into()),
            managed_by_npm: true,
            ..facts()
        };
        assert_eq!(classify(&unknown_value, false), Channel::Npm);
    }

    #[test]
    fn classify_uses_npm_env_and_paths() {
        let managed = InstallFacts {
            managed_by_npm: true,
            ..facts()
        };
        assert_eq!(classify(&managed, false), Channel::Npm);
        let user_agent = InstallFacts {
            npm_user_agent: true,
            ..facts()
        };
        assert_eq!(classify(&user_agent, false), Channel::Npm);
        // The npm package's own bin, and the shim's override, both name npm.
        let under_modules =
            facts_at("/usr/local/lib/node_modules/@innocarpe/deepseek-build/npm/native-bin/dsb");
        assert_eq!(classify(&under_modules, false), Channel::Npm);
        let overridden = InstallFacts {
            exe: Some(PathBuf::from("/usr/local/bin/dsb")),
            bin_override: Some(PathBuf::from(
                "/usr/local/lib/node_modules/@innocarpe/deepseek-build/npm/native-bin/dsb",
            )),
            ..facts()
        };
        assert_eq!(classify(&overridden, false), Channel::Npm);
    }

    #[test]
    fn classify_separates_home_bin_and_cargo_bin() {
        let cargo = InstallFacts {
            exe: Some(PathBuf::from("/Users/dev/.cargo/bin/dsb")),
            cargo_bin: Some(PathBuf::from("/Users/dev/.cargo/bin")),
            home_bin: Some(PathBuf::from("/Users/dev/.deepseek-build/bin")),
            ..facts()
        };
        assert_eq!(classify(&cargo, false), Channel::Source { checkout: None });
        assert_eq!(classify(&cargo, true), Channel::Source { checkout: None });
        // `<home>/bin` is written by the npm prebuilt install and by
        // `scripts/install.sh`, so the npm package decides.
        let home = InstallFacts {
            exe: Some(PathBuf::from("/Users/dev/.deepseek-build/bin/dsb")),
            home_bin: Some(PathBuf::from("/Users/dev/.deepseek-build/bin")),
            ..facts()
        };
        assert_eq!(classify(&home, true), Channel::Npm);
        assert_eq!(classify(&home, false), Channel::Source { checkout: None });
        // A custom `--prefix` install keeps the same tie-break.
        let prefix = InstallFacts {
            exe: Some(PathBuf::from("/opt/dsb/bin/dsb")),
            prefix_bin: Some(PathBuf::from("/opt/dsb/bin")),
            ..facts()
        };
        assert_eq!(classify(&prefix, false), Channel::Source { checkout: None });
        assert_eq!(classify(&prefix, true), Channel::Npm);
    }

    #[test]
    fn classify_refuses_to_guess() {
        let elsewhere = facts_at("/opt/tools/dsb");
        assert_eq!(classify(&elsewhere, false), Channel::Unknown);
        assert_eq!(classify(&elsewhere, true), Channel::Unknown);
        assert_eq!(classify(&facts(), false), Channel::Unknown);
    }

    #[test]
    fn classify_names_the_checkout_of_a_dev_binary() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("deepseek-build");
        for dir in [".git", "crates/dsb-cli", "scripts", "target/debug"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        std::fs::write(root.join("scripts").join("install.sh"), "#!/bin/sh\n").unwrap();

        let dev = facts_at(&root.join("target/debug/dsb").display().to_string());
        assert_eq!(
            classify(&dev, false),
            Channel::Source {
                checkout: Some(root.clone())
            }
        );
        // The npm package check does not override a checkout binary.
        assert_eq!(
            classify(&dev, true),
            Channel::Source {
                checkout: Some(root)
            }
        );
    }

    #[test]
    fn npm_command_matches_the_documented_install() {
        assert_eq!(
            npm_install_command(),
            "npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build@latest"
        );
        assert_eq!(
            npm_install_argv(),
            vec![
                "install",
                "-g",
                "--allow-scripts=@innocarpe/deepseek-build",
                "@innocarpe/deepseek-build@latest",
            ]
        );
    }

    #[test]
    fn feed_parsers_read_the_shipped_payloads() {
        let npm = r#"{"name":"@innocarpe/deepseek-build","version":"6.9.0","dist":{"tarball":"https://registry.npmjs.org/@innocarpe/deepseek-build/-/deepseek-build-6.9.0.tgz"}}"#;
        assert_eq!(parse_npm_latest(npm).as_deref(), Some("6.9.0"));
        assert_eq!(parse_npm_latest("{\"error\":\"Not found\"}"), None);
        assert_eq!(parse_npm_latest("not json"), None);

        let gh = r#"{"tag_name":"v6.9.0","name":"6.9.0"}"#;
        assert_eq!(parse_gh_tag(gh).as_deref(), Some("6.9.0"));
        assert_eq!(
            parse_gh_tag(r#"{"tag_name":"6.9.0"}"#).as_deref(),
            Some("6.9.0")
        );
        assert_eq!(parse_gh_tag(r#"{"tag_name":""}"#), None);
        assert_eq!(parse_gh_tag("[]"), None);
    }

    #[test]
    fn node_modules_is_a_whole_component() {
        assert!(has_component(
            Path::new("/a/node_modules/b/dsb"),
            "node_modules"
        ));
        assert!(!has_component(
            Path::new("/a/node_modules_backup/dsb"),
            "node_modules"
        ));
    }
}
