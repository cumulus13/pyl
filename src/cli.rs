// File: src\cli.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description:
// License: MIT

use crate::{config::Config, launch, list, probe, registry, resolve};
use clap_version_flag::colorful_version;

const USAGE: &str = r#"pyl — smarter Python launcher  (drop-in py.exe replacement)

LAUNCH:
  pyl [flag] [args...]              Run Python/PyPy/any interpreter

VERSION FLAGS:
  -3.11                             CPython 3.11  (same as py.exe)
  -3.10                             CPython 3.10
  -pypy                             Latest PyPy (any version)
  -pypy3                            Latest PyPy 3.x
  -pypy3.11                         PyPy 3.11 exactly
  -pypy311                          PyPy 3.11 (compact form)
  -pypy310                          PyPy 3.10
  -cpython3.12                      CPython 3.12 (explicit)
  -anaconda                         Anaconda (latest)
  -conda                            Anaconda (alias)
  -V:PyPy/3.11                      Full PEP 514 spec (always works)
  -V:PythonCore/3.13                Full PEP 514 spec
  -<alias>                          User-defined alias (see: pyl alias)

REGISTER COMMANDS:
  pyl add <exe>                     Probe & register any Python/PyPy exe
  pyl add <exe> --company X --tag Y Override company/tag
  pyl remove <Company> <Tag>        Remove a registered entry
  pyl scan <dir>                    Scan directory & register all found

LIST / INFO COMMANDS:
  pyl -0                            List all registered interpreters
  pyl list                          Same as -0 but wider output
  pyl which <flag>                  Print exe path without launching
  pyl config                        Show config file path & contents

ALIAS COMMANDS:
  pyl alias set <name> <spec>       e.g.  pyl alias set pp PyPy/3.11
  pyl alias list                    Show all aliases
  pyl alias remove <name>           Remove alias

  pyl help                          Show this help

EXAMPLES:
  pyl add C:\SDK\pypy3.10-v7.3.19-win64\pypy3.exe
  pyl add C:\SDK\pypy3.10-v7.3.19-win64\pypy3.exe --company PyPy --tag 3.10
  pyl scan C:\SDK
  pyl remove PyPy 3.10
  pyl -pypy3.11 -V
  pyl -pypy script.py
  pyl -3.13 -m pip install rich
  pyl alias set pp PyPy/3.11
  pyl -pp script.py
  pyl which -pypy
"#;

pub fn run() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    if raw_args.is_empty() {
        print!("{}", USAGE);
        return;
    }

    if raw_args.len() == 2 && (raw_args[1] == "-v" || raw_args[1] == "--version") {
        let version = colorful_version!();
        version.print_and_exit();
    }

    let first = raw_args[0].as_str();

    match first {
        "help" | "--help" | "-h" => {
            print!("{}", USAGE);
            return;
        }
        "-v" | "--version" => {
            let version = colorful_version!();
            version.print_and_exit();
        }
        "-0" | "list" => {
            list::print_all(&registry::read_all());
            return;
        }
        "alias" => {
            cmd_alias(&raw_args[1..]);
            return;
        }
        "config" => {
            cmd_config();
            return;
        }
        "which" => {
            cmd_which(&raw_args[1..]);
            return;
        }
        "add" => {
            cmd_add(&raw_args[1..]);
            return;
        }
        "remove" | "rm" => {
            cmd_remove(&raw_args[1..]);
            return;
        }
        "scan" => {
            cmd_scan(&raw_args[1..]);
            return;
        }
        _ => {}
    }

    // ── launcher mode ────────────────────────────────────────────────────────
    let cfg = Config::load();
    let entries = registry::read_all();
    let (flag_opt, rest) = resolve::parse_flag(&raw_args);

    let entry = match flag_opt {
        None => match &cfg.defaults.python {
            Some(spec) => match resolve::resolve(&format!("-V:{}", spec), &entries, &cfg) {
                resolve::ResolveResult::Found(e) => e,
                resolve::ResolveResult::NotFound(msg) => die(&msg),
                resolve::ResolveResult::NoFlag => unreachable!(),
            },
            None => {
                let mut candidates: Vec<_> = entries
                    .iter()
                    .filter(|e| e.company == "PythonCore")
                    .collect();
                candidates.sort_by(|a, b| version_cmp_str(&b.tag, &a.tag));
                match candidates.first() {
                    Some(e) => (*e).clone(),
                    None => die("no Python installations found. Run  pyl list  to check."),
                }
            }
        },
        Some(ref flag) => match resolve::resolve(flag, &entries, &cfg) {
            resolve::ResolveResult::Found(e) => e,
            resolve::ResolveResult::NotFound(msg) => {
                eprintln!("pyl: {}", msg);
                eprintln!("     Run  pyl -0  to see available interpreters.");
                std::process::exit(1);
            }
            resolve::ResolveResult::NoFlag => unreachable!(),
        },
    };

    launch::launch(&entry, &rest);
}

// ── pyl add ──────────────────────────────────────────────────────────────────

