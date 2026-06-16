# pyl

[![Crates.io](https://img.shields.io/crates/v/pyl.svg)](https://crates.io/crates/pyl)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/cumulus13/pyl/actions/workflows/ci.yml/badge.svg)](https://github.com/cumulus13/pyl/actions/workflows/ci.yml)
[![Release](https://github.com/cumulus13/pyl/actions/workflows/release.yml/badge.svg)](https://github.com/cumulus13/pyl/actions/workflows/release.yml)

**Smarter Python launcher for Windows** — drop-in `py.exe` replacement with native support for PyPy, Anaconda, Jython, and user-defined aliases.

`py.exe` only understands `-3.11` (CPython shorthand). `pyl` understands `-pypy`, `-pypy3.11`, `-pypy311`, `-anaconda`, `-anaconda3`, and any alias you define.

## Install

```
cargo install pyl
```

Or download a pre-built binary from [Releases](https://github.com/cumulus13/pyl/releases).

Then optionally copy to a directory in your PATH:

```bat
copy %USERPROFILE%\.cargo\bin\pyl.exe C:\Windows\pyl.exe
```

## Quick start

```bat
:: Register PyPy (reads from Windows registry via PEP 514)
pyl add C:\SDK\pypy3.11-v7.3.20-win64\pypy3.exe
pyl add C:\SDK\pypy3.10-v7.3.19-win64\pypy3.exe

:: Or scan an entire directory at once
pyl scan C:\SDK

:: Launch
pyl -pypy -V
pyl -pypy3.11 script.py
pyl -pypy310 -m pip install requests
pyl -3.13 -c "print('hello')"
```

## Version flags

| Flag | Launches |
|---|---|
| `-3.11` | CPython 3.11 (identical to `py -3.11`) |
| `-3.14t` | CPython 3.14 freethreaded |
| `-pypy` | Latest registered PyPy |
| `-pypy3` | Latest PyPy 3.x |
| `-pypy3.11` | PyPy 3.11 exactly |
| `-pypy311` | PyPy 3.11 (compact form) |
| `-pypy310` | PyPy 3.10 |
| `-cpython3.12` | CPython 3.12 (explicit) |
| `-anaconda` | Latest Anaconda |
| `-anaconda3` | Anaconda 3.x |
| `-anaconda2` | Anaconda 2.x |
| `-anaconda27-64` | Exact Anaconda tag match |
| `-conda` | Anaconda (alias) |
| `-jython` | Latest Jython |
| `-V:PyPy/3.11` | Full PEP 514 spec — always works |
| `-V:ContinuumAnalytics/Anaconda3-3.11` | Full spec for Anaconda |
| `-<alias>` | User-defined alias (see below) |

## Commands

### Register interpreters

```bat
pyl add <exe>                         Probe & register any Python/PyPy/Anaconda exe
pyl add <exe> --company X --tag Y     Override auto-detected company and tag
pyl remove <Company> <Tag>            Remove a registered entry
pyl scan <dir>                        Scan directory, register everything found
```

### List & inspect

```bat
pyl -0                                List all registered interpreters (like py -0)
pyl list                              Same with wider columns
pyl which -pypy                       Print resolved exe path without launching
pyl config                            Show config file path and current contents
```

### Aliases

```bat
pyl alias set pp   PyPy/3.11          Create alias: pyl -pp script.py
pyl alias set pypy PyPy/3.11          Override -pypy to always pick 3.11
pyl alias list                        Show all aliases
pyl alias remove pp                   Remove alias
```

## Config file

Located at `%APPDATA%\pyl\pyl.toml` (created on first `alias set`):

```toml
[aliases]
pp      = "PyPy/3.11"
pypy    = "PyPy/3.11"
pypy3   = "PyPy/3.11"
pypy310 = "PyPy/3.10"

[defaults]
# Interpreter used when pyl is called with no flag
python = "PythonCore/3.13"
```

## How it works

`pyl` reads `HKCU\SOFTWARE\Python` and `HKLM\SOFTWARE\Python` — the same PEP 514 registry locations as `py.exe`. It matches your flag against built-in patterns and user aliases, then launches the resolved executable directly with stdin/stdout/stderr inherited and exit code forwarded.

`pyl add` and `pyl scan` probe the executable (`python -c "..."`) to detect implementation, version, and architecture, then write the appropriate registry entry under `HKCU` (no admin required).

## Comparison

| Feature | `py.exe` | `pyl` |
|---|---|---|
| CPython `-3.11` | ✅ | ✅ |
| PyPy `-pypy` / `-pypy3.11` | ❌ | ✅ |
| Anaconda `-anaconda` | ❌ | ✅ |
| Jython `-jython` | ❌ | ✅ |
| User aliases | ❌ | ✅ |
| Register new interpreters | ❌ | ✅ (`pyl add`) |
| Scan directory | ❌ | ✅ (`pyl scan`) |
| Single binary, no runtime | ✅ | ✅ |

## Requirements

- Windows 10/11 (x86\_64)
- Rust 1.70+ to build from source

## License

MIT — see [LICENSE](LICENSE)

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)