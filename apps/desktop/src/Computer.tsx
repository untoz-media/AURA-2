import type {
  CurrentAppInfo,
  ObsConnectionState,
  ObsRuntimeState,
  RecentFilesSnapshot,
  RuntimeState,
} from "./bridge/types";
import "./feature-pages.css";

type Props = {
  currentApp: CurrentAppInfo | null;
  runtimeState: RuntimeState;
  obsConnection: ObsConnectionState;
  obsRuntime: ObsRuntimeState;
  recentFiles: RecentFilesSnapshot;
  onRecentFilesRefresh: () => Promise<RecentFilesSnapshot>;
  onCommand: (command: string) => Promise<void>;
};

export default function Computer({
  currentApp,
  runtimeState,
  obsConnection,
  obsRuntime,
  recentFiles,
  onRecentFilesRefresh,
  onCommand,
}: Props) {
  const contextualWindowActions = currentApp?.knownApp
    ? [
        {
          label: `Minimize ${currentApp.appName}`,
          command: `Minimize ${currentApp.appName}`,
        },
        {
          label: `Maximize ${currentApp.appName}`,
          command: `Maximize ${currentApp.appName}`,
        },
        {
          label: `Restore ${currentApp.appName}`,
          command: `Restore ${currentApp.appName}`,
        },
      ]
    : [];

  const actions = [
    ...contextualWindowActions,
    { label: "Show windows", command: "Show windows" },
    { label: "Current app", command: "What app am I using?" },
    { label: "Recent files", command: "Recent files" },
    { label: "Production health", command: "Check production health" },
    { label: "List OBS scenes", command: "List OBS scenes" },
  ];

  return (
    <section className="feature-page">
      <header className="feature-hero compact">
        <div>
          <span className="feature-kicker">COMPUTER CONTROL</span>
          <h2>Your PC, as context and capability.</h2>
          <p>
            AURA can inspect allowed system state and execute computer actions
            through the permission model you control.
          </p>
        </div>
      </header>

      <div className="computer-grid">
        <article className="computer-context-card">
          <span>Current app</span>
          <strong>{currentApp?.appName ?? "Detecting…"}</strong>
          <small>{currentApp?.windowTitle ?? currentApp?.processName ?? "Waiting for Windows"}</small>
          <em>
            {runtimeState.paused
              ? "Awareness paused"
              : currentApp?.contextSource === "lastExternal"
                ? "Last external context"
                : "Foreground context"}
          </em>
        </article>

        <article className="computer-context-card">
          <span>AURA runtime</span>
          <strong>{runtimeState.paused ? "Paused" : "Ready"}</strong>
          <small>
            {runtimeState.backgroundEnabled
              ? "Background mode enabled"
              : "Foreground-only mode"}
          </small>
          <em>{runtimeState.autostartEnabled ? "Starts with Windows" : "Manual start"}</em>
        </article>

        <article className="computer-context-card">
          <span>OBS Studio</span>
          <strong>{obsConnection.connected ? "Connected" : "Disconnected"}</strong>
          <small>
            {obsConnection.connected
              ? `${obsConnection.host}:${obsConnection.port}`
              : "Connect from Director Mode / Settings"}
          </small>
          <em>
            {obsRuntime.streaming
              ? "Stream live"
              : obsRuntime.recording
                ? "Recording"
                : "Idle"}
          </em>
        </article>
      </div>

      <div className="feature-section">
        <div className="feature-section-heading">
          <div>
            <span className="feature-kicker">RECENT CONTEXT</span>
            <strong>Windows Recent Items, without scanning your drives.</strong>
          </div>
          <button
            type="button"
            className="feature-secondary-button"
            onClick={() => void onRecentFilesRefresh()}
            disabled={runtimeState.paused}
          >
            Refresh
          </button>
        </div>

        {recentFiles.items.length > 0 ? (
          <div className="recent-files-list">
            {recentFiles.items.map((item) => (
              <article key={`${item.name}-${item.modifiedAtMs}`}>
                <div>
                  <strong>{item.name}</strong>
                  <span>Windows Recent Items</span>
                </div>
                <time dateTime={new Date(item.modifiedAtMs).toISOString()}>
                  {item.modifiedAtMs > 0
                    ? new Date(item.modifiedAtMs).toLocaleString()
                    : "Time unavailable"}
                </time>
              </article>
            ))}
          </div>
        ) : (
          <div className="feature-empty">No recent Windows items are available.</div>
        )}
      </div>

      <div className="feature-section">
        <div className="feature-section-heading">
          <div>
            <span className="feature-kicker">QUICK ACTIONS</span>
            <strong>Ask the computer layer directly.</strong>
            {currentApp?.knownApp && (
              <small>
                Window controls target {currentApp.appName}
                {currentApp.contextSource === "lastExternal"
                  ? " from your last external context."
                  : "."}
              </small>
            )}
          </div>
        </div>
        <div className="quick-action-grid">
          {actions.map((action) => (
            <button
              type="button"
              className="quick-action-card"
              key={action.command}
              onClick={() => void onCommand(action.command)}
              disabled={runtimeState.paused}
            >
              <strong>{action.label}</strong>
              <span>{action.command}</span>
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
