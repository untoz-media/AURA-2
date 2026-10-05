# Security Policy

AURA-2 can control applications and parts of the Windows desktop. Security and permission-bound execution are therefore release-critical.

## Supported versions

| Version | Support |
| --- | --- |
| `0.9.0-beta.x` | Beta/testing-preview security fixes |
| older Alpha builds | Best effort only |

## Reporting a vulnerability

Please **do not open a public GitHub issue** for a vulnerability that could expose data, bypass AURA permissions, execute unintended actions or compromise the host computer.

Use the repository's private GitHub Security Advisory reporting flow:

`Security → Advisories → Report a vulnerability`

Useful reports include the affected AURA version/build provenance, Windows version, component, minimal reproduction steps, expected security boundary, observed behaviour and impact.

Do not include real passwords, tokens, private prompts, personal files or other secrets. Use test data wherever possible.

## High-priority security areas

- bypassing Allow / Ask / Never permission policy
- making Sensitive or Destructive actions permanently Allow
- arbitrary command or shell execution through an Agent plan
- background Automations escaping their permission boundary
- an Agent executing steps outside its validated plan
- secret or credential exposure
- unintended screen/audio/file capture
- unsafe model/runtime download verification
- privilege escalation, remote code execution or undocumented persistence

## Current security boundaries

AURA intentionally keeps model planning separate from deterministic execution, constrains actions to known typed operations, blocks permanent Allow for Sensitive/Destructive classes, limits background Automations, supports Global Pause, and ships the Beta/testing preview with automatic product telemetry and automatic crash uploads disabled.

These controls are defence-in-depth and are not proof that a vulnerability cannot exist.
