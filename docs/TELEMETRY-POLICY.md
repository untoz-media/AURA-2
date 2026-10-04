# AURA-2 Beta Telemetry Policy

**Applies to:** AURA-2 Public Beta and pre-Beta builds  
**Default:** Off  
**Last updated:** 2026-10-04

AURA-2 is local-first. The Beta does not automatically send product analytics, usage telemetry or crash reports to Untoz.

## Automatic collection

**None.**

AURA-2 Beta does not automatically transmit:

- prompts or model responses
- memories or project memory
- screenshots or Vision captures
- microphone recordings or Voice transcripts
- file names, file paths or recent-file context
- application/window history
- saved routines or AURA Actions
- Agent plans or Agent run history
- Automation definitions or execution history
- OBS connection details, passwords or production state
- device usage analytics
- feature usage counters
- crash dumps or exception reports

## Local diagnostic data

AURA may store operational state locally when a feature requires it, including settings, permission policy, models, memories, routines, Actions, Automations, Vision history and Agent run history.

Local state is not telemetry and is not uploaded automatically.

## Network activity that is not telemetry

Some user-requested features require network access. Examples include downloading the managed Python/AI runtime, model files or other explicitly requested assets from their upstream sources.

When the user starts such a download, the remote provider may receive ordinary network metadata required to serve the file. AURA does not attach AURA usage analytics, prompts, memories or unrelated local context to those requests.

## Cloud AI

The normal AURA-2 Beta workflow does not require a cloud model. If optional cloud assistance is added later, it must have a separate disclosure describing exactly what content leaves the device before the feature can be enabled.

## Crash and bug reports

The Beta does not automatically upload crash reports.

Users may choose to report a bug manually and decide what logs, screenshots or reproduction details they share. A future in-app crash-reporting feature must be opt-in before transmission and must preview the data being sent.

## Policy changes

Any future telemetry introduced during Beta or Stable development must:

1. be documented before release
2. have a specific product purpose
3. avoid collecting content when aggregate technical signals are sufficient
4. default to Off unless a later release explicitly changes this policy
5. provide a clear user control
6. never silently include prompts, memory, screenshots, audio or file contents

## Beta statement

> AURA-2 Beta sends no automatic product telemetry to Untoz.
