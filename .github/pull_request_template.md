## Summary

Describe what changed and why.

## Product area

- [ ] Desktop / UI
- [ ] Models / Managed Runtime
- [ ] Computer Control
- [ ] Memory / context
- [ ] Voice
- [ ] Vision
- [ ] OBS / Director Mode
- [ ] Agents / Automations
- [ ] Installer / release tooling
- [ ] Documentation only

## Safety impact

- [ ] This PR does not change permission classification or execution authority.
- [ ] I reviewed any change to Allow / Ask / Never behaviour.
- [ ] Sensitive / Destructive actions still cannot become permanently Allow.
- [ ] Background Automations remain restricted to eligible Read/Act actions with current Allow permission.
- [ ] Model output still cannot directly execute arbitrary shell commands or unknown PC actions.
- [ ] Global Pause behaviour is preserved where relevant.

Explain any safety-impacting change:

<!-- Describe the exact boundary that changed, or write N/A. -->

## Privacy

- [ ] No new automatic telemetry is introduced.
- [ ] No prompts, memories, screenshots, audio, file contents or credentials are uploaded silently.
- [ ] New network activity is documented and user-initiated or explicitly configured.
- [ ] Diagnostics/logging changes avoid secrets and unnecessary local paths.

## Validation

- [ ] `npm run release:check`
- [ ] `npm run telemetry:check`
- [ ] `npm run beta:source-check`
- [ ] frontend TypeScript/Vite build
- [ ] relevant Rust tests
- [ ] Windows smoke test when the change touches native behaviour

## UI changes

Add screenshots or a short description when the visible UI changes.

## Notes / known limitations

List anything reviewers or Beta testers should know.
