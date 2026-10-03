import { useEffect, useState, type ReactNode } from "react";
import type {
  AppStatus,
  PermissionClass,
  PermissionDecision,
  PermissionPolicy,
  RuntimeState,
  ObsConnectRequest,
  ObsConnectionState,
  ObsRuntimeState,
  ObsSceneList,
  ObsSceneSwitchResult,
  ObsRecordingActionResult,
  ObsStreamingActionResult,
  ObsSourceItemList,
  ObsSourceVisibilityResult,
  ObsAudioInputList,
  ObsAudioControlResult,
  ObsProductionHealth,
} from "./bridge/types";
import { SectionLabel, ShortcutKey, Surface } from "./design-system/components";

type SettingsSection =
  | "general"
  | "appearance"
  | "privacy"
  | "permissions"
  | "models"
  | "voice"
  | "overlay"
  | "shortcuts"
  | "integrations";

type Props = {
  activeSection: SettingsSection;
  onSectionChange: (section: SettingsSection) => void;
  appStatus: AppStatus | null;
  runtimeState: RuntimeState;
  onPausedChange: (paused: boolean) => Promise<RuntimeState>;
  onBackgroundChange: (backgroundEnabled: boolean) => Promise<RuntimeState>;
  onAutostartChange: (autostartEnabled: boolean) => Promise<RuntimeState>;
  permissionPolicy: PermissionPolicy;
  onPermissionChange: (
    permissionClass: PermissionClass,
    decision: PermissionDecision,
  ) => Promise<PermissionPolicy>;
  onResetPermissions: () => Promise<PermissionPolicy>;
  obsConnection: ObsConnectionState;
  obsRuntime: ObsRuntimeState;
  obsScenes: ObsSceneList;
  obsSources: ObsSourceItemList;
  obsAudio: ObsAudioInputList;
  obsHealth: ObsProductionHealth | null;
  onObsConnect: (request: ObsConnectRequest) => Promise<ObsConnectionState>;
  onObsDisconnect: () => Promise<ObsConnectionState>;
  onObsRefresh: () => Promise<ObsRuntimeState>;
  onObsScenesRefresh: () => Promise<ObsSceneList>;
  onObsSourcesRefresh: () => Promise<ObsSourceItemList>;
  onObsAudioRefresh: () => Promise<ObsAudioInputList>;
  onObsHealthRefresh: () => Promise<ObsProductionHealth | null>;
  onObsProgramSceneChange: (sceneUuid: string) => Promise<ObsSceneSwitchResult>;
  onObsPreviewSceneChange: (sceneUuid: string) => Promise<ObsSceneSwitchResult>;
  onObsRecordingAction: (
    action: ObsRecordingActionResult["action"],
  ) => Promise<ObsRecordingActionResult>;
  onObsStreamingAction: (
    action: ObsStreamingActionResult["action"],
  ) => Promise<ObsStreamingActionResult>;
  onObsSourceVisibilityChange: (
    sceneName: string,
    itemId: number,
    enabled: boolean,
  ) => Promise<ObsSourceVisibilityResult>;
  onObsAudioMuteChange: (
    inputUuid: string,
    muted: boolean,
  ) => Promise<ObsAudioControlResult>;
  onObsAudioVolumeChange: (
    inputUuid: string,
    percent: number,
  ) => Promise<ObsAudioControlResult>;
};

const sections: Array<{
  id: SettingsSection;
  label: string;
  icon: string;
}> = [
  { id: "general", label: "General", icon: "⌂" },
  { id: "appearance", label: "Appearance", icon: "◐" },
  { id: "privacy", label: "Privacy", icon: "◇" },
  { id: "permissions", label: "Permissions", icon: "✓" },
  { id: "models", label: "Models", icon: "◎" },
  { id: "voice", label: "Voice", icon: "∿" },
  { id: "overlay", label: "Overlay", icon: "▱" },
  { id: "shortcuts", label: "Shortcuts", icon: "⌘" },
  { id: "integrations", label: "Integrations", icon: "⌁" },
];

function SettingRow({
  title,
  description,
  trailing,
}: {
  title: string;
  description: string;
  trailing: ReactNode;
}) {
  return (
    <div className="setting-row">
      <div className="setting-copy">
        <strong>{title}</strong>
        <span>{description}</span>
      </div>
      <div className="setting-trailing">{trailing}</div>
    </div>
  );
}

function Toggle({
  checked,
  onChange,
  disabled = false,
  label,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label: string;
}) {
  return (
    <button
      type="button"
      className={`settings-toggle ${checked ? "checked" : ""}`}
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span />
    </button>
  );
}

