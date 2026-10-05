# M009.1 — Windows Beta Installer

AURA-2 0.9.0-beta.1 is packaged as a Windows NSIS installer through Tauri 2.

## Install mode

The Beta installer is explicitly configured for current-user installation.

This keeps the default install path within the user's Windows profile and avoids requiring administrator privileges for a normal install.

## Installer policy

The Beta candidate config includes:

- NSIS target
- current-user install mode
- AURA icon for installer/uninstaller
- Untoz Start Menu folder
- publisher metadata
- Utility category
- downgrade protection
- local Tauri tools cache under the build target

## CI artifacts

The Windows workflow produces:

- AURA-2 Windows x64 NSIS installer
- SHA-256 checksum file

Artifacts are retained for 14 days.

## Release gate

The Public Beta must not be published until the Windows workflow successfully completes:

1. checkout
2. Node setup
3. Rust setup
4. dependency install
5. icon generation
6. cargo fmt check
7. Rust tests
8. frontend/Tauri build
9. NSIS installer upload
10. SHA-256 generation/upload
