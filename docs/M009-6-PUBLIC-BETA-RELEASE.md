# M009.6 — Public Beta Release

**Candidate:** AURA-2 0.9.0-beta.1  
**Status:** Prepared, not published

M009.6 prepares the first Public Beta without bypassing the unresolved stability/build gate.

## Already prepared

- [x] version synchronized to `0.9.0-beta.1` in root package, Desktop package, Tauri and Cargo
- [x] About/status identifies the Public Beta candidate
- [x] release notes created
- [x] Beta documentation complete
- [x] telemetry policy complete
- [x] installer/checksum pipeline defined
- [x] Beta source/release guards defined

## Blocking validation

- [ ] Beta Quality workflow executes and passes
- [ ] Rust Core tests execute and pass on Windows
- [ ] NSIS build executes and succeeds
- [ ] installer SHA-256 is produced
- [ ] critical Beta checklist is completed on Windows
- [ ] M009.3 is marked complete
- [ ] `package-lock.json` is generated and committed from a real npm resolution

Do not mark M009.6 complete and do not create the Public Beta tag while any blocking validation item remains open.

## Final command gate

After M009.3 is complete:

```powershell
npm run beta:quality
npm run beta:release-check
```

Or run the combined local gate once dependencies are installed:

```powershell
npm run beta:release-gate
```

## Stacked PR merge order

The current milestone work is intentionally stacked. Merge in dependency order:

1. M008 — PR #48
2. M009.1 — PR #49
3. M009.2 — PR #50
4. M009.3 — PR #51
5. M009.4 — PR #52
6. M009.5 — PR #53
7. M009.6 — release-prep PR

Do not merge a later stacked PR before its base PR unless the branch/base relationship is intentionally rewritten.

## Publication sequence

Once all blockers are cleared:

1. merge the approved stacked PR chain
2. confirm `main` reports `0.9.0-beta.1` everywhere
3. confirm `package-lock.json` exists and install with the locked dependency set
4. run/re-run Beta Quality on the final commit
5. run/re-run Windows Build on the final commit
6. download and verify installer + SHA-256
7. complete smoke tests on the exact installer artifact
8. mark M009.3 complete
9. run `npm run beta:release-gate`
10. create tag `aura-v0.9.0-beta.1` on the validated commit
11. create the GitHub pre-release using `RELEASE-0.9.0-beta.1.md`
12. attach or surface the exact validated Windows installer/checksum artifact
13. mark M009.6 and M009 complete

## Rollback rule

If a release-blocking bug appears after the candidate is built but before publication, do not reuse the validated artifact after changing code. Fix the bug, increment/rebuild as appropriate, and repeat the release gate on the new commit.
