# M007.3 — Active Window Vision

M007.3 adds screenshot understanding scoped to the app window the user was working in.

## Last external window

When AURA itself has focus, Active-window Vision prefers the cached last external application context.

AURA resolves a visible Windows top-level window from:

- process image
- preferred window title when available

This prevents a visual question typed inside AURA from accidentally analyzing the AURA window instead of the application the user was using.

If the external window can no longer be resolved, AURA falls back to the current Windows foreground window.

## Metadata

Captures can include:

- window title
- window bounds
- capture type
- timestamp

No hidden window content is read; Vision receives only captured pixels.