fn cmd_add(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: pyl add <exe> [--company <name>] [--tag <ver>]");
        std::process::exit(1);
    }

    let exe = &args[0];

    // Resolve path without the \\?\ prefix canonicalize adds on Windows
    if !std::path::Path::new(exe).exists() {
        eprintln!("pyl: executable not found: {}", exe);
        std::process::exit(1);
    }
    let abs_exe = probe::resolve_path(exe);

    // Parse optional --company / --tag flags from remaining args
    let mut override_company: Option<String> = None;
    let mut override_tag: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--company" | "-c" => {
                i += 1;
                override_company = args.get(i).cloned();
            }
            "--tag" | "-t" => {
                i += 1;
                override_tag = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    println!("\u{1F50D} Probing  {} ...", abs_exe);
    let info = match probe::probe(&abs_exe) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("pyl: probe failed: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "   {} {}  ({})",
        info.implementation, info.version, info.architecture
    );

    let company =
        override_company.unwrap_or_else(|| match info.implementation.to_lowercase().as_str() {
            "pypy" => "PyPy".to_string(),
            "anaconda" => "ContinuumAnalytics".to_string(),
            "jython" => "Jython".to_string(),
            _ => "PythonCore".to_string(),
        });

    // For Anaconda use a recognisable tag like "Anaconda3-3.11"
    let tag = override_tag.unwrap_or_else(|| {
        if company == "ContinuumAnalytics" {
            format!("Anaconda3-{}", probe::major_minor(&info.version))
        } else {
            probe::major_minor(&info.version)
        }
    });

    let install_path = std::path::Path::new(&abs_exe)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    // Build a clean display name — avoid "Anaconda Anaconda3-3.11 (64bit)"
    let display_name = if company == "ContinuumAnalytics" {
        format!(
            "Anaconda {} ({})",
            probe::major_minor(&info.version),
            info.architecture
        )
    } else {
        format!("{} {} ({})", info.implementation, tag, info.architecture)
    };

    let entry = registry::PythonEntry {
        company: company.clone(),
        tag: tag.clone(),
        display_name: display_name.clone(),
        executable: abs_exe.clone(),
        install_path,
        version: info.version.clone(),
        architecture: info.architecture.clone(),
    };

    println!(
        "\u{1F4DD} Registering  {}  [{}\\{}]",
        display_name, company, tag
    );

    match registry::write(&entry) {
        Ok(()) => {
            println!("\u{2705} Done!");
            println!("   pyl -V:{}/{} -V        \u{2190} verify", company, tag);
            if company == "PyPy" {
                println!(
                    "   pyl -pypy{} -V          \u{2190} shortcut works too",
                    tag.replace('.', "")
                );
            } else if company == "ContinuumAnalytics" {
                println!("   pyl -anaconda -V        \u{2190} shortcut works too");
            }
        }
        Err(e) => {
            eprintln!("pyl: registry write failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── pyl remove ───────────────────────────────────────────────────────────────

fn cmd_remove(args: &[String]) {
    if args.len() < 2 {
        eprintln!("usage: pyl remove <Company> <Tag>   e.g.  pyl remove PyPy 3.10");
        std::process::exit(1);
    }
    let company = &args[0];
    let tag = &args[1];

    println!("🗑️  Removing  {} {} ...", company, tag);

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let mut deleted = false;
        for hive in [
            RegKey::predef(HKEY_CURRENT_USER),
            RegKey::predef(HKEY_LOCAL_MACHINE),
        ] {
            let ip_path = format!(r"SOFTWARE\Python\{}\{}\InstallPath", company, tag);
            let tag_path = format!(r"SOFTWARE\Python\{}\{}", company, tag);
            let _ = hive.delete_subkey(&ip_path);
            if hive.delete_subkey(&tag_path).is_ok() {
                deleted = true;
            }
        }
        if deleted {
            println!("✅ Removed  {} {}", company, tag);
        } else {
            println!("⚠️  Not found in registry:  {} {}", company, tag);
            println!("    Run  pyl -0  to see registered entries.");
        }
    }

    #[cfg(not(windows))]
    println!("(stub: registry remove not available on this platform)");
}

// ── pyl scan ─────────────────────────────────────────────────────────────────

fn cmd_scan(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: pyl scan <directory>");
        std::process::exit(1);
    }
    let dir = &args[0];

    println!("🔎 Scanning  {} ...\n", dir);

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("pyl: cannot read directory: {}", e);
            std::process::exit(1);
        }
    };

    let exe_names = if cfg!(windows) {
        vec!["pypy3.exe", "pypy.exe", "python.exe", "python3.exe"]
    } else {
        vec!["pypy3", "pypy", "python3", "python"]
    };

    let mut found = 0usize;
    let mut failed = 0usize;

    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let subdir = entry.path();

        for &name in &exe_names {
            let exe_path = subdir.join(name);
            if !exe_path.exists() {
                continue;
            }
            let exe_str = exe_path.to_string_lossy().into_owned();
            print!("  {} ... ", exe_str);

            let info = match probe::probe(&exe_str) {
                Ok(i) => i,
                Err(e) => {
                    println!("⚠️  probe failed: {}", e);
                    failed += 1;
                    break;
                }
            };

            let company = match info.implementation.to_lowercase().as_str() {
                "pypy" => "PyPy",
                "anaconda" => "ContinuumAnalytics",
                "jython" => "Jython",
                _ => "PythonCore",
            };
            let tag = if company == "ContinuumAnalytics" {
                format!("Anaconda3-{}", probe::major_minor(&info.version))
            } else {
                probe::major_minor(&info.version)
            };
            let install_path = subdir.to_string_lossy().into_owned();
            let display_name = if company == "ContinuumAnalytics" {
                format!(
                    "Anaconda {} ({})",
                    probe::major_minor(&info.version),
                    info.architecture
                )
            } else {
                format!("{} {} ({})", info.implementation, tag, info.architecture)
            };

            let reg_entry = registry::PythonEntry {
                company: company.to_string(),
                tag: tag.clone(),
                display_name: display_name.clone(),
                executable: exe_str.clone(),
                install_path,
                version: info.version.clone(),
                architecture: info.architecture.clone(),
            };

            match registry::write(&reg_entry) {
                Ok(()) => {
                    println!("✅  {} [{}\\{}]", display_name, company, tag);
                    found += 1;
                }
                Err(e) => {
                    println!("❌  write failed: {}", e);
                    failed += 1;
                }
            }
            break; // found exe in this subdir, move on
        }
    }

    println!();
    if found == 0 && failed == 0 {
        println!("Nothing found. Make sure subdirectories contain python.exe or pypy3.exe.");
    } else {
        println!("{} registered, {} failed.", found, failed);
        if found > 0 {
            println!("Run  pyl -0  to verify.");
        }
    }
}

