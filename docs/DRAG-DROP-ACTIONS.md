# Drag & Drop Actions V2

Drag & Drop V2 turns AURA-2's native local file intake into an explicit, privacy-bounded file inspection surface.

## Principle

Dropping a file is **not** permission to execute it, upload it, remember it or analyze its contents.

The drop event only creates temporary local context.

Content inspection starts only after the user chooses **Inspect** or **Inspect all**.

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
- capability flags such as `canUseVision`, `canInspect` and `canPreviewText`

It does **not** receive the canonical filesystem path.

The Core registry keeps the real canonical path only for the current application session.

## Intake bounds

V2 preserves the V1 intake boundaries:

- maximum 8 submitted paths per drop
- local files only
- directories rejected
- canonicalization before registry storage
- duplicate canonical paths deduplicated
- missing/unreadable files rejected

Dropping a new batch replaces the previous drop registry.

Before any later Reveal, Inspect, Analyze or Vision handoff, Core re-checks the dropped file. If its size or modification timestamp changed since intake, the opaque drop ID is invalidated and the user must drop the file again. This prevents a stale Drop ID from silently reading different content after the initial consent gesture.

## File classification

Extension classification exposes only a coarse local category:

- image
- video
- audio
- document
- archive
- other

Classification itself does not open file contents.

## Inspect

**Inspect** is a separate explicit Read action.

If the Read permission is `never`, inspection is blocked.

If Read is `ask`, the explicit Inspect/Inspect all click is the one-shot user confirmation for the temporary dropped item or batch.

Inspection never returns the real path to React.

### Plain-text preview allowlist

AURA only reads bounded previews for explicitly allowlisted text-like extensions, including:

- TXT / Markdown / CSV
- JSON / YAML / TOML / XML
- HTML / CSS
- JavaScript / TypeScript
- Python / Rust / C / C++ / Java / Kotlin / Go
- SQL
- INI / CONF
- LOG

The preview is bounded by both:

- **64 KiB maximum bytes read**
- **12,000 maximum characters returned**

If a text-like file contains NUL bytes or is not valid UTF-8, AURA falls back to metadata-only inspection.

The preview is temporary UI context. It is not written to Memory, Project Memory, Agents, Automations or Beta diagnostics.

### Images

Inspect can read image dimensions locally.

This is separate from **Use in Vision**.

Inspecting an image does not create a Vision capture and does not start model analysis.

### PDFs, office documents, video, audio and archives

The current V2 Beta step intentionally keeps these formats metadata-only.

In particular:

- PDFs are not text-extracted automatically.
- Office documents are not unpacked.
- Video/audio are not transcoded or transcribed.
- Archives are never extracted by Inspect.

This avoids hidden decompression, codec execution and large background reads while the dedicated context-attachment pipeline is still being designed.

## Inspect all

The Drop Tray includes **Inspect all**.

It applies the same bounded inspection rules to the current temporary batch of at most eight accepted files.

It does not merge previews into Chat and does not trigger a model automatically.

## Reveal in Explorer

Reveal is an explicit Act action.

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

Drag & Drop V2 does not automatically write dropped data into:

- AURA Memory
- Project Memory
- chat history
- Agent state
- Automations
- Beta diagnostics
- telemetry

AURA Beta has no telemetry upload pipeline.

## Context Attachments V1

V2 also includes a dedicated local **context attachment** contract through **Attach to Chat**, **Attach all to Chat** and the **Analyze with AURA** shortcut.

Attach to Chat does not read the file immediately. It adds only the opaque drop ID to the next desktop Chat request. Attached items appear as removable filename chips above the composer.

The user can then write a custom question such as "compare these two files", "what changed between these notes?" or "summarize the important points". If the user presses Send with attachments but no typed text, AURA submits the visible fallback request "Analyze the attached local files."

After Core accepts the message, the frontend automatically detaches those items from the next turn. The temporary drop session itself remains available until replaced or dismissed.

The frontend sends only the selected current opaque drop IDs alongside the visible Chat request. The Core resolves those IDs and builds a separate ephemeral model context.

The attachment context:

- is desktop-only;
- requires Read not to be `never`;
- accepts at most eight current drop IDs;
- never includes canonical filesystem paths;
- is capped at **6,000 characters total** for the local model;
- caps each text-file excerpt again before model handoff;
- labels attached file data as untrusted;
- is supplied as turn-only context, separate from the visible user message;
- is not persisted in the model conversation history;
- leaves only safe filenames as local UI labels on the user message;
- is automatically detached from subsequent Chat turns after the request is accepted.

When any drop attachment is present, AURA deliberately bypasses deterministic action, Routine and Director routing and sends the request only to local model reasoning. Attached file text therefore cannot directly trigger computer actions.

The model runtime also receives a system-level instruction to treat filenames and attachment content as untrusted data rather than commands, including prompt-injection text found inside a file.

**Analyze with AURA** can therefore support requests such as:

- "analyze these files"
- "compare these notes"
- "summarize these documents"

within the formats and preview bounds currently available.

For images, PDFs, Office files, video, audio and archives, the attachment still contains only the metadata that V2 inspection is allowed to expose. Full rich-format extraction remains a later boundary.

## Still out of scope

Not implemented yet:

- opening/executing dropped files
- PDF/Office text extraction
- video/audio transcription
- archive extraction
- copying or moving files
- uploading files
- persistent dropped-file history
- directory drops
- multi-file Agent workflows

Those require separate permission and product boundaries.
