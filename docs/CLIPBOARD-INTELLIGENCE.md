# Clipboard Intelligence V1

AURA-2 can interact with the Windows text clipboard without monitoring it in the background.

## Scope

V1 intentionally supports **text only**.

Implemented:

- explicit read
- explicit write
- explicit clear
- English and Portuguese command aliases
- Computer workspace controls
- bounded read output
- voice privacy suppression for clipboard contents

Not implemented in V1:

- clipboard history
- automatic clipboard monitoring
- image clipboard ingestion
- file/drop clipboard ingestion
- automatic LLM analysis of copied text

## Permission model

Clipboard contents can contain passwords, tokens, private messages, URLs or personal data.

For that reason:

| Action | Permission |
| --- | --- |
| Read clipboard text | Sensitive |
| Write clipboard text | Modify |
| Clear clipboard | Destructive |

The default Permission Engine therefore asks before every clipboard operation.

## Voice privacy

A voice command such as:

`What's in my clipboard?`

still requires Sensitive confirmation.

After approval, the clipboard text is shown in AURA but is marked as a private voice result and is **not passed to auto-speak TTS**.

This prevents AURA from unexpectedly reading a password/token aloud.

## Storage and diagnostics

AURA does not persist a clipboard history.

Clipboard contents are not included in:

- Beta diagnostics
- local health checks
- crash/session markers
- model catalog state
- Automation metadata

The explicit command itself can still exist in the current in-memory Chat conversation, just like other commands typed by the user.

## Bounds

Writes are bounded by the clipboard module and by the desktop command bridge.

Reads returned to Chat are capped to the first 4,000 characters, with the original character count shown when truncation occurs.

## Windows implementation

The implementation uses the native Windows Unicode text clipboard format (`CF_UNICODETEXT`) and opens the clipboard only for the duration of the requested action.

No clipboard format listener or background polling loop is registered.
