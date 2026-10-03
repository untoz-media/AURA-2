# AURA Model Manager

The AURA Model Manager makes local model installation explicit inside the desktop application instead of hiding large downloads behind first-run inference.

## Current catalog

### AURA-1

AURA-1 currently uses the same upstream runtime model as the original AURA-1 application:

`Qwen/Qwen3-4B-Instruct-2507`

- Source: Hugging Face
- License: Apache-2.0
- Approximate repository size: 8.06 GB
- Runtime manifest: 12 required files

The Model Manager downloads:

- LICENSE
- README.md
- config.json
- generation_config.json
- merges.txt
- model-00001-of-00003.safetensors
- model-00002-of-00003.safetensors
- model-00003-of-00003.safetensors
- model.safetensors.index.json
- tokenizer.json
- tokenizer_config.json
- vocab.json

### AURA-2

AURA-2 remains visible in the catalog but is marked unavailable.

The repository does not currently define a real AURA-2 checkpoint, so the application deliberately does not substitute another model or expose a fake Download action.

When a real checkpoint is defined, it can be added to the same Model Manager manifest.

## Storage

Large model files are stored under Tauri's app-local-data directory:

`<AppLocalData>/models/`

Configuration such as the selected model is stored separately in the app configuration directory:

`model-manager.json`

This keeps multi-gigabyte model data out of roaming configuration storage.

## Download staging

Downloads are never written directly into an installed model directory.

AURA uses:

`models/<model-id>.partial/`

Only after all required files complete and pass verification does AURA rename the staging directory to:

`models/<model-id>/`

This makes final installation atomic on the same filesystem.

## Resume

AURA supports HTTP Range resume.

If the app closes or a transfer is interrupted, partial files remain in the staging directory.

On the next Download / Resume action AURA:

1. inspects remote file sizes;
2. checks existing partial byte counts;
3. resumes compatible files from the existing byte offset;
4. restarts an individual file if the origin does not honor Range.

## Pause and cancel

### Pause

Pause keeps:

- the background transfer task;
- the partial files;
- the current byte position.

Resume continues the existing task.

### Cancel

Cancel:

- wakes a paused transfer if necessary;
- stops the transfer;
- removes the staging directory;
- returns the model to Not Installed.

## Disk-space protection

Before starting a model download AURA checks available space on the filesystem containing the local model directory.

For AURA-1 it requires approximately:

`remaining model bytes + 1 GB headroom`

Partial bytes already present in the staging directory are taken into account.

## Progress events

The Rust backend emits:

`aura:model-download`

The desktop receives:

- model ID
- state
- downloaded bytes
- total bytes when known
- percentage
- average session bytes/second
- current file
- error
- updated timestamp

The Models workspace displays these values live.

## Remote-size discovery

AURA first tries an HTTP HEAD request.

If the CDN does not expose a usable Content-Length, AURA falls back to:

`Range: bytes=0-0`

and reads the total from Content-Range when available.

## Verification

Before final installation AURA verifies:

- every required manifest file exists;
- no file is empty;
- downloaded file sizes match the sizes observed from the source when available;
- the install marker contains the complete runtime manifest;
- the install marker belongs to the expected model;
- source repository matches the current model definition;
- source revision matches the current model definition;
- total installed bytes match the marker.

A broken or manually altered installation is surfaced as Failed rather than silently treated as valid.

## Active model

Installed models can be selected with **Use model**.

The selection is persisted in:

`model-manager.json`

When the first model finishes installing and no model is selected, AURA selects it automatically.

If an active model is removed manually or becomes invalid, the catalog repairs the stale active selection.

### Current scope

Active-model selection is implemented and persistent.

Free-form conversation inference has **not yet** been wired into the AURA-2 chat runtime. The next Model Runtime step will use the selected installation when AI reasoning is required.

Deterministic computer and OBS actions remain independent from the LLM runtime.

## Removal

Remove deletes:

- the verified installation directory;
- any partial staging directory for that model.

If the removed model was selected, the active-model selection is cleared.

A model cannot be removed while an active transfer exists; the transfer must be cancelled first.

## Security

Filesystem operations only accept IDs from the built-in model catalog.

Arbitrary model IDs cannot be used to construct deletion paths outside the model directory.

## UI states

The Models workspace supports:

- Not installed
- Partial download
- Downloading
- Paused
- Installed
- Active
- Failed
- Not released

Actions include:

- Download
- Resume download
- Pause
- Resume
- Cancel
- Retry
- Use model
- Remove
- Clear files
