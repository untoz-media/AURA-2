# M005.3 — Active Window Context

M005.3 extends current-app awareness with the title of the active foreground window.

This gives AURA lightweight desktop context such as:

- `Brave` → `GitHub - untoz-media/AURA-2`
- `Adobe Premiere Pro` → `Artemis 2 Documentary`
- `Windows Terminal` → the current terminal window title

The feature remains local and text-only. It does not capture screenshots, inspect pixels, OCR the display or read arbitrary UI contents.

## Windows detection

AURA reuses the native foreground-window path introduced in M005.2.

Core APIs:

- `GetForegroundWindow`
- `GetWindowTextLengthW`
- `GetWindowTextW`
- `GetWindowThreadProcessId`
- `OpenProcess`
- `QueryFullProcessImageNameW`

The current context snapshot now contains:

- friendly application name
- process image
- process ID
- known-app flag
- active window title when available
- context source
- capture timestamp

Window title is optional because some Windows surfaces do not expose one.

## AURA focus protection

AURA preserves the M005.2 last-external-context behavior.

When the AURA desktop or overlay takes focus, the most recently observed external snapshot remains available, including its window title.

The returned context is marked with:

`contextSource = lastExternal`

This cache remains RAM-only and is not an application/window history.

## Natural-language commands

Examples:

- `Active window`
- `Current window`
- `What window am I in?`
- `What window is active?`
- `What am I working on?`
- `Current tab`
- `Em que janela estou?`
- `Qual é a janela ativa?`
- `Qual é o separador ativo?`

These commands are classified as **Read**.

They report the active window title and application when the title is available.

## Desktop UI

The desktop now exposes **Active window** alongside **Current app**:

- sidebar runtime/context card
- Chat context strip

Long titles use the existing ellipsis behavior while preserving the full value in the native title tooltip.

## Local-model context

Free-form local inference receives the current desktop snapshot as ephemeral per-turn context.

The context can include:

- current app
- process image
- active window title
- foreground vs last-external source

This context is sent separately from the user message.

It is injected by the Python worker only for the current generation and is **not** stored as a conversation message.

This prevents stale desktop state from accumulating in chat history when the user changes applications or windows.

## Privacy boundary

M005.3 knows the text Windows exposes as the foreground window title.

It does **not**:

- capture screenshots
- inspect screen pixels
- OCR UI text
- enumerate controls inside the active application
- read the contents of documents or webpages
- infer file paths beyond text already present in the title
- persist a history of active windows

Visual understanding remains scoped to **M007 — Vision**.

## Validation

Regression coverage includes:

- active-window natural-language routing
- English and Portuguese variants
- Read permission classification
- Read policy override behavior
- optional window-title shape
- current-app context compatibility

The repository Windows Actions runner has previously failed before executing workflow steps, so the real Windows build/package gate remains required before merging the stacked chain.
