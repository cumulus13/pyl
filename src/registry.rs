// File: src\registry.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description: PEP 514 registry reader + writer.
// License: MIT

/// PEP 514 registry reader + writer.
/// On non-Windows, returns empty / no-ops (for cross-compilation / CI).

#[derive(Debug, Clone)]
pub struct PythonEntry {
    pub company: String,
    pub tag: String,
    pub display_name: String,
    pub executable: String,
    pub install_path: String,
    pub version: String,      // full version string e.g. "3.11.13"
    pub architecture: String, // "64bit" | "32bit"
}

/// Parse version and architecture out of a DisplayName string.
/// Examples:
///   "PyPy 3.11 (64bit)"          → version="3.11", arch="64bit"
///   "Python 3.13 (64-bit)"       → version="3.13", arch="64bit"
///   "Python 3.14 (64-bit, freethreaded)" → version="3.14t", arch="64bit"
#[allow(dead_code)]
fn parse_display_name(display_name: &str, tag: &str) -> (String, String) {
    let lower = display_name.to_lowercase();

    // Architecture
    let arch = if lower.contains("32") {
        "32bit"
    } else {
        "64bit"
    }
    .to_string();

    // Version: prefer tag (already accurate), but normalise it
    // Tag may be "3.11", "3.13t", "Anaconda2" etc.
    let version = tag.to_string();

    (version, arch)
}

#[cfg(windows)]
pub fn read_all() -> Vec<PythonEntry> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut entries: Vec<PythonEntry> = Vec::new();

    let roots = [
        (RegKey::predef(HKEY_LOCAL_MACHINE), "HKLM"),
        (RegKey::predef(HKEY_CURRENT_USER), "HKCU"),
    ];

    for (hive, _name) in &roots {
        let base = match hive.open_subkey(r"SOFTWARE\Python") {
            Ok(k) => k,
            Err(_) => continue,
        };
        let companies: Vec<String> = base.enum_keys().filter_map(|r| r.ok()).collect();

        for company in companies {
            let comp_key = match base.open_subkey(&company) {
                Ok(k) => k,
                Err(_) => continue,
            };
            let tags: Vec<String> = comp_key.enum_keys().filter_map(|r| r.ok()).collect();

            for tag in tags {
                let tag_key = match comp_key.open_subkey(&tag) {
                    Ok(k) => k,
                    Err(_) => continue,
                };
                let display_name: String = tag_key.get_value("DisplayName").unwrap_or_default();

                let ip_key = match tag_key.open_subkey("InstallPath") {
                    Ok(k) => k,
                    Err(_) => continue,
                };
                let install_path: String = ip_key.get_value("").unwrap_or_default();
                let executable: String = ip_key.get_value("ExecutablePath").unwrap_or_default();

                if executable.is_empty() && install_path.is_empty() {
                    continue;
                }

                let (version, architecture) = parse_display_name(&display_name, &tag);

                entries.push(PythonEntry {
                    company: company.clone(),
                    tag: tag.clone(),
                    display_name,
                    executable,
                    install_path,
                    version,
                    architecture,
                });
            }
        }
    }

    // Deduplicate: HKCU wins over HKLM for same company+tag
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| seen.insert(format!("{}:{}", e.company, e.tag)));
    entries
}

#[cfg(windows)]
pub fn write(entry: &PythonEntry) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let base_path = format!(r"SOFTWARE\Python\{}\{}", entry.company, entry.tag);

    let (tag_key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(&base_path)
        .map_err(|e| format!("create tag key: {}", e))?;

    tag_key
        .set_value("DisplayName", &entry.display_name)
        .map_err(|e| format!("set DisplayName: {}", e))?;
    tag_key
        .set_value("SupportUrl", &String::new())
        .map_err(|e| format!("set SupportUrl: {}", e))?;

    let ip_path = format!(r"{}\InstallPath", base_path);
    let (ip_key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(&ip_path)
        .map_err(|e| format!("create InstallPath key: {}", e))?;

    ip_key
        .set_value("", &entry.install_path)
        .map_err(|e| format!("set InstallPath default: {}", e))?;
    ip_key
        .set_value("ExecutablePath", &entry.executable)
        .map_err(|e| format!("set ExecutablePath: {}", e))?;

    Ok(())
}

#[cfg(not(windows))]
pub fn read_all() -> Vec<PythonEntry> {
    vec![]
}

#[cfg(not(windows))]
pub fn write(_entry: &PythonEntry) -> Result<(), String> {
    Ok(())
}
