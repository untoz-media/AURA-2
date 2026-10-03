import type {
  DirectorPreset,
  DirectorPresetRunResult,
  ObsConnectionState,
  ObsProductionHealth,
  ObsRuntimeState,
} from "./bridge/types";
import "./feature-pages.css";

type Props = {
  connection: ObsConnectionState;
  runtime: ObsRuntimeState;
  health: ObsProductionHealth | null;
  presets: DirectorPreset[];
  lastRun: DirectorPresetRunResult | null;
  onRun: (presetId: string) => Promise<DirectorPresetRunResult>;
  onOpenSettings: () => void;
};

export default function DirectorMode({
  connection,
  runtime,
  health,
  presets,
  lastRun,
  onRun,
  onOpenSettings,
}: Props) {
  return (
    <section className="feature-page">
      <header className="feature-hero compact">
        <div>
          <span className="feature-kicker">DIRECTOR MODE</span>
          <h2>Production control without leaving AURA.</h2>
          <p>
            Scenes, sources, audio, recording, streaming, health checks and
            reusable production presets are already part of the AURA Core.
          </p>
        </div>
        <button
          type="button"
          className="feature-secondary-button"
          onClick={onOpenSettings}
        >
          OBS settings
        </button>
      </header>

      <div className="computer-grid director-status-grid">
        <article className="computer-context-card">
          <span>OBS</span>
          <strong>{connection.connected ? "Connected" : "Disconnected"}</strong>
          <small>
            {connection.connected
              ? `${connection.host}:${connection.port}`
              : "Open OBS settings to connect"}
          </small>
        </article>
        <article className="computer-context-card">
          <span>Output</span>
          <strong>{runtime.streaming ? "LIVE" : runtime.recording ? "Recording" : "Idle"}</strong>
          <small>
            {runtime.currentProgramScene
              ? `Program · ${runtime.currentProgramScene}`
              : "No Program scene available"}
          </small>
        </article>
        <article className="computer-context-card">
          <span>Health</span>
          <strong>{health?.status.toUpperCase() ?? "—"}</strong>
          <small>{health?.summary ?? "Connect OBS to monitor production health."}</small>
        </article>
      </div>

      <div className="feature-section">
        <div className="feature-section-heading">
          <div>
            <span className="feature-kicker">PRODUCTION PRESETS</span>
            <strong>{presets.length} saved preset{presets.length === 1 ? "" : "s"}</strong>
          </div>
          <button type="button" className="feature-secondary-button" onClick={onOpenSettings}>
            Manage presets
          </button>
        </div>

        {presets.length > 0 ? (
          <div className="director-preset-overview">
            {presets.map((preset) => (
              <article key={preset.id}>
                <div>
                  <strong>{preset.name}</strong>
                  <span>{preset.actions.length} steps</span>
                </div>
                <button
                  type="button"
                  disabled={!connection.connected}
                  onClick={() => void onRun(preset.id)}
                >
                  Run
                </button>
              </article>
            ))}
          </div>
        ) : (
          <div className="feature-empty">
            No Director Mode presets yet. Create one from OBS settings.
          </div>
        )}
      </div>

      {lastRun && (
        <div className="feature-note">
          <strong>
            Last run · {lastRun.presetName} · {lastRun.success ? "Completed" : "Stopped"}
          </strong>
          <span>
            {lastRun.completedSteps}/{lastRun.totalSteps} steps completed.
            {lastRun.error ? ` ${lastRun.error}` : ""}
          </span>
        </div>
      )}
    </section>
  );
}
