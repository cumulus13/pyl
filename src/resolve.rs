// File: src\resolve.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description: Resolve a flag like `-pypy`, `-pypy3`, `-pypy3.11`, `-3.11`, `-V:PyPy/3.11` into a concrete PythonEntry from the registry.
// License: MIT

use crate::config::Config;
/// Resolve a flag like `-pypy`, `-pypy3`, `-pypy3.11`, `-3.11`, `-V:PyPy/3.11`
/// into a concrete PythonEntry from the registry.
use crate::registry::PythonEntry;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ResolveResult {
    Found(PythonEntry),
    NotFound(String),
    NoFlag, // no version flag given at all — use default
}

/// Parse the first argument if it's a version/launcher flag.
/// Returns (Option<flag_string>, rest_of_args).
pub fn parse_flag(args: &[String]) -> (Option<String>, Vec<String>) {
    if args.is_empty() {
        return (None, vec![]);
    }
    let first = &args[0];

    // Recognised flag patterns:
    //   -V:Company/Tag           (full PEP 514)
    //   -X.Y                     (PythonCore shortcut, e.g. -3.11)
    //   -pypy                    (shortcut)
    //   -pypy3                   (shortcut)
    //   -pypy3.X / -pypy3.XY    (shortcut with version)
    //   -cpython3.11             (explicit)
    //   -anaconda                (shortcut)
    //   -<alias>                 (user-defined alias)
    let is_flag = first.starts_with('-') && first.len() > 1 && {
        let rest = &first[1..];
        rest.starts_with(|c: char| c.is_ascii_digit())
            || rest.starts_with("V:")
            || rest.starts_with("pypy")
            || rest.starts_with("cpython")
            || rest.starts_with("anaconda")
            || rest.starts_with("jython")
            || rest
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
    };

    if is_flag {
        (Some(first.clone()), args[1..].to_vec())
    } else {
        (None, args.to_vec())
    }
}

pub fn resolve(flag: &str, entries: &[PythonEntry], cfg: &Config) -> ResolveResult {
    let key = flag.trim_start_matches('-');

    // 1. Full -V:Company/Tag
    if let Some(spec) = key.strip_prefix("V:") {
        return find_by_spec(spec, entries);
    }

    // 2. Bare version number: -3.11 → PythonCore/3.11
    if key
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        let spec = format!("PythonCore/{}", key);
        return find_by_spec(&spec, entries);
    }

    // 3. User alias
    if let Some(spec) = cfg.resolve_alias(key) {
        return find_by_spec(spec, entries);
    }

    // 4. Built-in pattern matching
    let lower = key.to_lowercase();

    // -pypy3.11 / -pypy311 / -pypy3 / -pypy
    if let Some(rest) = lower.strip_prefix("pypy") {
        let ver = normalise_version(rest);
        return find_company_ver("PyPy", ver.as_deref(), entries);
    }

    // -cpython3.11 / -cpython
    if let Some(rest) = lower.strip_prefix("cpython") {
        let ver = normalise_version(rest);
        return find_company_ver("PythonCore", ver.as_deref(), entries);
    }

    // -anaconda / -conda / -anaconda2 / -anaconda3 / -anaconda27 / -anaconda27-64 etc.
    if lower.starts_with("anaconda") || lower.starts_with("conda") {
        let rest = lower
            .strip_prefix("anaconda")
            .or_else(|| lower.strip_prefix("conda"))
            .unwrap_or("");
        // rest could be "" | "2" | "3" | "27" | "27-64" | "3-3.11" etc.
        // Try exact tag match first (e.g. "Anaconda27-64")
        if !rest.is_empty() {
            // Build candidate tags to try
            let candidates_tags: Vec<String> = vec![
                format!("Anaconda{}", rest),                  // Anaconda27-64
                format!("Anaconda{}", rest.replace('-', "")), // Anaconda2764
                format!("Anaconda3-{}", rest),                // Anaconda3-3.11
            ];
            for try_tag in &candidates_tags {
                let spec = format!("ContinuumAnalytics/{}", try_tag);
                match find_by_spec(&spec, entries) {
                    ResolveResult::NotFound(_) => continue,
                    other => return other,
                }
            }
            // Fall back: find by major version prefix
            // "2" → pick any ContinuumAnalytics entry whose tag starts with "Anaconda2"
            let major = rest
                .chars()
                .next()
                .filter(|c| c.is_ascii_digit())
                .map(|c| c.to_string());
            if let Some(maj) = major {
                let prefix = format!("anaconda{}", maj).to_lowercase();
                let mut found: Vec<&PythonEntry> = entries
                    .iter()
                    .filter(|e| {
                        e.company.eq_ignore_ascii_case("ContinuumAnalytics")
                            && e.tag.to_lowercase().starts_with(&prefix)
                    })
                    .collect();
                if !found.is_empty() {
                    found.sort_by(|a, b| a.tag.cmp(&b.tag));
                    return ResolveResult::Found(found[0].clone());
                }
            }
        }
        // No version or nothing matched — return latest
        return find_company_ver("ContinuumAnalytics", None, entries);
    }

    // -jython
    if let Some(rest) = lower.strip_prefix("jython") {
        let ver = normalise_version(rest);
        return find_company_ver("Jython", ver.as_deref(), entries);
    }

    ResolveResult::NotFound(format!("Unknown flag: -{}", key))
}

