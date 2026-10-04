# M007.1 — Screenshot Understanding

M007.1 adds explicit local Windows screenshot capture for AURA Vision.

## Capture backend

AURA uses Windows GDI directly through `windows-sys`.

The capture path uses:

- Windows virtual desktop coordinates
- BitBlt with CAPTUREBLT
- 32-bit pixel extraction
- local PNG encoding
- AURA app cache only

No cloud screenshot service is involved.

## Explicit capture

AURA does not continuously screenshot the desktop.

Full-screen capture occurs only after an explicit user request from:

- the Vision workspace
- an explicit visual command such as “look at my screen”

Screenshots are limited to 24 million pixels to prevent accidental excessive allocations.

## Privacy

By default, screenshots are ephemeral and are deleted after analysis or manual clear.
