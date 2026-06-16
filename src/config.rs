// File: src\config.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description:
// License: MIT

use serde::{Deserialize, Serialize};
/// Config file: %APPDATA%\pyl\pyl.toml
///
/// [aliases]
/// pypy    = "PyPy/3.11"
/// pypy3   = "PyPy/3.11"
/// pypy310 = "PyPy/3.10"
/// pypy311 = "PyPy/3.11"
/// pp      = "PyPy/3.11"
/// conda   = "ContinuumAnalytics/Anaconda27-64"
///
/// [defaults]
/// python = "PythonCore/3.13"   # used when no flag given
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default)]
    pub defaults: Defaults,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Defaults {
    pub python: Option<String>,
}

impl Config {
    pub fn path() -> PathBuf {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs_fallback());
        base.join("pyl").join("pyl.toml")
    }

    pub fn load() -> Self {
        let p = Self::path();
        if !p.exists() {
            return Self::default();
        }
        let text = std::fs::read_to_string(&p).unwrap_or_default();
        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let text = toml::to_string_pretty(self).unwrap_or_default();
        let _ = std::fs::write(&p, text);
    }

    /// Resolve an alias name → "Company/Tag" string, if present.
    pub fn resolve_alias<'a>(&'a self, name: &str) -> Option<&'a str> {
        self.aliases.get(name).map(|s| s.as_str())
    }
}

fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
