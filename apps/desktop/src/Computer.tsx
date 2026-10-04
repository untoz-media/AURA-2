import { useState } from "react";
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
  const [clipboardDraft, setClipboardDraft] = useState("");
  const [fileSearchDraft, setFileSearchDraft] = useState("");

  const browserSkillTarget =
    currentApp?.appName === "Brave" || currentApp?.appName === "Google Chrome"
      ? currentApp.appName
      : null;

  const browserSkills = browserSkillTarget
    ? [
        {
          label: "New tab",
          command: `New tab in ${browserSkillTarget}`,
        },
        {
          label: "Next tab",
          command: `Next tab in ${browserSkillTarget}`,
        },
        {
          label: "Previous tab",
          command: `Previous tab in ${browserSkillTarget}`,
        },
        {
          label: "Reload",
          command: `Reload ${browserSkillTarget}`,
        },
        {
          label: "Address bar",
          command: `Focus address bar in ${browserSkillTarget}`,
        },
        {
          label: "Reopen tab",
          command: `Reopen closed tab in ${browserSkillTarget}`,
        },
      ]
    : [];

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
    { label: "Read clipboard", command: "Read clipboard" },
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

      <div className="feature-section app-skills-section">
        <div className="feature-section-heading">
          <div>
            <span className="feature-kicker">APP SKILLS</span>
            <strong>
              {browserSkillTarget
                ? `Browser Skills · ${browserSkillTarget}`
                : "Contextual skills for supported applications."}
            </strong>
          </div>
          <span className="feature-badge">
            {browserSkillTarget ? "Browser V1" : "Context aware"}
          </span>
        </div>

        {browserSkillTarget ? (
          <>
            <div className="app-skill-grid">
              {browserSkills.map((skill) => (
                <button
                  key={skill.command}
                  type="button"
                  className="quick-action-card"
                  disabled={runtimeState.paused}
                  onClick={() => void onCommand(skill.command)}
                >
                  <strong>{skill.label}</strong>
                  <span>{skill.command}</span>
                </button>
              ))}
            </div>
            <div className="feature-note">
              <strong>Focus-safe shortcuts</strong>
              <span>
                AURA brings {browserSkillTarget} to the foreground, verifies the
                foreground process and only then sends the bounded browser shortcut.
              </span>
            </div>
          </>
        ) : (
          <div className="feature-empty">
            Bring Brave or Google Chrome into context to expose Browser Skills V1.
            OBS continues to use its deeper Director Mode integration.
          </div>
        )}
      </div>

      <div className="feature-section file-intelligence-section">
        <div className="feature-section-heading">
          <div>
            <span className="feature-kicker">FILE INTELLIGENCE</span>
            <strong>Find files by name without scanning your whole PC.</strong>
          </div>
          <span className="feature-badge">Read only</span>
        </div>

        <div className="file-intelligence-search">
          <input
            type="text"
            maxLength={120}
            value={fileSearchDraft}
            disabled={runtimeState.paused}
            placeholder="Search Desktop, Documents, Downloads, Pictures, Videos and Music…"
            onChange={(event) => setFileSearchDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                const value = fileSearchDraft.trim();
                if (value.length >= 2) {
                  void onCommand(`Find file ${value}`);
                }
              }
            }}
          />
          <button
            type="button"
            className="feature-primary-button"
            disabled={runtimeState.paused || fileSearchDraft.trim().length < 2}
            onClick={() => {
              const value = fileSearchDraft.trim();
              if (value.length < 2) return;
              void onCommand(`Find file ${value}`);
            }}
          >
            Search files
          </button>
        </div>

        <div className="file-intelligence-boundary">
          <span>6 personal folders</span>
          <span>Depth ≤ 4</span>
          <span>≤ 8,000 entries</span>
          <span>No file contents</span>
          <span>No symlink traversal</span>
        </div>

        <div className="file-intelligence-recent">
          <span>Recent file shortcuts</span>
          <div>
            {[
              ["Latest video", "Latest video"],
              ["Latest image", "Latest image"],
              ["Latest download", "Latest download"],
              ["Recent documents", "Recent documents"],
            ].map(([label, command]) => (
              <button
                key={command}
                type="button"
                className="feature-secondary-button"
                disabled={runtimeState.paused}
                onClick={() => void onCommand(command)}
              >
                {label}
              </button>
            ))}
          </div>
          <small>
            Ranked by filesystem modified time. AURA does not infer which app created or exported the file.
          </small>
        </div>
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

      <div className="feature-section clipboard-section">
        <div className="feature-section-heading">
          <div>
            <span className="feature-kicker">CLIPBOARD INTELLIGENCE</span>
            <strong>On-demand only. No clipboard monitoring.</strong>
          </div>
          <span className="feature-badge">Private by default</span>
        </div>

        <div className="clipboard-grid">
          <label className="clipboard-compose">
            <span>Copy text to Windows clipboard</span>
            <textarea
              rows={4}
              maxLength={3000}
              value={clipboardDraft}
              disabled={runtimeState.paused}
              placeholder="Text to place on the clipboard…"
              onChange={(event) => setClipboardDraft(event.target.value)}
            />
            <small>{clipboardDraft.length}/3000 · confirmation required</small>
          </label>

          <div className="clipboard-actions">
            <button
              type="button"
              className="feature-primary-button"
              disabled={runtimeState.paused || !clipboardDraft.trim()}
              onClick={() => {
                const value = clipboardDraft.trim();
                if (!value) return;
                void onCommand(`Copy to clipboard ${value}`);
              }}
            >
              Copy text
            </button>
            <button
              type="button"
              className="feature-secondary-button"
              disabled={runtimeState.paused}
              onClick={() => void onCommand("Read clipboard")}
            >
              Read clipboard
            </button>
            <button
              type="button"
              className="feature-secondary-button danger"
              disabled={runtimeState.paused}
              onClick={() => void onCommand("Clear clipboard")}
            >
              Clear clipboard
            </button>
          </div>
        </div>

        <div className="feature-note clipboard-privacy-note">
          <strong>Privacy boundary</strong>
          <span>
            Reading is Sensitive, writing is Modify and clearing is Destructive.
            AURA never polls the clipboard, never adds clipboard contents to Beta
            diagnostics and does not speak clipboard text aloud for voice commands.
          </span>
        </div>
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
