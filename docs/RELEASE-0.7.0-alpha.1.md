# AURA-2 0.7.0-alpha.1 — Vision

AURA-2 0.7.0-alpha.1 completes M007 — Vision.

## New capabilities

AURA can now:

- capture the Windows desktop explicitly
- capture the last external active window
- select a screen region with Ctrl + Shift + F9
- analyze screenshots locally
- understand visible UI state and text
- answer explicit screen/window questions through AURA Core
- show visible screen-access state
- keep Vision history off by default
- optionally save local text analysis history
- optionally retain screenshots locally
- clear Vision history at any time

## Local model

AURA Vision:

`HuggingFaceTB/SmolVLM2-500M-Video-Instruct`

Role:

`vision`

Feature-specific Vision models cannot become the main AURA assistant model.

## Architecture

```
Explicit capture
→ Windows GDI
→ local PNG
→ SmolVLM2
→ visual analysis text
→ AURA Core / Vision workspace
```

Vision does not execute clicks.

```
Vision understands
→ AURA Core decides
→ Permission Engine
→ Computer Control acts
```

## Privacy defaults

- no continuous screen capture
- no automatic screenshot history
- no screenshot upload
- screenshots deleted after analysis by default
- screenshot retention requires explicit opt-in
- Read=Never blocks Core-triggered visual capture

## Managed runtime upgrade

0.7 introduces Managed Runtime v2 with Pillow verified for local image loading.

Existing v1 runtimes are marked Needs repair instead of being incorrectly reported as Vision-ready.

## Release version

`0.7.0-alpha.1`
