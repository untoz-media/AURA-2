# M005.4 — Recent Files Context

M005.4 gives AURA a bounded, local snapshot of Windows Recent Items.

The goal is lightweight context — what files or documents the user has interacted with recently — without recursively scanning drives or building a separate activity history.

## Source

AURA reads the per-user Windows Recent Items folder:

`%APPDATA%\Microsoft\Windows\Recent`

Only shortcut-style recent entries are considered:

- `.lnk`
- `.url`

AURA does not recursively scan Documents, Downloads, Desktop, project folders or attached drives.

## Snapshot

Each recent item contains:

- display name derived from the Windows recent shortcut name
- shortcut modification timestamp

The desktop API returns up to 12 items by default.

Internal requests are capped at 50 items.

Duplicate names are removed case-insensitively and results are ordered newest-first.

## Natural-language commands

Examples:

- `Recent files`
- `Show recent files`
- `List recent files`
- `What files did I use recently?`
- `What have I been working on?`
- `Ficheiros recentes`
- `Mostra os ficheiros recentes`
- `Que ficheiros usei recentemente?`

Recent-file context is classified as **Read**.

## Computer workspace

The Computer page now contains a **Recent context** section.

It shows:

- recent item name
- Windows Recent Items as the source
- last modified timestamp
- manual Refresh action

The existing Current app card also surfaces the active window title when available.

## Local-model context

Free-form local-model generation can receive a small ephemeral list of the five most recent Windows items alongside the current app and active-window context.

This metadata:

- is generated locally
- is sent only to the local AURA model runtime
- is not added to the visible user message
- is not stored as a conversation turn
- is not written into persistent AURA memory

The snapshot is refreshed from Windows rather than maintained as AURA-owned history.

## Privacy boundary

M005.4 deliberately does not:

- crawl user folders
- index file contents
- open recent files
- resolve or expose arbitrary shortcut target paths
- read document contents
- persist a recent-file timeline
- sync file context to a cloud service

Opening or manipulating files is a separate action capability and should remain subject to the permission engine.

## Validation

Regression coverage includes:

- English recent-file routing
- Portuguese recent-file routing
- Read permission classification
- Read policy overrides
- filtering of non-shortcut payloads
- shortcut display-name extraction

## Roadmap

- M005.1 ✅ Local persistent memory
- M005.2 ✅ Current app awareness
- M005.3 ✅ Active window context
- M005.4 ✅ Recent files context
- M005.5 User-defined routines
- M005.6 Project memory
