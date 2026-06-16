// File: src\probe.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description: Probe a Python/PyPy executable to extract version, implementation, architecture.
// License: MIT

/// Probe a Python/PyPy executable to extract version, implementation, architecture.
use std::process::Command;

#[derive(Debug)]
pub struct ProbeResult {
    pub implementation: String, // "CPython" | "PyPy" | "Anaconda"
    pub version: String,        // "3.11.13"
    pub architecture: String,   // "64bit" | "32bit"
}

pub fn probe(exe: &str) -> Result<ProbeResult, String> {
    // Also grab sys.version for Anaconda detection (contains "Anaconda" or "conda")
    let script = "import sys, platform; \
                  print(platform.python_implementation(), \
                        sys.version.split()[0], \
                        platform.architecture()[0], \
                        'anaconda' if ('anaconda' in sys.version.lower() or \
                                       'conda' in sys.version.lower() or \
                                       'continuum' in sys.version.lower()) else 'std')";

    let out = Command::new(exe)
        .args(["-c", script])
        .output()
        .map_err(|e| format!("failed to run {}: {}", exe, e))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("probe exited non-zero: {}", err.trim()));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = stdout.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(format!("unexpected probe output: {:?}", stdout.trim()));
    }

    let mut implementation = parts[0].to_string();
    let version = parts[1].to_string();
    let architecture = parts[2].to_string();
    let is_conda = parts.get(3).map(|s| *s == "anaconda").unwrap_or(false);

    // Also detect Anaconda by exe path (e.g. C:\SDK\anaconda3\python.exe)
    let path_lower = exe.to_lowercase();
    let is_conda_path = path_lower.contains("anaconda")
        || path_lower.contains("miniconda")
        || path_lower.contains("miniforge")
        || path_lower.contains("mambaforge");

    if is_conda || is_conda_path {
        implementation = "Anaconda".to_string();
    }

    Ok(ProbeResult {
        implementation,
        version,
        architecture,
    })
}

pub fn major_minor(version: &str) -> String {
    let mut it = version.splitn(3, '.');
    let major = it.next().unwrap_or("0");
    let minor = it.next().unwrap_or("0");
    format!("{}.{}", major, minor)
}

/// Strip the \\?\ extended-path prefix Windows canonicalize() adds.
pub fn clean_path(p: &str) -> String {
    p.strip_prefix(r"\\?\").unwrap_or(p).to_string()
}

/// Like std::fs::canonicalize but without the \\?\ prefix and falling back
/// gracefully if the path can't be resolved.
pub fn resolve_path(raw: &str) -> String {
    match std::fs::canonicalize(raw) {
        Ok(p) => clean_path(&p.to_string_lossy()),
        Err(_) => raw.to_string(),
    }
}