function PermissionSelect({
  value,
  onChange,
  allowPermanent = true,
  label,
}: {
  value: PermissionDecision;
  onChange: (decision: PermissionDecision) => void;
  allowPermanent?: boolean;
  label: string;
}) {
  return (
    <select
      className="permission-select"
      value={value}
      aria-label={label}
      onChange={(event) => onChange(event.target.value as PermissionDecision)}
    >
      {allowPermanent && <option value="allow">Allow</option>}
      <option value="ask">Ask</option>
      <option value="never">Never</option>
    </select>
  );
}

function Badge({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: "neutral" | "ready" | "planned" | "warning" | "critical";
}) {
  return <span className={`settings-badge ${tone}`}>{children}</span>;
}


function formatObsDuration(durationMs: number) {
  const totalSeconds = Math.max(0, Math.floor(durationMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return [hours, minutes, seconds]
    .map((value) => String(value).padStart(2, "0"))
    .join(":");
}

export default function Settings({
  activeSection,
  onSectionChange,
  appStatus,
  runtimeState,
  onPausedChange,
  onBackgroundChange,
  onAutostartChange,
  permissionPolicy,
  onPermissionChange,
  onResetPermissions,
  obsConnection,
  obsRuntime,
  obsScenes,
  obsSources,
  obsAudio,
  obsHealth,
  onObsConnect,
  onObsDisconnect,
  onObsRefresh,
  onObsScenesRefresh,
  onObsSourcesRefresh,
  onObsAudioRefresh,
  onObsHealthRefresh,
  onObsProgramSceneChange,
  onObsPreviewSceneChange,
  onObsRecordingAction,
  onObsStreamingAction,
  onObsSourceVisibilityChange,
  onObsAudioMuteChange,
  onObsAudioVolumeChange,
}: Props) {
  const [obsHost, setObsHost] = useState(obsConnection.host);
  const [obsPort, setObsPort] = useState(String(obsConnection.port));
  const [obsPassword, setObsPassword] = useState("");
  const [obsConnecting, setObsConnecting] = useState(false);
  const [obsSceneChanging, setObsSceneChanging] = useState<string | null>(null);
  const [obsSourceChanging, setObsSourceChanging] = useState<number | null>(null);
  const [obsAudioChanging, setObsAudioChanging] = useState<string | null>(null);
  const [obsAudioDrafts, setObsAudioDrafts] = useState<Record<string, number>>({});
  const [obsRecordingAction, setObsRecordingAction] =
    useState<ObsRecordingActionResult["action"] | null>(null);
  const [obsStreamingAction, setObsStreamingAction] =
    useState<ObsStreamingActionResult["action"] | null>(null);
  const [lastRecordingOutput, setLastRecordingOutput] = useState<string | null>(null);
  const [obsStreamClockMs, setObsStreamClockMs] = useState(0);

  useEffect(() => {
    setObsHost(obsConnection.host);
    setObsPort(String(obsConnection.port));
  }, [obsConnection.host, obsConnection.port]);


  useEffect(() => {
    if (!obsRuntime.streaming) {
      setObsStreamClockMs(0);
      return;
    }

    const updateClock = () => {
      const elapsedSinceRefresh = Math.max(
        0,
        Date.now() - obsRuntime.refreshedAtMs,
      );
      setObsStreamClockMs(obsRuntime.streamDurationMs + elapsedSinceRefresh);
    };

    updateClock();
    const interval = window.setInterval(updateClock, 1000);

    return () => window.clearInterval(interval);
  }, [
    obsRuntime.streaming,
    obsRuntime.streamDurationMs,
    obsRuntime.refreshedAtMs,
  ]);

  async function handleObsConnect() {
    const port = Number(obsPort);
    if (!Number.isInteger(port) || port < 1 || port > 65535) {
      return;
    }

    setObsConnecting(true);
    try {
      await onObsConnect({
        host: obsHost.trim() || "127.0.0.1",
        port,
        password: obsPassword || undefined,
      });
      setObsPassword("");
    } catch {
      // Connection feedback is exposed through obsConnection.lastError.
    } finally {
      setObsConnecting(false);
    }
  }

  async function handleObsDisconnect() {
    setObsConnecting(true);
    try {
      await onObsDisconnect();
      setObsPassword("");
    } finally {
      setObsConnecting(false);
    }
  }


  async function handleObsSceneChange(
    target: "program" | "preview",
    sceneUuid: string,
  ) {
    setObsSceneChanging(`${target}:${sceneUuid}`);
    try {
      if (target === "program") {
        await onObsProgramSceneChange(sceneUuid);
      } else {
        await onObsPreviewSceneChange(sceneUuid);
      }
    } catch {
      // Bridge activity/error state already carries the failure details.
    } finally {
      setObsSceneChanging(null);
    }
  }


  async function handleObsSourceVisibility(
    sceneName: string,
    itemId: number,
    enabled: boolean,
  ) {
    setObsSourceChanging(itemId);
    try {
      await onObsSourceVisibilityChange(sceneName, itemId, enabled);
    } catch {
      // Bridge activity/error state already carries the failure details.
    } finally {
      setObsSourceChanging(null);
    }
  }


  async function handleObsAudioMute(inputUuid: string, muted: boolean) {
    setObsAudioChanging(`mute:${inputUuid}`);
    try {
      await onObsAudioMuteChange(inputUuid, muted);
    } catch {
      // Bridge activity/error state already carries the failure details.
    } finally {
      setObsAudioChanging(null);
    }
  }

  async function handleObsAudioVolume(inputUuid: string, fallbackPercent: number) {
    const percent = obsAudioDrafts[inputUuid] ?? fallbackPercent;
    if (!Number.isInteger(percent) || percent < 0 || percent > 100) {
      return;
    }

    setObsAudioChanging(`volume:${inputUuid}`);
    try {
      await onObsAudioVolumeChange(inputUuid, percent);
      setObsAudioDrafts((current) => {
        const next = { ...current };
        delete next[inputUuid];
        return next;
      });
    } catch {
      // Bridge activity/error state already carries the failure details.
    } finally {
      setObsAudioChanging(null);
    }
  }


  async function handleObsRecordingAction(
    action: ObsRecordingActionResult["action"],
  ) {
    setObsRecordingAction(action);
    try {
      const result = await onObsRecordingAction(action);
      if (result.outputPath) {
        setLastRecordingOutput(result.outputPath);
      }
    } catch {
      // Bridge activity/error state already carries the failure details.
    } finally {
      setObsRecordingAction(null);
    }
  }


  async function handleObsStreamingAction(
    action: ObsStreamingActionResult["action"],
  ) {
    setObsStreamingAction(action);
    try {
      await onObsStreamingAction(action);
    } catch {
      // Bridge activity/error state already carries the failure details.
    } finally {
      setObsStreamingAction(null);
    }
  }

  return (
    <section className="settings-layout">
      <aside className="settings-nav" aria-label="Settings sections">
        <div className="settings-nav-heading">Settings</div>
        {sections.map((section) => (
          <button
            type="button"
            key={section.id}
            className={`settings-nav-item ${activeSection === section.id ? "active" : ""}`}
            onClick={() => onSectionChange(section.id)}
          >
            <span aria-hidden="true">{section.icon}</span>
            {section.label}
          </button>
        ))}
      </aside>

      <div className="settings-content">
        {activeSection === "general" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>General</h2>
              <p>Core behaviour and desktop lifecycle.</p>
            </header>

            <Surface className="settings-card">
              <SectionLabel>Runtime</SectionLabel>
              <SettingRow
                title="Pause AURA"
                description="Stops new commands and future background actions until resumed."
                trailing={
                  <Toggle
                    checked={runtimeState.paused}
                    onChange={(value) => void onPausedChange(value)}
                    label="Pause AURA"
                  />
                }
              />
              <SettingRow
                title="Run in background"
                description="Keep AURA, the tray and the Overlay available after closing the main window."
                trailing={
                  <Toggle
                    checked={runtimeState.backgroundEnabled}
                    onChange={(value) => void onBackgroundChange(value)}
                    label="Run AURA in background"
                  />
                }
              />
              <SettingRow
                title="Start with Windows"
                description="Launch AURA silently in the background when you sign in to Windows."
                trailing={
                  <Toggle
                    checked={runtimeState.autostartEnabled}
                    onChange={(value) => void onAutostartChange(value)}
                    label="Start AURA with Windows"
                  />
                }
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>About</SectionLabel>
              <SettingRow
                title={appStatus?.name ?? "AURA-2"}
                description={appStatus?.stage ?? "Desktop Foundation"}
                trailing={<Badge>{appStatus?.version ?? "0.2.0"}</Badge>}
              />
              <SettingRow
                title="Execution model"
                description="AURA keeps local execution as the default."
                trailing={<Badge tone="ready">Local-first</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "appearance" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Appearance</h2>
              <p>Visual behaviour for the desktop app and Overlay.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Theme</SectionLabel>
              <SettingRow
                title="AURA Dark"
                description="Current AURA-2 visual system."
                trailing={<Badge tone="ready">Default</Badge>}
              />
              <SettingRow
                title="Reduced motion"
                description="AURA automatically respects the Windows/browser reduced-motion preference."
                trailing={<Badge tone="ready">System</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "privacy" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Privacy</h2>
              <p>Control what AURA can access and where processing happens.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Processing</SectionLabel>
              <SettingRow
                title="Local-first processing"
                description="Prefer local tools and models whenever the requested capability supports it."
                trailing={<Badge tone="ready">Enabled</Badge>}
              />
              <SettingRow
                title="Cloud assistance"
                description="Optional cloud routing will require explicit configuration."
                trailing={<Badge tone="planned">Not configured</Badge>}
              />
              <SettingRow
                title="Screen access"
                description="Vision access will remain permission-based."
                trailing={<Badge tone="planned">M007</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "permissions" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Permissions</h2>
              <p>Rules that will determine what AURA may do on your computer.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel trailing={<Badge tone="ready">Active</Badge>}>
                Action levels
              </SectionLabel>
              <SettingRow
                title="Read"
                description="Inspect allowed local and system information."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.read}
                    onChange={(decision) => void onPermissionChange("read", decision)}
                    label="Read permission"
                  />
                }
              />
              <SettingRow
                title="Act"
                description="Open apps and perform reversible computer actions."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.act}
                    onChange={(decision) => void onPermissionChange("act", decision)}
                    label="Act permission"
                  />
                }
              />
              <SettingRow
                title="Modify"
                description="Change application state, type text or activate controls."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.modify}
                    onChange={(decision) => void onPermissionChange("modify", decision)}
                    label="Modify permission"
                  />
                }
              />
              <SettingRow
                title="Sensitive"
                description="Session or privacy-sensitive actions. Permanent Allow is intentionally unavailable."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.sensitive}
                    onChange={(decision) => void onPermissionChange("sensitive", decision)}
                    allowPermanent={false}
                    label="Sensitive permission"
                  />
                }
              />
              <SettingRow
                title="Destructive"
                description="High-impact actions. Permanent Allow is intentionally unavailable."
                trailing={
                  <PermissionSelect
                    value={permissionPolicy.destructive}
                    onChange={(decision) => void onPermissionChange("destructive", decision)}
                    allowPermanent={false}
                    label="Destructive permission"
                  />
                }
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Policy</SectionLabel>
              <SettingRow
                title="Safe defaults"
                description="Read and Act are allowed; Modify, Sensitive and Destructive require confirmation."
                trailing={
                  <button
                    type="button"
                    className="settings-action-button"
                    onClick={() => void onResetPermissions()}
                  >
                    Reset
                  </button>
                }
              />
              <SettingRow
                title="Storage"
                description="Permission policy is stored locally on this PC."
                trailing={<Badge tone="ready">Local</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "models" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Models</h2>
              <p>Local and optional cloud intelligence used by AURA.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Runtime</SectionLabel>
              <SettingRow
                title="Local language model"
                description="AURA-2 model runtime has not been connected to the desktop generation yet."
                trailing={<Badge tone="planned">Not configured</Badge>}
              />
              <SettingRow
                title="Model Router"
                description="Routes simple actions directly and uses AI only when reasoning is required."
                trailing={<Badge tone="planned">Planned</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "voice" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Voice</h2>
              <p>Speech input, output and wake-word controls.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Voice</SectionLabel>
              <SettingRow title="Push to talk" description="Talk to AURA without opening the main window." trailing={<Badge tone="planned">M006</Badge>} />
              <SettingRow title="Wake word" description="Optional hands-free activation." trailing={<Badge tone="planned">M006</Badge>} />
              <SettingRow title="Voice output" description="Natural spoken responses for actions and status." trailing={<Badge tone="planned">M006</Badge>} />
            </Surface>
          </>
        )}

        {activeSection === "overlay" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Overlay</h2>
              <p>Quick access to AURA from anywhere on Windows.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Behaviour</SectionLabel>
              <SettingRow title="AURA Overlay" description="Compact always-on-top command surface." trailing={<Badge tone="ready">Enabled</Badge>} />
              <SettingRow title="Hide on focus loss" description="Automatically dismiss the Overlay when you return to another app." trailing={<Badge tone="ready">Enabled</Badge>} />
              <SettingRow title="Overlay shortcut" description="Show or hide AURA instantly." trailing={<span className="settings-keys"><ShortcutKey>Ctrl</ShortcutKey><span>+</span><ShortcutKey>Shift</ShortcutKey><span>+</span><ShortcutKey>Space</ShortcutKey></span>} />
            </Surface>
          </>
        )}

        {activeSection === "shortcuts" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Shortcuts</h2>
              <p>Keyboard access to AURA.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>Global shortcuts</SectionLabel>
              <SettingRow
                title="Open AURA Overlay"
                description="Available anywhere while AURA is running."
                trailing={<span className="settings-keys"><ShortcutKey>Ctrl</ShortcutKey><span>+</span><ShortcutKey>Shift</ShortcutKey><span>+</span><ShortcutKey>Space</ShortcutKey></span>}
              />
              <SettingRow
                title="Custom shortcuts"
                description="Shortcut editing and conflict detection arrive later in M002."
                trailing={<Badge tone="planned">Planned</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "integrations" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA SETTINGS</span>
              <h2>Integrations</h2>
              <p>Apps and services AURA can control directly.</p>
            </header>
            <Surface className="settings-card">
              <SectionLabel>OBS Control</SectionLabel>
              <SettingRow
                title="OBS Studio"
                description="Connect AURA directly to the built-in OBS WebSocket server."
                trailing={
                  <Badge
                    tone={
                      obsConnection.connected
                        ? "ready"
                        : obsConnection.lastError
                          ? "warning"
                          : "planned"
                    }
                  >
                    {obsConnection.connected ? "Connected" : "Disconnected"}
                  </Badge>
                }
              />

              <div className="obs-connection-form">
                <label className="settings-field">
                  <span>Host</span>
                  <input
                    value={obsHost}
                    onChange={(event) => setObsHost(event.target.value)}
                    placeholder="127.0.0.1"
                    disabled={obsConnection.connected || obsConnecting}
                  />
                </label>

                <label className="settings-field compact">
                  <span>Port</span>
                  <input
                    inputMode="numeric"
                    value={obsPort}
                    onChange={(event) => setObsPort(event.target.value)}
                    placeholder="4455"
                    disabled={obsConnection.connected || obsConnecting}
                  />
                </label>

                <label className="settings-field">
                  <span>Password</span>
                  <input
                    type="password"
                    value={obsPassword}
                    onChange={(event) => setObsPassword(event.target.value)}
                    placeholder="OBS WebSocket password"
                    autoComplete="off"
                    disabled={obsConnection.connected || obsConnecting}
                  />
                </label>

                <button
                  type="button"
                  className="settings-action-button obs-connect-button"
                  disabled={obsConnecting}
                  onClick={() =>
                    void (
                      obsConnection.connected
                        ? handleObsDisconnect()
                        : handleObsConnect()
                    )
                  }
                >
                  {obsConnecting
                    ? "Working…"
                    : obsConnection.connected
                      ? "Disconnect"
                      : "Connect"}
                </button>
              </div>

              {obsConnection.connected && (
                <div className="obs-connection-meta">
                  <span>
                    OBS {obsConnection.obsStudioVersion ?? "Unknown"}
                  </span>
                  <span>
                    WebSocket {obsConnection.obsWebsocketVersion ?? "Unknown"}
                  </span>
                  <span>
                    RPC {obsConnection.rpcVersion ?? "—"}
                  </span>
                  <span>
                    {obsConnection.host}:{obsConnection.port}
                  </span>
                </div>
              )}

              {obsConnection.lastError && (
                <p className="obs-connection-error">{obsConnection.lastError}</p>
              )}

              {obsConnection.connected && (
                <div className="obs-runtime-panel">
                  <div className="obs-runtime-heading">
                    <div>
                      <strong>Live OBS state</strong>
                      <span>Automatically refreshed every 2 seconds.</span>
                    </div>
                    <button
                      type="button"
                      className="settings-action-button"
                      onClick={() => void onObsRefresh()}
                    >
                      Refresh
                    </button>
                  </div>

                  <div className="obs-runtime-grid">
                    <div className="obs-runtime-item">
                      <span>Streaming</span>
                      <strong
                        className={obsRuntime.streaming ? "active" : ""}
                        title={obsRuntime.streaming ? obsRuntime.streamTimecode : undefined}
                      >
                        {obsRuntime.streaming
                          ? `LIVE · ${formatObsDuration(obsStreamClockMs)}`
                          : "Off"}
                      </strong>
                    </div>
                    <div className="obs-runtime-item">
                      <span>Recording</span>
                      <strong className={obsRuntime.recording ? "active" : ""}>
                        {obsRuntime.recording
                          ? obsRuntime.recordingPaused
                            ? "Paused"
                            : "Recording"
                          : "Off"}
                      </strong>
                    </div>
                    <div className="obs-runtime-item">
                      <span>Studio Mode</span>
                      <strong className={obsRuntime.studioMode ? "active" : ""}>
                        {obsRuntime.studioMode ? "On" : "Off"}
                      </strong>
                    </div>
                    <div className="obs-runtime-item wide">
                      <span>Program Scene</span>
                      <strong>{obsRuntime.currentProgramScene ?? "—"}</strong>
                    </div>
                    <div className="obs-runtime-item wide">
                      <span>Preview Scene</span>
                      <strong>
                        {obsRuntime.studioMode
                          ? obsRuntime.currentPreviewScene ?? "—"
                          : "Studio Mode off"}
                      </strong>
                    </div>
                  </div>

                  <div className="obs-streaming-controls">
                    <div className="obs-recording-copy">
                      <strong>Streaming Control</strong>
                      <span>
                        {obsRuntime.streaming
                          ? `LIVE for ${formatObsDuration(obsStreamClockMs)}.`
                          : "The OBS stream is offline."}
                      </span>
                    </div>

                    <div className="obs-recording-actions">
                      {!obsRuntime.streaming ? (
                        <button
                          type="button"
                          className="settings-action-button obs-streaming-start"
                          disabled={obsStreamingAction !== null}
                          onClick={() => void handleObsStreamingAction("start")}
                        >
                          {obsStreamingAction === "start" ? "Going live…" : "Go Live"}
                        </button>
                      ) : (
                        <button
                          type="button"
                          className="settings-action-button obs-recording-stop"
                          disabled={obsStreamingAction !== null}
                          onClick={() => void handleObsStreamingAction("stop")}
                        >
                          {obsStreamingAction === "stop" ? "Stopping…" : "Stop Stream"}
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="obs-recording-controls">
                    <div className="obs-recording-copy">
                      <strong>Recording Control</strong>
                      <span>
                        {obsRuntime.recording
                          ? obsRuntime.recordingPaused
                            ? "Recording is paused."
                            : "Recording is active."
                          : "Recording is stopped."}
                      </span>
                    </div>

                    <div className="obs-recording-actions">
                      {!obsRuntime.recording && (
                        <button
                          type="button"
                          className="settings-action-button"
                          disabled={obsRecordingAction !== null}
                          onClick={() => void handleObsRecordingAction("start")}
                        >
                          {obsRecordingAction === "start" ? "Starting…" : "Start Recording"}
                        </button>
                      )}

                      {obsRuntime.recording && !obsRuntime.recordingPaused && (
                        <button
                          type="button"
                          className="settings-action-button"
                          disabled={obsRecordingAction !== null}
                          onClick={() => void handleObsRecordingAction("pause")}
                        >
                          {obsRecordingAction === "pause" ? "Pausing…" : "Pause"}
                        </button>
                      )}

                      {obsRuntime.recording && obsRuntime.recordingPaused && (
                        <button
                          type="button"
                          className="settings-action-button"
                          disabled={obsRecordingAction !== null}
                          onClick={() => void handleObsRecordingAction("resume")}
                        >
                          {obsRecordingAction === "resume" ? "Resuming…" : "Resume"}
                        </button>
                      )}

                      {obsRuntime.recording && (
                        <button
                          type="button"
                          className="settings-action-button obs-recording-stop"
                          disabled={obsRecordingAction !== null}
                          onClick={() => void handleObsRecordingAction("stop")}
                        >
                          {obsRecordingAction === "stop" ? "Stopping…" : "Stop Recording"}
                        </button>
                      )}
                    </div>
                  </div>

                  {lastRecordingOutput && (
                    <div className="obs-recording-output">
                      <span>Last recording</span>
                      <strong title={lastRecordingOutput}>{lastRecordingOutput}</strong>
                    </div>
                  )}

                  {obsRuntime.lastError && (
                    <p className="obs-connection-error">{obsRuntime.lastError}</p>
                  )}
                </div>
              )}

              {obsConnection.connected && obsHealth && (
                <div className="obs-health-panel">
                  <div className="obs-runtime-heading">
                    <div>
                      <strong>Production Health</strong>
                      <span>{obsHealth.summary}</span>
                    </div>
                    <div className="obs-health-heading-actions">
                      <Badge
                        tone={
                          obsHealth.status === "good"
                            ? "ready"
                            : obsHealth.status === "warning"
                              ? "warning"
                              : "critical"
                        }
                      >
                        {obsHealth.status.toUpperCase()}
                      </Badge>
                      <button
                        type="button"
                        className="settings-action-button"
                        onClick={() => void onObsHealthRefresh()}
                      >
                        Check now
                      </button>
                    </div>
                  </div>

                  <div className="obs-health-grid">
                    <div className="obs-health-metric">
                      <span>FPS</span>
                      <strong>{obsHealth.activeFps.toFixed(1)}</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>OBS CPU</span>
                      <strong>{obsHealth.cpuUsagePercent.toFixed(1)}%</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Memory</span>
                      <strong>{(obsHealth.memoryUsageMb / 1024).toFixed(2)} GB</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Disk Free</span>
                      <strong>{(obsHealth.availableDiskSpaceMb / 1024).toFixed(1)} GB</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Render Time</span>
                      <strong>{obsHealth.averageFrameRenderTimeMs.toFixed(2)} ms</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Render Skips</span>
                      <strong>{obsHealth.renderSkippedPercent.toFixed(2)}%</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Output Skips</span>
                      <strong>{obsHealth.outputSkippedPercent.toFixed(2)}%</strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Congestion</span>
                      <strong>
                        {obsHealth.streamCongestionPercent == null
                          ? "—"
                          : `${obsHealth.streamCongestionPercent.toFixed(0)}%`}
                      </strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Stream Drops</span>
                      <strong>
                        {obsHealth.streamDroppedPercent == null
                          ? "—"
                          : `${obsHealth.streamDroppedPercent.toFixed(2)}%`}
                      </strong>
                    </div>
                    <div className="obs-health-metric">
                      <span>Bitrate</span>
                      <strong>
                        {obsHealth.streamBitrateKbps == null
                          ? "—"
                          : `${Math.round(obsHealth.streamBitrateKbps)} kbps`}
                      </strong>
                    </div>
                  </div>

                  {obsHealth.issues.length > 0 && (
                    <div className="obs-health-issues">
                      {obsHealth.issues.map((issue) => (
                        <div key={issue}>{issue}</div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {obsConnection.connected && (
                <div className="obs-scenes-panel">
                  <div className="obs-runtime-heading">
                    <div>
                      <strong>Scenes</strong>
                      <span>
                        {obsScenes.scenes.length} scene{obsScenes.scenes.length === 1 ? "" : "s"} detected · refreshed every 5 seconds.
                      </span>
                    </div>
                    <button
                      type="button"
                      className="settings-action-button"
                      onClick={() => void onObsScenesRefresh()}
                    >
                      Refresh scenes
                    </button>
                  </div>

                  {obsScenes.scenes.length > 0 ? (
                    <div className="obs-scene-list">
                      {obsScenes.scenes.map((scene) => (
                        <div className="obs-scene-row" key={scene.uuid}>
                          <div className="obs-scene-index">
                            {String(scene.index + 1).padStart(2, "0")}
                          </div>
                          <div className="obs-scene-copy">
                            <strong>{scene.name}</strong>
                            <span>{scene.uuid}</span>
                          </div>
                          <div className="obs-scene-flags">
                            {scene.isProgram ? (
                              <Badge tone="ready">Program</Badge>
                            ) : (
                              <button
                                type="button"
                                className="settings-action-button obs-scene-action"
                                disabled={obsSceneChanging !== null}
                                onClick={() =>
                                  void handleObsSceneChange("program", scene.uuid)
                                }
                              >
                                {obsSceneChanging === `program:${scene.uuid}`
                                  ? "Taking…"
                                  : "Take Program"}
                              </button>
                            )}

                            {obsRuntime.studioMode && (
                              scene.isPreview ? (
                                <Badge>Preview</Badge>
                              ) : (
                                <button
                                  type="button"
                                  className="settings-action-button obs-scene-action"
                                  disabled={obsSceneChanging !== null}
                                  onClick={() =>
                                    void handleObsSceneChange("preview", scene.uuid)
                                  }
                                >
                                  {obsSceneChanging === `preview:${scene.uuid}`
                                    ? "Setting…"
                                    : "Set Preview"}
                                </button>
                              )
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="obs-scenes-empty">
                      No OBS scenes are available yet.
                    </div>
                  )}

                  {obsScenes.lastError && (
                    <p className="obs-connection-error">{obsScenes.lastError}</p>
                  )}
                </div>
              )}

              {obsConnection.connected && (
                <div className="obs-sources-panel">
                  <div className="obs-runtime-heading">
                    <div>
                      <strong>Program Sources</strong>
                      <span>
                        {obsSources.sceneName
                          ? `${obsSources.items.length} item${obsSources.items.length === 1 ? "" : "s"} in ${obsSources.sceneName} · refreshed every 5 seconds.`
                          : "Waiting for the current Program scene."}
                      </span>
                    </div>
                    <button
                      type="button"
                      className="settings-action-button"
                      disabled={!obsRuntime.currentProgramScene}
                      onClick={() => void onObsSourcesRefresh()}
                    >
                      Refresh sources
                    </button>
                  </div>

                  {obsSources.items.length > 0 ? (
                    <div className="obs-source-list">
                      {obsSources.items.map((source) => (
                        <div
                          className={`obs-source-row ${source.enabled ? "visible" : "hidden"}`}
                          key={`${source.sceneName}:${source.itemId}`}
                        >
                          <div className="obs-source-index">
                            {String(source.index + 1).padStart(2, "0")}
                          </div>

                          <div className="obs-source-copy">
                            <strong>{source.sourceName}</strong>
                            <span>
                              {source.isGroup
                                ? "Group"
                                : source.inputKind ?? "Scene source"}
                              {" · "}item #{source.itemId}
                            </span>
                          </div>

                          <div className="obs-source-actions">
                            <Badge tone={source.enabled ? "ready" : "planned"}>
                              {source.enabled ? "Visible" : "Hidden"}
                            </Badge>
                            <button
                              type="button"
                              className="settings-action-button obs-source-action"
                              disabled={obsSourceChanging !== null}
                              onClick={() =>
                                void handleObsSourceVisibility(
                                  source.sceneName,
                                  source.itemId,
                                  !source.enabled,
                                )
                              }
                            >
                              {obsSourceChanging === source.itemId
                                ? "Updating…"
                                : source.enabled
                                  ? "Hide"
                                  : "Show"}
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="obs-scenes-empty">
                      {obsSources.lastError
                        ? "Could not load the Program scene sources."
                        : obsRuntime.currentProgramScene
                          ? "No source items were found in the Program scene."
                          : "No Program scene is available yet."}
                    </div>
                  )}

                  {obsSources.lastError && (
                    <p className="obs-connection-error">{obsSources.lastError}</p>
                  )}
                </div>
              )}

              {obsConnection.connected && (
                <div className="obs-audio-panel">
                  <div className="obs-runtime-heading">
                    <div>
                      <strong>Audio Inputs</strong>
                      <span>
                        {obsAudio.inputs.length} controllable audio input{obsAudio.inputs.length === 1 ? "" : "s"} · refreshed every 5 seconds.
                      </span>
                    </div>
                    <button
                      type="button"
                      className="settings-action-button"
                      onClick={() => void onObsAudioRefresh()}
                    >
                      Refresh audio
                    </button>
                  </div>

                  {obsAudio.inputs.length > 0 ? (
                    <div className="obs-audio-list">
                      {obsAudio.inputs.map((input) => {
                        const draft =
                          obsAudioDrafts[input.inputUuid] ?? input.volumePercent;
                        const volumeChanged = draft !== input.volumePercent;

                        return (
                          <div className="obs-audio-row" key={input.inputUuid}>
                            <div className="obs-audio-copy">
                              <strong>{input.inputName}</strong>
                              <span>
                                {input.inputKind} · {Number.isFinite(input.volumeDb)
                                  ? `${input.volumeDb.toFixed(1)} dB`
                                  : "dB unavailable"}
                              </span>
                            </div>

                            <div className="obs-audio-level">
                              <input
                                type="range"
                                min="0"
                                max="100"
                                step="1"
                                value={draft}
                                aria-label={`${input.inputName} volume`}
                                disabled={obsAudioChanging !== null}
                                onChange={(event) => {
                                  const percent = Number(event.target.value);
                                  setObsAudioDrafts((current) => ({
                                    ...current,
                                    [input.inputUuid]: percent,
                                  }));
                                }}
                              />
                              <span>{draft}%</span>
                            </div>

                            <div className="obs-audio-actions">
                              <Badge tone={input.muted ? "warning" : "ready"}>
                                {input.muted ? "Muted" : "Live"}
                              </Badge>
                              <button
                                type="button"
                                className="settings-action-button obs-audio-set"
                                disabled={
                                  obsAudioChanging !== null || !volumeChanged
                                }
                                onClick={() =>
                                  void handleObsAudioVolume(
                                    input.inputUuid,
                                    input.volumePercent,
                                  )
                                }
                              >
                                {obsAudioChanging === `volume:${input.inputUuid}`
                                  ? "Setting…"
                                  : "Set"}
                              </button>
                              <button
                                type="button"
                                className="settings-action-button obs-audio-mute"
                                disabled={obsAudioChanging !== null}
                                onClick={() =>
                                  void handleObsAudioMute(
                                    input.inputUuid,
                                    !input.muted,
                                  )
                                }
                              >
                                {obsAudioChanging === `mute:${input.inputUuid}`
                                  ? "Updating…"
                                  : input.muted
                                    ? "Unmute"
                                    : "Mute"}
                              </button>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  ) : (
                    <div className="obs-scenes-empty">
                      {obsAudio.lastError
                        ? "Could not load OBS audio inputs."
                        : "No controllable OBS audio inputs were found."}
                    </div>
                  )}

                  {obsAudio.lastError && (
                    <p className="obs-connection-error">{obsAudio.lastError}</p>
                  )}
                </div>
              )}
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Available & planned</SectionLabel>
              <SettingRow title="Windows" description="Native app, window, input and system controls." trailing={<Badge tone="ready">M003</Badge>} />
              <SettingRow title="Future Skills" description="Modular app integrations built on the AURA Skills architecture." trailing={<Badge tone="planned">Later</Badge>} />
            </Surface>
          </>
        )}
      </div>
    </section>
  );
}

export type { SettingsSection };
