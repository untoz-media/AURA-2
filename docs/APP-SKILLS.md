# App Skills V2

App Skills are deterministic application-specific capabilities built on top of AURA Computer Control.

They are not generic macros and do not require an LLM for execution.

V2 introduces a **Core-owned Skill Registry**. The backend publishes each skill's stable id, app/group, display name, description, command, permission class and current contextual availability. The React Computer workspace renders that registry rather than maintaining a second hard-coded skill list.

## Implemented V1 skills

App Skills V2 currently contains:

- Browser Skills for Brave and Google Chrome
- File Explorer Skills for known personal folders
- Notepad Skills for safe document/navigation actions



## Skill Registry

The registry is the single source of truth for the Computer workspace.

Each published skill contains:

- stable `id`
- `group`
- `appName`
- display `name`
- human-readable `description`
- deterministic `command`
- `permission` class
- `available` state
- whether it is `contextual`

Current catalog size:

- 6 File Explorer skills
- 6 Brave skills
- 6 Google Chrome skills
- 5 Notepad skills

Total: **23 registered skills**.

File Explorer skills are always available. Browser and Notepad skills become available only when their application is current/last-external context.

## Browser Skills V1

Initial supported targets:

- Brave
- Google Chrome

Supported actions:

| Skill | Shortcut |
| --- | --- |
| New tab | Ctrl+T |
| Next tab | Ctrl+Tab |
| Previous tab | Ctrl+Shift+Tab |
| Reload tab | Ctrl+R |
| Focus address bar | Ctrl+L |
| Reopen closed tab | Ctrl+Shift+T |

## Safety model

Browser Skills never inject a shortcut into whichever app happens to have focus.

Execution is:

```text
Natural-language command
        ↓
Deterministic Action Router
        ↓
Known BrowserSkill target/action
        ↓
Act permission policy
        ↓
switch_to_app(Brave/Chrome)
        ↓
short focus-settle delay
        ↓
read current foreground process
        ↓
target verified?
    yes ↓       no → abort
fixed shortcut
```

AURA compares the foreground process image with the known process images for the requested browser.

If the browser does not remain foreground, the skill fails before keyboard injection.

## Commands

English examples:

- `New tab in Brave`
- `Next tab in Chrome`
- `Previous tab in Brave`
- `Reload Chrome`
- `Focus address bar in Brave`
- `Reopen closed tab in Chrome`

Portuguese examples:

- `Novo separador no Brave`
- `Próximo separador no Chrome`
- `Separador anterior no Brave`
- `Recarrega o Chrome`
- `Foca a barra de endereços no Brave`
- `Reabre o separador fechado no Chrome`





## Notepad Skills V1

Notepad skills appear only when Notepad is the current or last external known app.

Supported actions:

| Skill | Shortcut | Permission |
| --- | --- | --- |
| New note | Ctrl+N | Act |
| Find | Ctrl+F | Act |
| Select all | Ctrl+A | Act |
| Undo | Ctrl+Z | Modify |
| Redo | Ctrl+Y | Modify |

Undo and Redo are classified as **Modify** because they change document state, even though they are reversible.

Execution uses the same focus-verification path as Browser Skills:

1. switch to the known Notepad target;
2. wait for Windows focus transition;
3. re-read the foreground process;
4. abort if Notepad is not actually foreground;
5. only then send the fixed shortcut.

V1 deliberately does not expose:

- Save
- Save As
- Close
- arbitrary text insertion

Those actions can write files, overwrite data or affect unsaved work and need their own permission/product design.

## File Explorer Skills V1

AURA can open only the Windows personal folders resolved through the platform path API:

- Desktop
- Documents
- Downloads
- Pictures
- Videos
- Music

Examples:

- `Open Downloads`
- `Open Documents`
- `Abre os Downloads`
- `Abre a pasta vídeos`

Execution uses `explorer.exe` with the resolved path as a direct argument.

AURA does not:

- interpolate a shell command;
- accept an arbitrary folder path through this skill;
- use PowerShell or cmd.exe;
- create, rename, move or delete anything.

These skills use `PermissionClass::Act`.

## Permission boundary

Browser Skills V1 and File Explorer Skills V1 are classified as `PermissionClass::Act`.

They are reversible navigation/state actions and do not:

- type arbitrary user content;
- execute shell commands;
- navigate to arbitrary URLs;
- read page contents;
- close browser tabs/windows;
- submit forms;
- bypass Windows focus restrictions.

Those higher-impact capabilities require separate permission and safety design before they are added.

## UI

Computer → App Skills is contextual.

When Brave or Google Chrome is the current/last external known app, Browser Skills V1 appear as direct controls.

OBS does not use Browser Skills because it already has a deeper WebSocket-based Director Mode integration.
