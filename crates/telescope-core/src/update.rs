//! Self-update from the latest GitHub release.
//!
//! Each release carries `latest.json`, its Ed25519 signature
//! `latest.json.sig` (hex) and one update package per platform. The manifest
//! only counts if the signature matches [`PUBLIC_KEY`], and a package only if
//! it matches the SHA-256 the manifest gives for it.
//!
//! Installing swaps the files in place; the running app keeps going on the
//! old ones until it restarts:
//! - Windows: a running exe can't be overwritten but can be renamed, so the
//!   old one moves aside to `*.exe.old` (removed on the next start).
//! - macOS: the whole `.app` bundle is replaced next to the old one.
//! - Linux: the AppImage is renamed over the old one. Other installs (the
//!   `.deb`) can't replace themselves and only link to the release.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

use ed25519_dalek::{Signature, VerifyingKey};
use log::info;
use sha2::{Digest, Sha256};

use crate::api::create_client;
use crate::domain::version::is_newer_version;

const REPO: &str = "https://github.com/eve-telescope/telescope-app";

/// Points the updater at another folder with the same files, for testing.
/// The manifest still has to carry a valid signature.
const URL_ENV: &str = "TELESCOPE_UPDATE_URL";

/// Verifies `latest.json.sig`. The private half signs releases in CI; a copy
/// is in 1Password ("Telescope update signing key").
const PUBLIC_KEY: [u8; 32] = [
    0x77, 0x0c, 0x9e, 0x30, 0xdb, 0x42, 0x5b, 0xb0, 0xb6, 0x59, 0x78, 0xa1, 0x12, 0x6b, 0x23, 0x33,
    0xcd, 0xba, 0x55, 0x17, 0x55, 0x05, 0x36, 0xd9, 0xc0, 0x45, 0xfa, 0x40, 0x23, 0x65, 0x31, 0x97,
];

/// Set on the process an update restarts, which then waits for the old one
/// to quit first.
pub const RESTARTED_ENV: &str = "TELESCOPE_UPDATE_RESTARTED";

/// The manifest's key for this build's package.
fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// A newer release, as the signed manifest describes it.
#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
    /// The package for this platform, if the manifest has one.
    package: Option<Package>,
}

#[derive(Clone, Debug)]
struct Package {
    name: String,
    sha256: [u8; 32],
}

impl Release {
    pub fn url(&self) -> String {
        format!("{REPO}/releases/tag/v{}", self.version)
    }

    /// Whether this copy can update itself to the release, rather than only
    /// link to it.
    pub fn installable(&self) -> bool {
        self.package.is_some() && Target::locate().is_ok()
    }
}

/// The newest release if it's newer than this build.
pub async fn check_for_update() -> Result<Option<Release>, String> {
    let current = env!("CARGO_PKG_VERSION");
    info!("Checking for updates (current: v{})", current);
    let base = base_url();
    let Some(manifest) = fetch(&format!("{base}/latest.json")).await? else {
        // No release has a manifest yet.
        return Ok(None);
    };
    let sig = fetch(&format!("{base}/latest.json.sig"))
        .await?
        .ok_or("The update manifest isn't signed")?;
    verify(&manifest, &sig)?;
    let release = parse(&manifest)?;
    if is_newer_version(&release.version, current) {
        info!("Update available: v{} → v{}", current, release.version);
        Ok(Some(release))
    } else {
        info!("App is up to date (v{})", current);
        Ok(None)
    }
}

/// Downloads, verifies and puts the release in place. Returns the executable
/// to start for the new version.
pub async fn install(release: &Release, progress: &AtomicU8) -> Result<PathBuf, String> {
    let package = release
        .package
        .as_ref()
        .ok_or("There's no update package for this system")?;
    let target = Target::locate()?;
    let download = target.download();
    let url = format!("{}/{}", base_url(), package.name);
    let result = match fetch_to(&url, &download, package, progress).await {
        Ok(()) => {
            let (target, download) = (target.clone(), download.clone());
            tokio::task::spawn_blocking(move || {
                target.replace(&download).map_err(|e| target.explain(&e))?;
                Ok(target.exe)
            })
            .await
            .map_err(|e| e.to_string())?
        }
        Err(e) => Err(e),
    };
    let _ = fs::remove_file(&download);
    result
}

/// Starts the installed version next to this one; the caller quits right
/// after.
pub fn restart(exe: &Path) -> Result<(), String> {
    std::process::Command::new(exe)
        .env(RESTARTED_ENV, "1")
        .spawn()
        .map(drop)
        .map_err(|e| format!("Couldn't start the new version: {e}"))
}

/// Leftovers from an earlier update; quiet if there are none.
pub fn clean_up() {
    if let Ok(target) = Target::locate() {
        let _ = fs::remove_file(target.download());
        #[cfg(windows)]
        let _ = fs::remove_file(target.exe.with_extension("exe.old"));
    }
}

