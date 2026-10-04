# M009.2 — Settings & Permissions UI

**Milestone:** M009.2  
**Stage:** AURA-2 Beta hardening  
**Base build:** 0.8.0-alpha.1

M009.2 turns the existing Alpha settings screens into a clearer user-facing Privacy and Safety Center without weakening AURA Core policy enforcement.

## Privacy Center

The Privacy page now explains the actual current product state instead of future milestone placeholders.

It shows:

- local-first processing as the default
- managed local AI runtime/model storage
- permission-based Screen & Vision access
- local Memory / routines / Actions / Automations storage
- cloud routing as not configured
- voice capture as user-controlled
- background execution as permission-bound and pausable

## Safety & Permissions Center

The Permissions page now provides a live summary of the current policy:

- number of permission classes set to Allow
- number set to Ask every time
- number blocked

The five permission classes remain:

- Read
- Act
- Modify
- Sensitive
- Destructive

The user-facing wording is clearer:

- `Allow` — execute without a prompt when the capability itself permits it
- `Ask every time` — require confirmation
- `Block` — never execute

## Core guardrails remain authoritative

The UI is not the security boundary.

AURA Core still:

- rejects permanent Allow for Sensitive
- rejects permanent Allow for Destructive
- sanitizes unsafe persisted policy values on load
- validates Agent plans before execution
- requires approval for higher-risk Agent plans
- restricts unattended Automations to Read/Act actions with current Allow permission
- suspends Agents/Automations when Global Pause is active

These rules are surfaced in Settings so users can understand what the Core is enforcing.

## Beta polish

M009.2 also removes stale Alpha copy that still referenced future M002/M006 work even though those milestones are already complete.

## Acceptance criteria

- [x] Privacy page reflects implemented capabilities
- [x] Permission policy is understandable without knowing internal enum names
- [x] Current Allow / Ask / Block counts are visible
- [x] Sensitive and Destructive permanent Allow remain unavailable
- [x] Core enforcement is clearly surfaced
- [x] Agent and Automation safety rules are visible
- [x] Global Pause state is visible in Privacy and Permissions
- [x] Reset to safe defaults requires explicit confirmation
- [x] Settings remain responsive on narrow windows
- [x] Stale Alpha milestone copy is removed

## Next

**M009.3 — Stability Testing**

The next pass should focus on automated checks, regression coverage, crash/error recovery and Beta release blockers across Computer Control, Voice, Vision, Agents, Automations, Models and OBS.
