// File: src\list.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description:
// License: MIT

use crate::registry::PythonEntry;

pub fn print_all(entries: &[PythonEntry]) {
    if entries.is_empty() {
        println!("No Python installations found.");
        return;
    }
    println!();
    println!(
        "  {:<30} {:<10} {:<14} {:<8} Executable",
        "DisplayName", "Tag", "Company", "Arch"
    );
    println!("  {}", "─".repeat(102));
    for e in entries {
        let name = if e.display_name.is_empty() {
            format!("{} {}", e.company, e.tag)
        } else {
            e.display_name.clone()
        };
        // Show full version if it's more detailed than the tag (e.g. "3.11.13" vs "3.11")
        let tag_display = if !e.version.is_empty() && e.version != e.tag {
            format!("{} ({})", e.tag, e.version)
        } else {
            e.tag.clone()
        };
        println!(
            "  {:<30} {:<14} {:<14} {:<8} {}",
            trunc(&name, 29),
            trunc(&tag_display, 13),
            trunc(&e.company, 13),
            trunc(&e.architecture, 7),
            e.executable,
        );
    }
    println!();
}

fn trunc(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
