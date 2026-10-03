# M004.1 — OBS WebSocket Connection

M004.1 starts **OBS Control / Director Mode** by adding a native OBS WebSocket v5 connection layer to the AURA-2 desktop application.

## Scope

- Connect to an OBS Studio WebSocket server.
- Default local endpoint: `127.0.0.1:4455`.
- Optional WebSocket password authentication.
- Four-second connection timeout so a missing OBS instance does not stall the desktop UI.
- Read the connected OBS Studio version, obs-websocket version and RPC version.
- Expose connection state through the Tauri bridge.
- Disconnect cleanly.
- Show connection controls and status in **Settings → Integrations**.

## Security

The OBS WebSocket password is used only for the connection attempt and is not persisted by AURA-2 in M004.1.

AURA defaults to a local OBS endpoint. Remote/TLS connectivity is intentionally outside the M004.1 scope.

## Desktop bridge

The following Tauri commands are available:

- `get_obs_connection_state`
- `connect_obs`
- `disconnect_obs`

The TypeScript bridge exposes matching helpers and keeps the current OBS connection state in `useAuraBridge`.

## UI

The Integrations settings page now includes:

- host
- port
- password
- Connect / Disconnect
- connection badge
- OBS Studio version
- obs-websocket version
- RPC version
- connection errors

## Validation

Rust unit tests cover the default OBS WebSocket v5 endpoint and target validation.

Full Windows build/installer validation remains dependent on GitHub Actions runner provisioning, which is currently failing before workflow steps start on this repository.