/// Where `latest.json` and the packages are: the latest published release.
fn base_url() -> String {
    std::env::var(URL_ENV).unwrap_or_else(|_| format!("{REPO}/releases/latest/download"))
}

/// The body of `url`, or `None` for a 404.
async fn fetch(url: &str) -> Result<Option<Vec<u8>>, String> {
    let failed = |e: &dyn std::fmt::Display| format!("Update check failed: {e}");
    let response = create_client()?
        .get(url)
        .send()
        .await
        .map_err(|e| failed(&e))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = response.error_for_status().map_err(|e| failed(&e))?;
    let body = response.bytes().await.map_err(|e| failed(&e))?;
    Ok(Some(body.to_vec()))
}

fn verify(manifest: &[u8], sig_hex: &[u8]) -> Result<(), String> {
    let bad = || "The update manifest's signature doesn't match".to_string();
    let sig: [u8; 64] = decode_hex(sig_hex.trim_ascii())
        .and_then(|b| b.try_into().ok())
        .ok_or_else(bad)?;
    let key = VerifyingKey::from_bytes(&PUBLIC_KEY).map_err(|e| e.to_string())?;
    key.verify_strict(manifest, &Signature::from_bytes(&sig))
        .map_err(|_| bad())
}

fn parse(manifest: &[u8]) -> Result<Release, String> {
    let bad = |what: &str| format!("Malformed update manifest: {what}");
    let json: serde_json::Value =
        serde_json::from_slice(manifest).map_err(|e| bad(&e.to_string()))?;
    let version = json["version"]
        .as_str()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| bad("version"))?
        .to_string();
    let entry = &json["files"][platform()];
    let package = if entry.is_null() {
        None
    } else {
        let name = entry["name"]
            .as_str()
            .filter(|n| {
                !n.is_empty()
                    && !n.starts_with('.')
                    && n.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
            })
            .ok_or_else(|| bad("package name"))?
            .to_string();
        let sha256 = entry["sha256"]
            .as_str()
            .and_then(|h| decode_hex(h.as_bytes()))
            .and_then(|b| b.try_into().ok())
            .ok_or_else(|| bad("sha256"))?;
        Some(Package { name, sha256 })
    };
    Ok(Release { version, package })
}

fn decode_hex(hex: &[u8]) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    hex.chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

/// Streams the package to `to`, hashing on the way; keeps it only if the
/// hash matches.
async fn fetch_to(
    url: &str,
    to: &Path,
    package: &Package,
    progress: &AtomicU8,
) -> Result<(), String> {
    let failed = |e: &dyn std::fmt::Display| format!("Download failed: {e}");
    let mut response = create_client()?
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| failed(&e))?;
    let total = response.content_length().unwrap_or(0);
    let mut out = File::create(to).map_err(|e| failed(&e))?;
    let mut hasher = Sha256::new();
    let mut done = 0u64;
    while let Some(chunk) = response.chunk().await.map_err(|e| failed(&e))? {
        hasher.update(&chunk);
        out.write_all(&chunk).map_err(|e| failed(&e))?;
        done += chunk.len() as u64;
        if let Some(percent) = (done * 100).checked_div(total) {
            progress.store(percent.min(100) as u8, Ordering::Relaxed);
        }
    }
    out.sync_all().map_err(|e| failed(&e))?;
    if hasher.finalize().as_slice() != package.sha256 {
        return Err("The downloaded update is damaged (checksum mismatch)".into());
    }
    Ok(())
}

/// What an update replaces: the executable (on Linux the AppImage), and on
/// macOS its `.app` bundle.
#[derive(Clone)]
struct Target {
    exe: PathBuf,
    #[cfg(target_os = "macos")]
    bundle: PathBuf,
}

impl Target {
    fn locate() -> Result<Self, String> {
        if cfg!(debug_assertions) {
            return Err("Development builds don't update themselves".into());
        }
        #[cfg(target_os = "linux")]
        {
            let exe = std::env::var_os("APPIMAGE")
                .map(PathBuf::from)
                .ok_or("Only the AppImage updates itself")?;
            Ok(Self { exe })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let exe = std::env::current_exe()
                .and_then(fs::canonicalize)
                .map_err(|e| format!("Can't find the running app: {e}"))?;
            #[cfg(target_os = "macos")]
            {
                let bundle = exe
                    .ancestors()
                    .nth(3)
                    .filter(|b| b.extension().is_some_and(|e| e == "app"))
                    .ok_or("Updates only work for the app from the .dmg")?
                    .to_owned();
                if bundle.to_string_lossy().contains("/AppTranslocation/") {
                    return Err("macOS runs this copy from a read-only place. Move Telescope into Applications, start it from there and update again.".into());
                }
                Ok(Self { exe, bundle })
            }
            #[cfg(not(target_os = "macos"))]
            Ok(Self { exe })
        }
    }

