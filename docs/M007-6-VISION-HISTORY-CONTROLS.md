# M007.6 — Vision History Controls

M007.6 adds privacy-first local history controls.

## Default

Vision history is OFF.

Screenshot retention is OFF.

## Options

Users can choose:

- save text prompt + analysis history
- retain screenshots locally
- retain up to 1–50 history items
- clear all Vision history

If history is disabled, image retention is forcibly disabled.

If text history is enabled but screenshot retention is off, the screenshot is deleted after analysis and only the prompt/result text remains.

If screenshot retention is enabled, AURA copies the screenshot into its local application data history folder and removes the ephemeral capture.

## Storage

- preferences: `vision-preferences.json`
- text history: `vision-history.json`
- optional images: local AURA application data

History is never uploaded by this subsystem.
