# M009.1 — Beta Installer & Release Packaging

**Milestone:** M009.1  
**Stage:** AURA-2 Beta hardening  
**Base build:** 0.8.0-alpha.1  
**Target:** Windows x64  
**Installer:** NSIS setup executable

M009.1 turns the existing development bundle into a repeatable Beta-grade Windows packaging path.

## Installer policy

The AURA-2 installer now explicitly uses:

- per-user installation (`currentUser`)
- no Administrator requirement for the normal installation path
- LZMA compression
- Untoz Start Menu grouping
- AURA installer and uninstaller icon
- downgrade protection
- embedded Microsoft WebView2 bootstrapper

The WebView2 bootstrapper adds a small amount to the installer but makes the setup path more robust on machines where the runtime must be repaired or installed.

The managed AURA AI runtime is intentionally **not** bundled into the desktop installer.

The expected user flow remains:

`Install AURA → Install AURA Runtime → Download model → Use AURA`

This keeps the core application installer much smaller and lets users choose whether they need the local AI runtime and model weights.

## Release metadata guard

A new root command is available:

```powershell
npm run release:check
```

It fails when release versions drift between:

- root `package.json`
- desktop `package.json`
- Tauri `tauri.conf.json`
- Rust `Cargo.toml`

It also verifies the Beta installer policy:

- NSIS bundling is enabled
- install mode is `currentUser`
- downgrades are blocked
- the WebView2 bootstrapper is embedded

This closes an existing version drift where the root package still reported `0.3.0-alpha.1` while the desktop application was already at `0.8.0-alpha.1`.

## Windows CI artifact

The Windows workflow now:

1. installs dependencies
2. validates release metadata
3. runs Rust Core tests
4. builds the Tauri desktop application
5. creates the NSIS installer
6. calculates a SHA-256 checksum
7. uploads the installer and checksum as the `AURA-2-Windows-x64` artifact

Artifacts are retained for 14 days on development builds.

## Integrity verification

A downloaded Beta build can be verified in PowerShell with:

```powershell
Get-FileHash .\AURA-2_*_x64-setup.exe -Algorithm SHA256
```

Compare the result with the matching `.sha256` file from the GitHub Actions artifact.

## Code signing

M009.1 does not invent or embed a signing identity.

Unsigned internal/Beta builds can still trigger Microsoft Defender SmartScreen warnings. Production code signing should only be enabled after Untoz has a real Windows code-signing certificate and a protected CI signing path.

## Acceptance criteria

- [x] Installer configuration is explicit and reproducible
- [x] Normal install path does not require Administrator privileges
- [x] Accidental downgrade is blocked
- [x] WebView2 bootstrapper is packaged
- [x] Release version drift is caught before packaging
- [x] CI emits a SHA-256 checksum beside the installer
- [x] Existing local AI runtime remains independently installable
- [x] Signing is documented without adding fake credentials

## Next

**M009.2 — Settings & Permissions UI**

The next Beta pass should consolidate the current settings surfaces into a user-facing safety center with clear permission state, privacy controls, reset/export controls and first-run defaults.
