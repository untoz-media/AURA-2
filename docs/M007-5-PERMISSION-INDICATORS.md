# M007.5 — Vision Permission Indicators

M007.5 makes screen access visible in the desktop UI.

## Vision indicator

The Vision workspace exposes an explicit screen-access status for:

- region selection
- capture
- local analysis state

The UI describes the current activity instead of silently reading the screen.

## Read permission

Natural Vision queries respect AURA's Read permission policy.

- Read = Never blocks visual capture from Core commands.
- Explicit user capture buttons are direct user-initiated reads.
- Vision does not inherit Act, Modify, Destructive or Sensitive authority.

No background visual monitoring is enabled by M007.
