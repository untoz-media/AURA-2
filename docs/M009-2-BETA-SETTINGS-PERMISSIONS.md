# M009.2 — Beta Settings & Permissions UX

M009.2 adds first-run Beta onboarding and a permanent Beta & Diagnostics settings area.

## First-run onboarding

The onboarding explains:

- AURA is local-first
- models are installed explicitly
- computer actions are permission-gated
- Sensitive and Destructive actions cannot be permanently allowed
- no usage telemetry is uploaded by this Beta
- no automatic crash reports are uploaded
- Beta features may change

The onboarding can be reopened from Settings.

## Permission review

The Beta onboarding links directly to the existing Permission Engine UI.

The same permission policy controls:

- typed commands
- Voice commands
- Vision reads
- Agents
- Saved AURA Actions
- background Automations

## Recovery notice

AURA records a local session marker.

If the previous process did not record a clean exit, the next launch shows a recovery notice and points the user to local diagnostics.

This is not an automatic crash uploader.
