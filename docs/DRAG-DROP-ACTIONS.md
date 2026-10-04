# Drag & Drop Actions V1

Drag & Drop V1 adds a local file-intake surface to the AURA-2 desktop.

## Principle

Dropping a file is **not** permission to execute it, upload it, remember it or analyze it.

The drop event only creates temporary local context.

## Native event path

The desktop uses Tauri 2's native window drag/drop listener.

The frontend reacts to:

- enter
- over
- drop
- leave

Only the `drop` payload is passed into AURA Core for validation.

## Core intake registry

AURA Core owns an in-memory `DropIntakeState`.

Each accepted item receives an opaque id such as:

`drop-<timestamp>-<counter>`

The React frontend receives:

- opaque id
- filename
- extension
- metadata category
- file size
- modified timestamp
- capability flags such as `canUseVision`

It does **not** receive the canonical filesystem path.

The Core registry keeps the real canonical path only for the current application session.

## Bounds

V1 enforces:

- maximum 8 submitted paths per drop
- local files only
- directories rejected
- canonicalization before registry storage
- duplicate canonical paths deduplicated
- missing/unreadable files rejected

Dropping a new batch replaces the previous drop registry.

## File classification

Metadata-only extension classification:

- image
- video
- audio
- document
- archive
- other

Classification does not open file contents.

## Reveal in Explorer

Reveal is an explicit user action.

The frontend sends only the opaque drop id.

Core resolves that id back to the canonical path and asks File Explorer to select the item.

If the Act policy is `never`, the action is blocked.

For `ask`, pressing the explicit Reveal button is treated as the one-shot UI confirmation for that dropped item.

## AURA Vision handoff

Only supported raster images can be staged:

- PNG
- JPEG/JPG
- WebP
- GIF
- BMP

The original dropped image is never installed as Vision's removable capture.

Instead AURA:

1. resolves the opaque id in Core;
2. verifies the file still exists;
3. verifies Read is not blocked;
4. decodes the local image with the Rust image stack;
5. enforces a 40 MB source limit;
6. enforces the existing 24-million-pixel Vision limit;
7. normalizes the decoded image to PNG inside AURA's Vision cache;
8. creates a `droppedImage` Vision capture pointing only to the cache copy;
9. stores that cache copy as the current Vision capture.

Clearing Vision can therefore delete the cached copy without touching the original dropped file.

No image analysis starts automatically. The user still chooses when to run Vision.

## Privacy

Drag & Drop V1 does not automatically write dropped data into:

- AURA Memory
- Project Memory
- chat history
- Agent state
- Automations
- Beta diagnostics
- telemetry

AURA Beta has no telemetry upload pipeline.

## Out of scope for V1

Not implemented yet:

- opening/executing dropped files
- reading documents automatically
- video/audio transcription
- archive extraction
- copying or moving files
- uploading files
- persistent dropped-file history
- directory drops
- multi-file Agent workflows

Those require separate permission and product boundaries.
