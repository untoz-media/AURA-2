# M007.2 — Screen Region Selection

M007.2 lets the user limit Vision to a specific rectangle instead of exposing the entire desktop.

## Global region shortcut

Default Alpha shortcut:

`Ctrl + Shift + F9`

Flow:

1. move the cursor to the first corner
2. press Ctrl + Shift + F9
3. move to the opposite corner
4. press Ctrl + Shift + F9 again
5. AURA captures only that rectangle

The two points are normalized so dragging direction does not matter.

The Vision workspace also exposes manual X/Y/width/height region capture.

## Safety

Regions must:

- be at least 8×8 pixels
- remain within the Windows virtual desktop
- stay below the global Vision pixel limit

Region selection captures pixels only; it does not click or interact with the selected UI.