/// "PyPy/3.11" → find entry with company=PyPy tag=3.11
fn find_by_spec(spec: &str, entries: &[PythonEntry]) -> ResolveResult {
    let (company, tag) = match spec.split_once('/') {
        Some(p) => p,
        None => return ResolveResult::NotFound(format!("Invalid spec: {}", spec)),
    };
    entries
        .iter()
        .find(|e| e.company.eq_ignore_ascii_case(company) && e.tag == tag)
        .cloned()
        .map(ResolveResult::Found)
        .unwrap_or_else(|| ResolveResult::NotFound(format!("Not found: {}/{}", company, tag)))
}

/// Find by company + optional version prefix. Picks highest tag if no version given.
fn find_company_ver(company: &str, ver: Option<&str>, entries: &[PythonEntry]) -> ResolveResult {
    let mut candidates: Vec<&PythonEntry> = entries
        .iter()
        .filter(|e| e.company.eq_ignore_ascii_case(company))
        .collect();

    if candidates.is_empty() {
        return ResolveResult::NotFound(format!("No {} installations found", company));
    }

    if let Some(v) = ver {
        candidates.retain(|e| e.tag.starts_with(v) || e.tag == v);
        if candidates.is_empty() {
            return ResolveResult::NotFound(format!("No {} {} found", company, v));
        }
    }

    // Sort descending by tag so highest version wins
    candidates.sort_by(|a, b| version_cmp(&b.tag, &a.tag));
    ResolveResult::Found(candidates[0].clone())
}

/// "311" → "3.11", "3.11" → "3.11", "3" → "3", "" → None
fn normalise_version(s: &str) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    // Already has dot
    if s.contains('.') {
        return Some(s.to_string());
    }
    // "311" → "3.11"
    if s.len() >= 3 && s.chars().all(|c| c.is_ascii_digit()) {
        let major = &s[..1];
        let minor = &s[1..];
        return Some(format!(
            "{}.{}",
            major,
            minor.trim_start_matches('0').to_string().pipe_if_empty("0")
        ));
    }
    // "3" or "31"
    Some(s.to_string())
}

trait PipeIfEmpty {
    fn pipe_if_empty(self, fallback: &str) -> String;
}
impl PipeIfEmpty for String {
    fn pipe_if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> (u32, u32) {
        let mut it = s.splitn(2, '.');
        let major = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let minor = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        (major, minor)
    };
    parse(a).cmp(&parse(b))
}
