# AURA-2 Beta Feedback

AURA-2 Beta feedback should be useful without requiring testers to expose private computer data.

## Bug reports

Use the repository's **AURA-2 Beta bug report** issue form.

Before opening a report:

1. reproduce the issue once more if safe
2. restart AURA and note whether the issue persists
3. open **Settings → Diagnostics**
4. use **Copy diagnostics**
5. review the copied text before sharing it

The diagnostics report is intentionally limited to technical state such as version, runtime status, feature readiness, counts and permission decisions.

It does **not** intentionally include:

- prompts or model responses
- memories
- screenshots or Vision images
- microphone recordings or transcripts
- local file paths
- OBS passwords
- account tokens or credentials

## What makes a useful bug report

Include:

- AURA version
- Windows version
- affected feature
- exact reproduction steps
- expected behaviour
- actual behaviour/error text
- whether restart changes the issue
- privacy-safe Diagnostics report when relevant

## Security issues

Do not open a public issue for a vulnerability that could expose data, bypass permissions, execute unintended actions or compromise the machine.

Use the repository's private **Security Advisory** reporting path instead.

## Feature requests

Describe the user problem first. For actions that read data or control Windows, also describe the permission/confirmation behaviour you expect.

## Beta triage priorities

Release-blocking issues include:

- install/startup failures
- permission bypasses
- unintended destructive/sensitive execution
- data loss
- background Automation escaping its permission policy
- Agent execution outside its validated plan
- crashes that prevent normal startup or shutdown
- inability to stop/pause a running Agent safely

Lower-severity visual polish and feature requests can be tracked without blocking the Beta unless they materially affect usability or safety.
