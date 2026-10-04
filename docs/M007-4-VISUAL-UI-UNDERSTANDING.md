# M007.4 — Visual UI Understanding

M007.4 adds local screenshot analysis using a feature-specific multimodal model.

## Model

AURA Vision uses:

`HuggingFaceTB/SmolVLM2-500M-Video-Instruct`

Role:

`vision`

The Vision model is separate from AURA-1 / AURA-2 and cannot become the main reasoning model.

## Runtime

A persistent local VisionRuntime:

- uses AURA Managed Python
- loads the model with Transformers
- uses local_files_only
- uses CUDA when available and CPU otherwise
- receives only local screenshot paths
- returns textual visual analysis over NDJSON

## Natural visual queries

Explicit visual questions can route through Vision, including requests about:

- the screen
- the active window
- visible errors
- visible controls
- where a control appears

AURA Vision is prompted to describe only visible information and never claim that it clicked or changed anything.

## Action boundary

Vision understands.

Computer Control acts.

Any click, typing, system action or other mutation remains outside Vision and must pass through the existing deterministic action router and permission model.
