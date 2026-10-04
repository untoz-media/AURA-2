# M008.2 — Action execution engine

M008.2 adds a deterministic multi-step Agent executor.

## Execution boundary

The local model proposes a plan.

AURA Core executes each validated step.

The model does not receive a generic shell, filesystem mutation API or unrestricted computer-control handle.

## Runtime states

Agent runs expose:

- queued
- running
- paused
- cancelling
- completed
- failed
- cancelled
- interrupted

Each step records:

- index
- label
- status
- attempt count
- result/error
- start time
- completion time

## Controls

Users can:

- run a reviewed plan
- pause an Agent
- resume an Agent
- cancel an Agent

Global AURA Pause also pauses active Agents between safe execution boundaries.

Wait steps are interruptible in short intervals so pause/cancel remains responsive.
