// File: src\launch.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description: Launch the target executable, passing all remaining args.
// License: MIT

use crate::registry::PythonEntry;
use std::process;

/// Launch the target executable, passing all remaining args.
/// On Windows we use a child process (no execv in Win32) but we forward
/// stdin/stdout/stderr and exit with the child's code — feels identical.
pub fn launch(entry: &PythonEntry, args: &[String]) -> ! {
    let exe = resolve_exe(entry);

    let status = process::Command::new(&exe)
        .args(args)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .status();

    match status {
        Ok(s) => process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("pyl: failed to launch {}: {}", exe, e);
            process::exit(127);
        }
    }
}

fn resolve_exe(entry: &PythonEntry) -> String {
    if !entry.executable.is_empty() {
        return entry.executable.clone();
    }
    // Fallback: guess from install_path
    #[cfg(windows)]
    {
        let candidates = if entry.company.eq_ignore_ascii_case("PyPy") {
            vec!["pypy3.exe", "pypy.exe"]
        } else {
            vec!["python.exe", "python3.exe"]
        };
        for name in candidates {
            let p = std::path::Path::new(&entry.install_path).join(name);
            if p.exists() {
                return p.to_string_lossy().into_owned();
            }
        }
    }
    entry.install_path.clone()
}
