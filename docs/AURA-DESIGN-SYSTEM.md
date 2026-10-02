# AURA Design System

**Milestone:** M002.3  
**Status:** Initial system complete

AURA-2 should feel like an intelligent layer of the operating system, not a web chatbot inside a desktop wrapper.

## Principles

### Calm
The interface should remain quiet when AURA is idle. Glow, animation and saturated color are reserved for active states.

### Precise
System state, permissions and actions must be clear. Decorative elements must never make an action ambiguous.

### Ambient
AURA can use subtle light, blur and depth to feel present without looking like a game UI.

### Fast
The visual system should support keyboard-first workflows and instant transitions.

### Local
Privacy and local execution should be visible product qualities, not hidden technical details.

## Visual language

### Base
The default UI is dark with near-black navy surfaces.

### Spectrum
AURA's identity uses a restrained cyan → blue → violet spectrum.

- Cyan: `#20C7FF`
- Blue: `#5F7CFF`
- Violet: `#9B5CFF`

The spectrum is an identity accent. It should not become a full-screen gradient.

### Typography

Primary product typeface:

`Sora`

Fallback stack:

`Inter → system UI → Segoe UI`

The desktop app does not currently bundle font files. Until a licensed distribution strategy is added, the fallback stack remains valid offline.

### Status colors

- Idle — neutral
- Listening — cyan
- Thinking — violet
- Working — blue
- Waiting — amber
- Success — green
- Danger — red

Status color must always be paired with text or another non-color signal.

## Component rules

### AURA Mark
The AURA-1 mark is retained as the current product-family mark and has been slightly recolored to the AURA-2 spectrum. It can be replaced later without changing component layouts.

### Navigation
Navigation is low contrast by default. The active item uses a subtle spectrum tint, never a large saturated block.

### Command Bar
The command bar is the primary interaction surface.

It should:
- be visually dominant without filling the window
- clearly accept keyboard focus
- support status/context additions later
- be reusable inside the future Overlay

### Surfaces
Cards use one-pixel translucent borders and shallow contrast changes rather than heavy drop shadows.

## Motion

Motion should communicate state.

- Fast: 120ms
- Standard: 180ms
- Slow: 320ms

Avoid permanent decorative animation. Pulses and glow changes should only communicate listening, thinking, working or other active states.

Respect `prefers-reduced-motion`.

## Overlay direction

The future AURA Overlay should reuse exactly the same:

- typography tokens
- command bar
- status system
- keycaps
- AURA mark
- surfaces

The Overlay should be a compact expression of the desktop app, not a second visual system.

## Source files

- `src/design-system/tokens.css`
- `src/design-system/components.css`
- `src/design-system/components.tsx`
- `public/aura-mark.svg`
