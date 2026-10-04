# Security Policy

AURA-2 can control applications and parts of the Windows desktop. Security and permission-bound execution are therefore release-critical.

## Supported versions

| Version | Support |
| --- | --- |
| `0.9.0-beta.x` | Beta security fixes |
| pre-Beta Alpha builds | Best effort only |

## Reporting a vulnerability

Please **do not open a public GitHub issue** for a vulnerability that could expose data, bypass AURA permissions, execute unintended actions or compromise the host computer.

Use the repository's private GitHub Security Advisory reporting flow:

`Security → Advisories → Report a vulnerability`

Useful reports include:

- affected AURA version
- affected Windows version
- component involved
- minimal reproduction steps
- expected security boundary
- observed behaviour
- impact
- whether the issue requires prior user approval or interaction

Do not include real passwords, tokens, private prompts, personal files or other secrets. Use test data wherever possible.

## High-priority security areas

Examples include:

- bypassing Allow / Ask / Never permission policy
- making Sensitive or Destructive actions permanently Allow
- arbitrary command or shell execution through an Agent plan
- background Automations running Modify/Sensitive/Destructive operations
- an Agent executing steps that were not in the validated plan
- secret or credential exposure
- unintended screen/audio capture
- unsafe model/runtime download verification
- privilege escalation
- remote code execution
- persistence outside documented AURA settings

## AURA security boundaries

The current architecture intentionally keeps:

- model planning separate from deterministic execution
- actions constrained to known typed operations
- Sensitive and Destructive permanent Allow blocked by AURA Core
- background Automations limited to eligible Read/Act actions with current Allow permission
- Global Pause able to suspend Agents and Automation scheduling
- automatic product telemetry Off in the Beta

These controls are defence-in-depth and should not be treated as proof that a vulnerability cannot exist.

## Disclosure

Please allow time for a fix to be prepared and tested before public disclosure of a confirmed vulnerability.
