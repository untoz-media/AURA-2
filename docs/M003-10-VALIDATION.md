# M003.10 — Computer Control Validation

**Milestone:** M003.10  
**Release marker:** AURA-2 0.3.0-alpha.1  
**Scope:** M003 Computer Control

M003.10 is the validation and hardening pass that closes the first complete Windows-control layer of AURA-2.

## Automated validation

The Windows CI pipeline is the release gate for M003.

It must pass:

1. dependency installation
2. Tauri icon generation
3. Rust Core test suite
4. TypeScript + Vite frontend build
5. Rust/Tauri Windows build
6. NSIS installer generation
7. installer artifact upload

The Rust validation suite covers deterministic routing without executing destructive operating-system actions.

### Router matrix

| Area | English | Portuguese | Permission |
| --- | --- | --- | --- |
| Launch app | Open OBS | Abre o OBS | Act |
| Close app | Close OBS | Fecha o OBS | Modify |
| Switch window | Switch to Brave | Vai para o Brave | Act |
| List windows | List windows | Que janelas estão abertas? | Read |
| Keyboard navigation | Press F11 | — | Act |
| Keyboard modify | Press Ctrl+S | — | Modify |
| Keyboard destructive | Press Delete | — | Destructive |
| Type text | Type "Hello" | Escreve "Olá" | Modify |
| Move pointer | Move mouse to X,Y | Move o rato para X,Y | Act |
| Scroll | Scroll down 3 | Scroll para baixo 3 | Act |
| Click | Right click at X,Y | Clique direito em X,Y | Modify |
| Volume query | What is the volume? | Qual é o volume? | Read |
| Exact volume | Set volume to 35% | Define o volume para 35% | Act |
| Media | Pause music | Pausa a música | Act |
| System status | System status | Estado do sistema | Read |
| Settings | Open display settings | Abre as definições de ecrã | Act |
| Lock | Lock PC | Bloqueia o PC | Sensitive |
| Shutdown | Shut down PC | Desliga o PC | Destructive |

### Fail-closed cases

Automated tests require invalid inputs to be rejected:

- exact volume above 100%
- excessive scroll values
- modifier-only keyboard shortcuts
- unknown applications
- unmatched commands

### Permission regression cases

The suite verifies:

- default Read → Allow
- default Act → Allow
- default Modify → Ask
- default Sensitive → Ask
- default Destructive → Ask
- Act can be configured to Never
- Modify can be configured to Allow/Never
- Sensitive cannot be permanently Allow
- Destructive cannot be permanently Allow

## Confirmation validation

M003.9 is part of the M003 release boundary.

Expected behaviour:

- Ask produces a pending confirmation
- Allow executes only the exact pending command
- Cancel consumes the pending request without execution
- approval is one-shot
- confirmation expires after 60 seconds
- command/source mismatch is rejected
- a policy changed to Never remains blocked at approval time

## Manual Windows validation

Some behaviours must not be automatically exercised on a CI runner.

### Desktop / Overlay

- [ ] Ctrl+Shift+Space opens the Overlay
- [ ] Overlay command input focuses correctly
- [ ] Overlay hides before immediate keyboard input
- [ ] Allow / Cancel renders correctly in Overlay
- [ ] Allow / Cancel renders correctly in the main app
- [ ] expired confirmation does not leave the UI stuck

### Applications and windows

- [ ] Open OBS launches OBS when installed
- [ ] Open Brave launches Brave when installed
- [ ] Switch to OBS focuses/restores an existing OBS window
- [ ] List windows returns visible titled windows
- [ ] Close OBS requests confirmation before closing

### Keyboard

- [ ] Press F11 targets the previous app through Overlay
- [ ] Ctrl+S requests confirmation
- [ ] approved Ctrl+S is sent once
- [ ] Unicode text injection preserves Portuguese characters

### Mouse

- [ ] pointer movement works on the primary monitor
- [ ] pointer movement accepts valid negative coordinates on a secondary monitor
- [ ] invalid coordinates are rejected
- [ ] scroll targets the app beneath the Overlay after it hides
- [ ] click requests confirmation before injection

### Audio/media

- [ ] exact master volume reaches requested percentage
- [ ] mute/unmute changes the default render endpoint
- [ ] volume query reports current state
- [ ] Play/Pause controls an active media session

### System

- [ ] System status reports plausible RAM and uptime
- [ ] battery-less desktop reports no fake battery percentage
- [ ] Task Manager opens
- [ ] known Settings pages open
- [ ] Lock/Sleep/Restart/Shutdown never execute without confirmation

## M003 completion criteria

M003 is complete when:

- all automated CI gates pass
- no permission-class regression is present
- invalid deterministic input fails closed
- Windows packaging still succeeds
- the manual validation checklist is available for local hardware verification

M003 completion does **not** mean Vision-driven UI control or arbitrary natural-language automation. Those belong to later milestones.

## Next

**M004 — OBS Control / Director Mode**

M004 builds on the stable Computer Control and Permission layers with direct OBS WebSocket integration instead of visual clicking.