// ── alias subcommand ─────────────────────────────────────────────────────────

fn cmd_alias(args: &[String]) {
    let mut cfg = Config::load();
    match args.first().map(|s| s.as_str()) {
        Some("list") | None => {
            if cfg.aliases.is_empty() {
                println!("No aliases defined.");
                println!("  pyl alias set pp PyPy/3.11");
            } else {
                println!("\n  {:<16} Spec", "Alias");
                println!("  {}", "─".repeat(40));
                let mut pairs: Vec<_> = cfg.aliases.iter().collect();
                pairs.sort_by_key(|(k, _)| (*k).clone());
                for (k, v) in pairs {
                    println!("  -{:<15} {}", k, v);
                }
                println!();
            }
        }
        Some("set") => {
            if args.len() < 3 {
                eprintln!("usage: pyl alias set <name> <Company/Tag>");
                std::process::exit(1);
            }
            let name = args[1].trim_start_matches('-').to_string();
            let spec = args[2].clone();
            cfg.aliases.insert(name.clone(), spec.clone());
            cfg.save();
            println!("✅  Alias set:  -{} → {}", name, spec);
            println!("    Usage:  pyl -{} script.py", name);
        }
        Some("remove") | Some("rm") => {
            if args.len() < 2 {
                eprintln!("usage: pyl alias remove <name>");
                std::process::exit(1);
            }
            let name = args[1].trim_start_matches('-').to_string();
            if cfg.aliases.remove(&name).is_some() {
                cfg.save();
                println!("✅  Alias removed: -{}", name);
            } else {
                println!("⚠️   Alias not found: -{}", name);
            }
        }
        Some(other) => {
            eprintln!("pyl alias: unknown subcommand '{}'", other);
            std::process::exit(1);
        }
    }
}

// ── config subcommand ────────────────────────────────────────────────────────

fn cmd_config() {
    let path = Config::path();
    println!("Config file: {}", path.display());
    println!();
    if path.exists() {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        println!("{}", text);
    } else {
        println!("(no config file yet — created on first  pyl alias set)");
        println!();
        println!("Example:\n");
        println!(
            r#"[aliases]
pypy    = "PyPy/3.11"
pypy3   = "PyPy/3.11"
pp      = "PyPy/3.11"
pypy310 = "PyPy/3.10"

[defaults]
python = "PythonCore/3.13"
"#
        );
    }
}

// ── which subcommand ─────────────────────────────────────────────────────────

fn cmd_which(args: &[String]) {
    let cfg = Config::load();
    let entries = registry::read_all();

    let flag = match args.first() {
        Some(f) => f.clone(),
        None => {
            eprintln!("usage: pyl which <flag>");
            std::process::exit(1);
        }
    };
    let flag = if flag.starts_with('-') {
        flag
    } else {
        format!("-{}", flag)
    };

    match resolve::resolve(&flag, &entries, &cfg) {
        resolve::ResolveResult::Found(e) => println!("{}", e.executable),
        resolve::ResolveResult::NotFound(msg) => {
            eprintln!("pyl: {}", msg);
            std::process::exit(1);
        }
        resolve::ResolveResult::NoFlag => {}
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn die(msg: &str) -> ! {
    eprintln!("pyl: {}", msg);
    std::process::exit(1);
}

fn version_cmp_str(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> (u32, u32) {
        let mut it = s.splitn(2, '.');
        let major = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let minor = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        (major, minor)
    };
    parse(a).cmp(&parse(b))
}
