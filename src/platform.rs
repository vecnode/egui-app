//! Platform-specific detection and theming.
//!
//! egui can be told to use a light or dark theme. On Windows 11 the OS
//! default is dark, so this template forces the dark egui theme there to
//! avoid a jarring mismatch with the rest of the system; on every other
//! platform (and on older Windows builds) the default theme is kept.

/// Windows 11 and later use build numbers >= 22000 (see `CurrentBuild` in the
/// registry key `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`).
#[cfg(target_os = "windows")]
const WINDOWS_11_MIN_BUILD: u32 = 22_000;

/// Reads the Windows build number by querying the registry with `reg.exe`.
///
/// Returns `Some(true)` when the running Windows is 11 or newer, `Some(false)`
/// for older builds, and `None` when the build number could not be determined
/// (for example when `reg.exe` is unavailable or the key cannot be read).
#[cfg(target_os = "windows")]
fn detect_windows_11_or_later() -> Option<bool> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "/v",
            "CurrentBuild",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("CurrentBuild") {
            let build = line.split_whitespace().last()?.parse::<u32>().ok()?;
            return Some(build >= WINDOWS_11_MIN_BUILD);
        }
    }
    None
}

/// Returns `true` when the app should force egui's dark theme.
///
/// On Windows, the dark theme is forced for Windows 11 or newer, and also when
/// the build number cannot be read (failing closed to dark matches the
/// Windows 11 default). On all other platforms this always returns `false` and
/// the default theme is used.
pub fn force_dark_theme_on_windows() -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
    #[cfg(target_os = "windows")]
    {
        match detect_windows_11_or_later() {
            Some(true) | None => true,
            Some(false) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::force_dark_theme_on_windows;

    #[test]
    fn theme_decision_is_total_and_infallible() {
        // Must never panic and must always produce a decision on every host.
        let _ = force_dark_theme_on_windows();
    }
}