    /// The folder the swap happens in; same file system, so renames work.
    fn dir(&self) -> &Path {
        #[cfg(target_os = "macos")]
        let inner = &self.bundle;
        #[cfg(not(target_os = "macos"))]
        let inner = &self.exe;
        inner.parent().unwrap_or(Path::new("."))
    }

    fn download(&self) -> PathBuf {
        self.dir().join(".telescope-update.part")
    }

    fn explain(&self, e: &io::Error) -> String {
        if e.kind() == io::ErrorKind::PermissionDenied {
            format!(
                "No permission to replace the app in {}",
                self.dir().display()
            )
        } else {
            format!("Installing the update failed: {e}")
        }
    }

    /// Windows and Linux: the package is the gzipped executable or AppImage.
    #[cfg(not(target_os = "macos"))]
    fn replace(&self, package: &Path) -> io::Result<()> {
        let new = self.dir().join(".telescope-update.new");
        let unpacked = (|| {
            let mut gz = flate2::read::GzDecoder::new(File::open(package)?);
            let mut out = File::create(&new)?;
            io::copy(&mut gz, &mut out)?;
            out.sync_all()
        })();
        if let Err(e) = unpacked {
            let _ = fs::remove_file(&new);
            return Err(e);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&new, fs::Permissions::from_mode(0o755))?;
            fs::rename(&new, &self.exe)
        }
        #[cfg(windows)]
        {
            let old = self.exe.with_extension("exe.old");
            let _ = fs::remove_file(&old);
            fs::rename(&self.exe, &old)?;
            fs::rename(&new, &self.exe).inspect_err(|_| {
                let _ = fs::rename(&old, &self.exe);
                let _ = fs::remove_file(&new);
            })
        }
    }

    /// macOS: the package is a `.tar.gz` of `Telescope.app`.
    #[cfg(target_os = "macos")]
    fn replace(&self, package: &Path) -> io::Result<()> {
        let stage = self.dir().join(".telescope-update");
        let _ = fs::remove_dir_all(&stage);
        fs::create_dir_all(&stage)?;
        let result = (|| {
            let status = std::process::Command::new("/usr/bin/tar")
                .arg("-xzf")
                .arg(package)
                .arg("-C")
                .arg(&stage)
                .status()?;
            let new = stage.join("Telescope.app");
            if !status.success() || !new.join("Contents/Info.plist").is_file() {
                return Err(io::Error::other("the update package is incomplete"));
            }
            let old = self.dir().join(".telescope-old.app");
            let _ = fs::remove_dir_all(&old);
            fs::rename(&self.bundle, &old)?;
            if let Err(e) = fs::rename(&new, &self.bundle) {
                let _ = fs::rename(&old, &self.bundle);
                return Err(e);
            }
            let _ = fs::remove_dir_all(&old);
            Ok(())
        })();
        let _ = fs::remove_dir_all(&stage);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex() {
        assert_eq!(decode_hex(b"00ff7a"), Some(vec![0, 255, 0x7a]));
        assert_eq!(decode_hex(b"0"), None);
        assert_eq!(decode_hex(b"zz"), None);
    }

    #[test]
    fn manifest() {
        let json = format!(
            r#"{{"version":"9.1.0","files":{{"{}":{{"name":"telescope-update.gz","sha256":"{}"}}}}}}"#,
            platform(),
            "ab".repeat(32)
        );
        let r = parse(json.as_bytes()).unwrap();
        assert_eq!(r.version, "9.1.0");
        let package = r.package.unwrap();
        assert_eq!(package.name, "telescope-update.gz");
        assert_eq!(package.sha256, [0xab; 32]);
        let without = parse(br#"{"version":"9.1.0","files":{}}"#).unwrap();
        assert!(without.package.is_none());
    }

    #[test]
    fn manifest_rejects_paths() {
        for name in ["../x.gz", "a/b.gz", ".hidden", ""] {
            let json = format!(
                r#"{{"version":"9.1.0","files":{{"{}":{{"name":"{name}","sha256":"{}"}}}}}}"#,
                platform(),
                "ab".repeat(32)
            );
            assert!(parse(json.as_bytes()).is_err(), "{name}");
        }
    }

    /// Signed with the release key the way CI signs (`openssl pkeyutl -sign -rawin`).
    #[test]
    fn release_key_signature() {
        let sig = b"a6a225c9215634baf7058e8d7a02ece804f30985b88d31f0abdf18963c54a25f2cb37145ccfe7075d5896e521aa64c7be02c76f6d937b0f8f1734987e5a9fe04";
        assert!(verify(b"telescope update key\n", sig).is_ok());
        assert!(verify(b"telescope update kez\n", sig).is_err());
    }

    #[test]
    fn signature_must_match() {
        assert!(verify(b"{}", &[b'0'; 128]).is_err());
        assert!(verify(b"{}", b"abc").is_err());
    }
}
