# Changelog

All notable changes to `pyl` are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2025-06-15

### Added
- Initial release
- Drop-in `py.exe` replacement reading PEP 514 Windows registry
- Built-in shortcut flags: `-pypy`, `-pypy3`, `-pypy3.11`, `-pypy311`, `-pypy310`, `-cpython`, `-anaconda`, `-conda`, `-jython`
- Full PEP 514 spec support: `-V:Company/Tag`
- `pyl add <exe>` — probe and register any Python/PyPy/Anaconda executable
- `pyl remove <Company> <Tag>` — remove a registry entry
- `pyl scan <dir>` — scan directory and register all found interpreters
- `pyl -0` / `pyl list` — list all registered interpreters with arch column
- `pyl which <flag>` — resolve flag to exe path without launching
- `pyl alias set/list/remove` — user-defined aliases stored in `%APPDATA%\pyl\pyl.toml`
- `pyl config` — show config file path and contents
- Anaconda auto-detection via `sys.version` and path heuristics
- Automatic `\\?\` prefix stripping from Windows extended paths
- Zero warnings build; single binary, no runtime dependencies
