# M004.10 — Production Health Checks

M004.10 turns the AURA OBS integration into a production-monitoring layer.

## OBS metrics

AURA reads `General::stats` from OBS WebSocket v5 / `obws 0.15.0`:

- OBS CPU usage
- OBS memory usage
- available recording-disk space
- active FPS
- average frame render time
- render skipped / total frames
- output skipped / total frames

The stream status also contributes:

- output active state
- reconnecting state
- output congestion
- output bytes
- stream output skipped / total frames
- stream dropped-frame percentage

## Derived metrics

AURA derives:

- render skipped percentage
- output skipped percentage
- stream congestion
- stream dropped-frame percentage percentage
- live bitrate in kbps from the change in `outputBytes` between health samples

The bitrate value represents what OBS is actually sending. It is not treated as a configured target.

## Production health classification

The labels **GOOD**, **WARNING** and **CRITICAL** are AURA operational classifications. They are not statuses provided by OBS.

Current thresholds:

### Critical

- stream is reconnecting
- stream congestion >= 95%
- OBS CPU >= 95%
- available recording disk space < 1 GB
- render skipped frames >= 5%
- output skipped frames >= 5%
- stream dropped frames >= 5%

### Warning

- stream congestion >= 75%
- OBS CPU >= 80%
- available recording disk space < 5 GB
- average frame render time >= 20 ms
- render skipped frames >= 1%
- output skipped frames >= 1%
- stream dropped frames >= 1%

If no warning or critical condition is active, production health is **GOOD**.

Frame-loss percentages are based on the cumulative counters OBS exposes in `GetStats`.

## AURA commands

Production health is a read-only action.

Examples:

- `Check production health`
- `Production health`
- `Check OBS health`
- `How is the production?`
- `Verifica a saúde da produção`
- `Estado da produção`

These commands use the **Read** permission class.

## Settings dashboard

Settings → Integrations → OBS Control now includes **Production Health**.

The panel refreshes every five seconds and displays:

- health status
- FPS
- OBS CPU
- memory
- available disk space
- average frame render time
- render skipped percentage
- output skipped percentage
- stream congestion
- measured bitrate

Health issues are listed below the metrics.

## Sampling

The health dashboard uses a separate five-second polling loop.

Bitrate requires two stream-health samples. The first health sample after connecting or starting a stream therefore has no bitrate value; later samples calculate bitrate from the byte delta and elapsed time.

The bitrate sample is reset when OBS disconnects, connection fails or the stream is offline.

## Scope boundary

M004.10 monitors the production but does not perform automatic corrective actions.

Multi-action production workflows remain isolated to **M004.11 — Director Mode Presets**.

## Validation

Regression coverage includes:

- frame-loss percentage calculation
- GOOD / WARNING / CRITICAL classification
- English and Portuguese production-health routing
- Read permission classification
- Read policy overrides

The repository's GitHub Actions runner provisioning issue still prevents the Windows workflow from reaching its first step.
