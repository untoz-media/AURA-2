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
  DirectorPreset,
  DirectorPresetRunResult,
  SaveDirectorPresetRequest,
  ModelCatalog,
  ModelRuntimeStatus,
  ManagedRuntimeStatus,
  AudioInputSnapshot,
  VoiceCaptureEvent,
  SpeechRuntimeStatus,
  TtsRuntimeStatus,
  VoicePreferences,
  BetaStatus,
  DiagnosticsSnapshot,
  SetBetaPreferencesRequest,
} from "./bridge/types";
import { SectionLabel, ShortcutKey, Surface } from "./design-system/components";
import DirectorPresets from "./DirectorPresets";
import { auraThemes, type AuraTheme } from "./theme";

type SettingsSection =
  | "general"
  | "appearance"
  | "privacy"
  | "permissions"
  | "models"
  | "voice"
  | "overlay"
  | "shortcuts"
  | "beta"
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
  directorPresets: DirectorPreset[];
  directorLastRun: DirectorPresetRunResult | null;
  onObsConnect: (request: ObsConnectRequest) => Promise<ObsConnectionState>;
  onObsDisconnect: () => Promise<ObsConnectionState>;
  onObsRefresh: () => Promise<ObsRuntimeState>;
  onObsScenesRefresh: () => Promise<ObsSceneList>;
  onObsSourcesRefresh: () => Promise<ObsSourceItemList>;
  onObsAudioRefresh: () => Promise<ObsAudioInputList>;
  onObsHealthRefresh: () => Promise<ObsProductionHealth | null>;
  onDirectorPresetsRefresh: () => Promise<DirectorPreset[]>;
  onDirectorPresetSave: (
    request: SaveDirectorPresetRequest,
  ) => Promise<DirectorPreset>;
  onDirectorPresetDelete: (presetId: string) => Promise<void>;
  onDirectorPresetRun: (
    presetId: string,
  ) => Promise<DirectorPresetRunResult>;
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
  theme: AuraTheme;
  onThemeChange: (theme: AuraTheme) => void;
  modelCatalog: ModelCatalog;
  modelRuntimeStatus: ModelRuntimeStatus;
  managedRuntimeStatus: ManagedRuntimeStatus;
  audioInput: AudioInputSnapshot;
  voiceCapture: VoiceCaptureEvent | null;
  speechRuntime: SpeechRuntimeStatus;
  ttsRuntime: TtsRuntimeStatus;
  voicePreferences: VoicePreferences;
  onVoicePreferencesChange: (
    preferences: VoicePreferences,
  ) => Promise<VoicePreferences>;
  onStopSpeaking: () => Promise<TtsRuntimeStatus>;
  onTtsRuntimeRefresh: () => Promise<TtsRuntimeStatus>;
  onTtsRuntimePrepare: () => Promise<TtsRuntimeStatus>;
  onTtsVoiceTest: (text?: string) => Promise<TtsRuntimeStatus>;
  onVoiceModelOperation: (
    operation: "download" | "pause" | "resume" | "cancel" | "remove",
    modelId: string,
  ) => Promise<ModelCatalog>;
  onAudioRefresh: () => Promise<AudioInputSnapshot>;
  onAudioSelect: (deviceName?: string) => Promise<AudioInputSnapshot>;
  onAudioTestStart: () => Promise<AudioInputSnapshot>;
  onAudioTestStop: () => Promise<AudioInputSnapshot>;
  betaStatus: BetaStatus;
  betaDiagnostics: DiagnosticsSnapshot | null;
  onBetaRefresh: () => Promise<BetaStatus>;
  onBetaPreferencesChange: (
    request: SetBetaPreferencesRequest,
  ) => Promise<BetaStatus>;
  onBetaDiagnosticsRefresh: () => Promise<DiagnosticsSnapshot>;
  onBetaDiagnosticsExport: () => Promise<string>;
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
  { id: "beta", label: "Beta & Diagnostics", icon: "β" },
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
  directorPresets,
  directorLastRun,
  onObsConnect,
  onObsDisconnect,
  onObsRefresh,
  onObsScenesRefresh,
  onObsSourcesRefresh,
  onObsAudioRefresh,
  onObsHealthRefresh,
  onDirectorPresetsRefresh,
  onDirectorPresetSave,
  onDirectorPresetDelete,
  onDirectorPresetRun,
  onObsProgramSceneChange,
  onObsPreviewSceneChange,
  onObsRecordingAction,
  onObsStreamingAction,
  onObsSourceVisibilityChange,
  onObsAudioMuteChange,
  onObsAudioVolumeChange,
  theme,
  onThemeChange,
  modelCatalog,
  modelRuntimeStatus,
  managedRuntimeStatus,
  audioInput,
  voiceCapture,
  speechRuntime,
  ttsRuntime,
  voicePreferences,
  onVoicePreferencesChange,
  onStopSpeaking,
  onTtsRuntimeRefresh,
  onTtsRuntimePrepare,
  onTtsVoiceTest,
  onVoiceModelOperation,
  onAudioRefresh,
  onAudioSelect,
  onAudioTestStart,
  onAudioTestStop,
  betaStatus,
  betaDiagnostics,
  onBetaRefresh,
  onBetaPreferencesChange,
  onBetaDiagnosticsRefresh,
  onBetaDiagnosticsExport,
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
  const [voiceModelBusy, setVoiceModelBusy] = useState<string | null>(null);
  const [ttsBusy, setTtsBusy] = useState<string | null>(null);
  const [voicePrefsBusy, setVoicePrefsBusy] = useState(false);
  const [wakePhraseDraft, setWakePhraseDraft] = useState(
    voicePreferences.wakePhrase,
  );
  const [betaBusy, setBetaBusy] = useState<string | null>(null);
  const [betaExportPath, setBetaExportPath] = useState<string | null>(null);
  const betaPrivacyInvariant =
    !betaStatus.telemetryEnabled &&
    !betaStatus.automaticCrashUploads &&
    betaStatus.localDiagnosticsOnly;
  const betaSnapshotIdentityValid = betaDiagnostics
    ? betaDiagnostics.schemaVersion === 2 &&
      betaDiagnostics.appName === (appStatus?.name ?? "AURA-2") &&
      betaDiagnostics.appVersion === (appStatus?.version ?? betaDiagnostics.appVersion) &&
      betaDiagnostics.channel === "beta" &&
      !betaDiagnostics.telemetryEnabled
    : null;
  const betaCountersValid = betaDiagnostics
    ? betaDiagnostics.activeAgentRuns <= betaDiagnostics.agentRunsTotal &&
      betaDiagnostics.enabledAutomations <= betaDiagnostics.automations
    : null;
  const betaBackendHealthValid = betaDiagnostics
    ? betaDiagnostics.healthStatus === "healthy" &&
      betaDiagnostics.healthChecks.every((check) => check.status === "passed")
    : null;
  const betaPassedHealthChecks = betaDiagnostics
    ? betaDiagnostics.healthChecks.filter((check) => check.status === "passed").length
    : 0;
  const betaSelfCheckPassed =
    betaDiagnostics !== null &&
    betaPrivacyInvariant &&
    betaSnapshotIdentityValid === true &&
    betaCountersValid === true &&
    betaBackendHealthValid === true;
  const voiceModel = modelCatalog.models.find(
    (model) => model.id === "voice-whisper-base",
  );
  const ttsVoices = modelCatalog.models.filter(
    (model) => model.role === "textToSpeech",
  );
  const ttsModel = ttsVoices.find(
    (model) => model.id === voicePreferences.ttsVoiceId,
  );

  async function runVoiceModel(
    operation: "download" | "pause" | "resume" | "cancel" | "remove",
  ) {
    if (!voiceModel) return;
    if (
      operation === "remove" &&
      !window.confirm("Remove AURA Voice STT from this computer?")
    ) {
      return;
    }

    setVoiceModelBusy(operation);
    try {
      await onVoiceModelOperation(operation, voiceModel.id);
    } finally {
      setVoiceModelBusy(null);
    }
  }

  async function updateVoicePreference(
    patch: Partial<VoicePreferences>,
  ) {
    setVoicePrefsBusy(true);
    try {
      await onVoicePreferencesChange({
        ...voicePreferences,
        ...patch,
      });
    } finally {
      setVoicePrefsBusy(false);
    }
  }

  async function stopSpeakingNow() {
    setTtsBusy("stop");
    try {
      await onStopSpeaking();
    } finally {
      setTtsBusy(null);
    }
  }

  async function runTtsModel(
    operation: "download" | "pause" | "resume" | "cancel" | "remove",
  ) {
    if (!ttsModel) return;
    if (
      operation === "remove" &&
      !window.confirm("Remove the Portuguese AURA Voice TTS model?")
    ) {
      return;
    }

    setTtsBusy(operation);
    try {
      await onVoiceModelOperation(operation, ttsModel.id);
      await onTtsRuntimeRefresh();
    } finally {
      setTtsBusy(null);
    }
  }

  async function prepareLocalTts() {
    setTtsBusy("runtime");
    try {
      await onTtsRuntimePrepare();
    } finally {
      setTtsBusy(null);
    }
  }

  async function testLocalTts() {
    setTtsBusy("test");
    try {
      await onTtsVoiceTest();
    } finally {
      setTtsBusy(null);
    }
  }

  useEffect(() => {
    setWakePhraseDraft(voicePreferences.wakePhrase);
  }, [voicePreferences.wakePhrase]);

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
              <div className="theme-grid">
                {auraThemes.map((option) => (
                  <button
                    type="button"
                    className={`theme-card ${theme === option.id ? "active" : ""}`}
                    key={option.id}
                    aria-pressed={theme === option.id}
                    onClick={() => onThemeChange(option.id)}
                  >
                    <span className={`theme-preview ${option.id}`} aria-hidden="true">
                      <i />
                      <b />
                    </span>
                    <span className="theme-card-copy">
                      <strong>{option.name}</strong>
                      <small>{option.description}</small>
                    </span>
                    {theme === option.id && <Badge tone="ready">Active</Badge>}
                  </button>
                ))}
              </div>
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
                description="Vision capture is permission-based and explicit. Read = Never blocks every Vision capture path."
                trailing={<Badge tone="ready">Permission-gated</Badge>}
              />
              <SettingRow
                title="Usage telemetry"
                description="AURA-2 Beta does not upload product analytics or usage telemetry."
                trailing={<Badge tone="ready">Off</Badge>}
              />
              <SettingRow
                title="Crash uploads"
                description="Unexpected exits are detected locally. Crash reports are not uploaded automatically."
                trailing={<Badge tone="ready">Off</Badge>}
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
                title="Selected local model"
                description={
                  modelCatalog.activeModelId
                    ? `${modelCatalog.models.find((model) => model.id === modelCatalog.activeModelId)?.name ?? modelCatalog.activeModelId} is selected in Model Manager.`
                    : "No local language model is selected yet. Install one from the Models workspace."
                }
                trailing={
                  modelCatalog.activeModelId
                    ? <Badge tone="ready">Selected</Badge>
                    : <Badge tone="planned">None</Badge>
                }
              />
              <SettingRow
                title="Installed models"
                description="Model files are stored in AURA Local Data and verified before installation is finalized."
                trailing={
                  <Badge tone="ready">
                    {modelCatalog.models.filter((model) => model.installed).length} installed
                  </Badge>
                }
              />
              <SettingRow
                title="Managed AI runtime"
                description={
                  managedRuntimeStatus.state === "ready"
                    ? `Private Python ${managedRuntimeStatus.pythonVersion ?? ""} runtime is installed and verified.`
                    : managedRuntimeStatus.state === "notInstalled"
                      ? "AURA can install its own private Python and AI dependencies from the Models workspace."
                      : managedRuntimeStatus.message
                }
                trailing={
                  <Badge tone={managedRuntimeStatus.state === "ready" ? "ready" : "planned"}>
                    {managedRuntimeStatus.state === "ready"
                      ? "Ready"
                      : managedRuntimeStatus.state === "notInstalled"
                        ? "Not installed"
                        : managedRuntimeStatus.state === "error"
                          ? "Error"
                          : "Setup"}
                  </Badge>
                }
              />
              <SettingRow
                title="Managed PyTorch"
                description={
                  managedRuntimeStatus.torchVersion
                    ? `PyTorch ${managedRuntimeStatus.torchVersion} · Transformers ${managedRuntimeStatus.transformersVersion ?? "installed"}`
                    : "Installed together with the managed runtime when requested."
                }
                trailing={
                  <Badge tone={managedRuntimeStatus.torchVersion ? "ready" : "planned"}>
                    {managedRuntimeStatus.torchVersion ? "Installed" : "Pending"}
                  </Badge>
                }
              />
              <SettingRow
                title="Conversation inference"
                description={
                  modelRuntimeStatus.state === "ready"
                    ? `Local inference is ready${modelRuntimeStatus.device ? ` on ${modelRuntimeStatus.device}` : ""}.`
                    : modelRuntimeStatus.state === "loading"
                      ? "The selected local model is loading into the inference runtime."
                      : modelRuntimeStatus.state === "generating"
                        ? "The selected local model is currently generating a response."
                        : modelRuntimeStatus.state === "error"
                          ? modelRuntimeStatus.lastError ?? "The local inference runtime reported an error."
                          : "The runtime starts on demand when a free-form chat message needs the selected model."
                }
                trailing={
                  <Badge tone={modelRuntimeStatus.state === "error" ? "planned" : modelRuntimeStatus.state === "stopped" ? "planned" : "ready"}>
                    {modelRuntimeStatus.state}
                  </Badge>
                }
              />
              <SettingRow
                title="Inference Python"
                description={
                  modelRuntimeStatus.pythonExecutable
                    ? modelRuntimeStatus.pythonExecutable
                    : managedRuntimeStatus.pythonPath
                      ? `Managed runtime ready at ${managedRuntimeStatus.pythonPath}. It will be used on the next free-form prompt.`
                      : "AURA will prefer its managed runtime, then fall back to AURA_PYTHON or a compatible system Python."
                }
                trailing={
                  <Badge tone={modelRuntimeStatus.pythonExecutable || managedRuntimeStatus.pythonPath ? "ready" : "planned"}>
                    {modelRuntimeStatus.pythonExecutable ? "Active" : managedRuntimeStatus.pythonPath ? "Ready" : "Unavailable"}
                  </Badge>
                }
              />
              <SettingRow
                title="GPU acceleration"
                description={
                  modelRuntimeStatus.cuda === true
                    ? "CUDA is available to the local model runtime."
                    : modelRuntimeStatus.cuda === false
                      ? "CUDA is not active; local inference may be much slower and use more system memory."
                      : "GPU capability is reported after the selected model runtime starts."
                }
                trailing={
                  <Badge tone={modelRuntimeStatus.cuda ? "ready" : "planned"}>
                    {modelRuntimeStatus.cuda == null ? "Unknown" : modelRuntimeStatus.cuda ? "CUDA" : "CPU / Auto"}
                  </Badge>
                }
              />
              <SettingRow
                title="Model Router"
                description="Deterministic computer actions stay direct; AI reasoning will only be used when it is actually needed."
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
              <p>Local speech input, spoken replies, conversation mode and wake phrase.</p>
            </header>

            <Surface className="settings-card">
              <SectionLabel>Microphone</SectionLabel>

              <div className="voice-input-panel">
                <label className="voice-device-field">
                  <span>Input device</span>
                  <select
                    value={audioInput.selectedDevice ?? ""}
                    onChange={(event) =>
                      void onAudioSelect(event.target.value || undefined)
                    }
                    disabled={audioInput.testing}
                  >
                    {audioInput.devices.length === 0 && (
                      <option value="">No microphone detected</option>
                    )}
                    {audioInput.devices.map((device) => (
                      <option value={device.name} key={device.name}>
                        {device.name}{device.isDefault ? " · Default" : ""}
                      </option>
                    ))}
                  </select>
                </label>

                <div className="voice-test-actions">
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void onAudioRefresh()}
                    disabled={audioInput.testing}
                  >
                    Refresh
                  </button>
                  <button
                    type="button"
                    className={audioInput.testing ? "feature-secondary-button" : "feature-primary-button"}
                    onClick={() =>
                      void (audioInput.testing ? onAudioTestStop() : onAudioTestStart())
                    }
                    disabled={!audioInput.selectedDevice && audioInput.devices.length === 0}
                  >
                    {audioInput.testing ? "Stop test" : "Test microphone"}
                  </button>
                </div>
              </div>

              <div className="voice-meter-card">
                <div className="voice-meter-heading">
                  <div>
                    <strong>{audioInput.testing ? "Listening locally" : "Input level"}</strong>
                    <span>
                      {audioInput.testing
                        ? "AURA is reading microphone amplitude only. Audio is not stored."
                        : "Start a microphone test to verify the selected input."}
                    </span>
                  </div>
                  <Badge tone={audioInput.testing ? "ready" : "neutral"}>
                    {audioInput.testing ? "Live" : "Idle"}
                  </Badge>
                </div>

                <div className="voice-meter-track" aria-label="Microphone input level">
                  <span style={{ width: `${Math.round(Math.min(1, audioInput.level) * 100)}%` }} />
                </div>

                <div className="voice-stream-meta">
                  <span>{audioInput.sampleRate ? `${audioInput.sampleRate} Hz` : "Sample rate —"}</span>
                  <span>{audioInput.channels ? `${audioInput.channels} ch` : "Channels —"}</span>
                  <span>{audioInput.sampleFormat ?? "Format —"}</span>
                </div>
              </div>

              {audioInput.lastError && (
                <p className="voice-input-error">{audioInput.lastError}</p>
              )}
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Voice behaviour</SectionLabel>

              <SettingRow
                title="Automatic spoken replies"
                description="Speak final responses only when the command originated from Voice."
                trailing={
                  <Toggle
                    checked={voicePreferences.autoSpeak}
                    disabled={voicePrefsBusy}
                    onChange={(checked) =>
                      void updateVoicePreference({ autoSpeak: checked })
                    }
                    label="Automatic spoken replies"
                  />
                }
              />

              <div className="voice-setting-grid">
                <label className="voice-device-field">
                  <span>Speech speed</span>
                  <input
                    type="range"
                    min="0.6"
                    max="1.5"
                    step="0.05"
                    value={voicePreferences.ttsSpeed}
                    disabled={voicePrefsBusy}
                    onChange={(event) =>
                      void updateVoicePreference({
                        ttsSpeed: Number(event.target.value),
                      })
                    }
                  />
                  <small>{voicePreferences.ttsSpeed.toFixed(2)}×</small>
                </label>

                <label className="voice-device-field">
                  <span>Conversation timeout</span>
                  <select
                    value={voicePreferences.conversationTimeoutSeconds}
                    disabled={voicePrefsBusy}
                    onChange={(event) =>
                      void updateVoicePreference({
                        conversationTimeoutSeconds: Number(event.target.value),
                      })
                    }
                  >
                    {[5, 8, 10, 12, 15, 20].map((seconds) => (
                      <option value={seconds} key={seconds}>
                        {seconds}s
                      </option>
                    ))}
                  </select>
                </label>
              </div>

              <SettingRow
                title="Conversation Mode"
                description="After AURA finishes a spoken reply, automatically listen for one follow-up and stop after silence or timeout."
                trailing={
                  <Toggle
                    checked={voicePreferences.conversationMode}
                    disabled={voicePrefsBusy}
                    onChange={(checked) =>
                      void updateVoicePreference({ conversationMode: checked })
                    }
                    label="Conversation Mode"
                  />
                }
              />

              <SettingRow
                title="Wake Phrase (experimental)"
                description="Continuously checks short local audio windows with Whisper. More private than cloud wake-word services, but uses noticeably more CPU/GPU."
                trailing={
                  <Toggle
                    checked={voicePreferences.wakeWordEnabled}
                    disabled={
                      voicePrefsBusy ||
                      voiceModel?.state !== "installed" ||
                      speechRuntime.state === "error"
                    }
                    onChange={(checked) =>
                      void updateVoicePreference({ wakeWordEnabled: checked })
                    }
                    label="Wake Phrase"
                  />
                }
              />

              <label className="voice-device-field voice-wake-phrase">
                <span>Wake phrase</span>
                <input
                  type="text"
                  value={wakePhraseDraft}
                  disabled={voicePrefsBusy || !voicePreferences.wakeWordEnabled}
                  maxLength={32}
                  onChange={(event) => setWakePhraseDraft(event.target.value)}
                  onBlur={() =>
                    void updateVoicePreference({
                      wakePhrase: wakePhraseDraft,
                    })
                  }
                />
                <small>Default: AURA. Wake mode is fully local and off by default.</small>
              </label>
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Voice stack</SectionLabel>
              <SettingRow
                title="Push to talk"
                description={
                  voiceCapture?.phase === "captured"
                    ? `Last capture: ${(voiceCapture.durationMs / 1000).toFixed(1)}s · ${voiceCapture.sampleCount.toLocaleString()} samples. Ready for M006.3 STT.`
                    : "Hold Ctrl + Shift + F8 anywhere in Windows. AURA listens while held and stops when released."
                }
                trailing={
                  audioInput.pushToTalk ? (
                    <Badge tone="ready">Listening</Badge>
                  ) : (
                    <span className="settings-keys">
                      <ShortcutKey>Ctrl</ShortcutKey><span>+</span>
                      <ShortcutKey>Shift</ShortcutKey><span>+</span>
                      <ShortcutKey>F8</ShortcutKey>
                    </span>
                  )
                }
              />
              <div className="voice-stt-panel">
                <div className="voice-stt-heading">
                  <div>
                    <strong>AURA Voice STT</strong>
                    <span>Whisper Base · multilingual · local · Apache-2.0</span>
                  </div>
                  <Badge
                    tone={
                      voiceModel?.state === "installed"
                        ? "ready"
                        : voiceModel?.state === "failed"
                          ? "critical"
                          : "neutral"
                    }
                  >
                    {voiceModel?.state === "installed"
                      ? "Ready"
                      : voiceModel?.state === "downloading"
                        ? "Downloading"
                        : voiceModel?.state === "paused"
                          ? "Paused"
                          : voiceModel?.state === "failed"
                            ? "Error"
                            : "Not installed"}
                  </Badge>
                </div>

                {voiceModel &&
                  (voiceModel.state === "downloading" ||
                    voiceModel.state === "paused") && (
                    <div className="voice-stt-progress">
                      <div className="voice-meter-track">
                        <span
                          style={{
                            width: `${Math.max(
                              0,
                              Math.min(100, voiceModel.progressPercent),
                            )}%`,
                          }}
                        />
                      </div>
                      <span>{voiceModel.progressPercent.toFixed(1)}%</span>
                    </div>
                  )}

                <div className="voice-stt-actions">
                  {voiceModel?.state === "notInstalled" && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      disabled={voiceModelBusy !== null}
                      onClick={() => void runVoiceModel("download")}
                    >
                      Install STT model
                    </button>
                  )}
                  {voiceModel?.state === "downloading" && (
                    <>
                      <button
                        type="button"
                        className="feature-secondary-button"
                        onClick={() => void runVoiceModel("pause")}
                      >
                        Pause
                      </button>
                      <button
                        type="button"
                        className="feature-secondary-button"
                        onClick={() => void runVoiceModel("cancel")}
                      >
                        Cancel
                      </button>
                    </>
                  )}
                  {voiceModel?.state === "paused" && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      onClick={() => void runVoiceModel("resume")}
                    >
                      Resume
                    </button>
                  )}
                  {voiceModel?.state === "installed" && (
                    <button
                      type="button"
                      className="feature-secondary-button"
                      disabled={voiceModelBusy !== null}
                      onClick={() => void runVoiceModel("remove")}
                    >
                      Remove
                    </button>
                  )}
                </div>

                <div className="voice-stream-meta">
                  <span>
                    {speechRuntime.cuda
                      ? speechRuntime.device ?? "CUDA"
                      : speechRuntime.device ?? "CPU / not loaded"}
                  </span>
                  <span>{speechRuntime.state}</span>
                  <span>~295 MB</span>
                </div>

                {(voiceCapture?.phase === "transcribing" ||
                  voiceCapture?.phase === "transcribed" ||
                  voiceCapture?.phase === "submitted") && (
                  <div className="voice-transcript-card">
                    <span>
                      {voiceCapture.phase === "transcribing"
                        ? "TRANSCRIBING"
                        : voiceCapture.phase === "submitted"
                          ? "SENT TO AURA CORE"
                          : "LAST TRANSCRIPTION"}
                    </span>
                    <strong>
                      {voiceCapture.phase === "transcribing"
                        ? "Whisper is processing locally…"
                        : voiceCapture.text || "No speech detected."}
                    </strong>
                  </div>
                )}

                {(speechRuntime.lastError ||
                  (voiceCapture?.phase === "error" ? voiceCapture.message : null)) && (
                  <p className="voice-input-error">
                    {speechRuntime.lastError ||
                      (voiceCapture?.phase === "error" ? voiceCapture.message : "")}
                  </p>
                )}
              </div>
              <div className="voice-stt-panel">
                <div className="voice-stt-heading">
                  <div>
                    <strong>AURA Voice TTS</strong>
                    <span>Piper · Tugão Medium · Português (Portugal) · local</span>
                  </div>
                  <Badge
                    tone={
                      ttsModel?.state === "installed" && ttsRuntime.dependencyReady
                        ? "ready"
                        : ttsModel?.state === "failed" || ttsRuntime.state === "error"
                          ? "critical"
                          : "neutral"
                    }
                  >
                    {ttsRuntime.state === "speaking"
                      ? "Speaking"
                      : ttsModel?.state === "downloading"
                        ? "Downloading"
                        : ttsModel?.state === "paused"
                          ? "Paused"
                          : ttsModel?.state === "installed" && ttsRuntime.dependencyReady
                            ? "Ready"
                            : "Setup required"}
                  </Badge>
                </div>

                <p className="voice-tts-copy">
                  Spoken replies are generated on-device. Automatic speech is used
                  only for commands that originated from Voice.
                </p>

                <label className="voice-device-field voice-tts-select">
                  <span>Voice</span>
                  <select
                    value={voicePreferences.ttsVoiceId}
                    disabled={voicePrefsBusy || ttsBusy !== null}
                    onChange={(event) =>
                      void updateVoicePreference({
                        ttsVoiceId: event.target.value,
                      })
                    }
                  >
                    <option value="voice-piper-ptpt">
                      Tugão · Português (Portugal)
                    </option>
                    <option value="voice-piper-engb-alan">
                      Alan · English (UK)
                    </option>
                  </select>
                </label>

                {ttsModel &&
                  (ttsModel.state === "downloading" ||
                    ttsModel.state === "paused") && (
                    <div className="voice-stt-progress">
                      <div className="voice-meter-track">
                        <span
                          style={{
                            width: `${Math.max(
                              0,
                              Math.min(100, ttsModel.progressPercent),
                            )}%`,
                          }}
                        />
                      </div>
                      <span>{ttsModel.progressPercent.toFixed(1)}%</span>
                    </div>
                  )}

                <div className="voice-stream-meta">
                  <span>
                    {ttsRuntime.dependencyReady
                      ? "Piper runtime ready"
                      : "Piper runtime not installed"}
                  </span>
                  <span>
                    {ttsModel?.state === "installed"
                      ? "PT-PT voice installed"
                      : "~63 MB voice"}
                  </span>
                  <span>{ttsRuntime.sampleRate ? `${ttsRuntime.sampleRate} Hz` : "Sample rate —"}</span>
                </div>

                <div className="voice-stt-actions">
                  {!ttsRuntime.dependencyReady && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      disabled={ttsBusy !== null || managedRuntimeStatus.state !== "ready"}
                      onClick={() => void prepareLocalTts()}
                    >
                      {managedRuntimeStatus.state === "ready"
                        ? "Install Piper runtime"
                        : "Managed runtime required"}
                    </button>
                  )}

                  {ttsModel?.state === "notInstalled" && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      disabled={ttsBusy !== null}
                      onClick={() => void runTtsModel("download")}
                    >
                      Install PT-PT voice
                    </button>
                  )}

                  {ttsModel?.state === "downloading" && (
                    <>
                      <button
                        type="button"
                        className="feature-secondary-button"
                        onClick={() => void runTtsModel("pause")}
                      >
                        Pause
                      </button>
                      <button
                        type="button"
                        className="feature-secondary-button"
                        onClick={() => void runTtsModel("cancel")}
                      >
                        Cancel
                      </button>
                    </>
                  )}

                  {ttsModel?.state === "paused" && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      onClick={() => void runTtsModel("resume")}
                    >
                      Resume
                    </button>
                  )}

                  {ttsModel?.state === "installed" && ttsRuntime.dependencyReady && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      disabled={ttsBusy !== null}
                      onClick={() => void testLocalTts()}
                    >
                      {ttsBusy === "test" ? "Speaking…" : "Test voice"}
                    </button>
                  )}

                  {ttsModel?.state === "installed" &&
                    ttsRuntime.dependencyReady && (
                    <button
                      type="button"
                      className="feature-secondary-button"
                      disabled={ttsBusy === "stop"}
                      onClick={() => void stopSpeakingNow()}
                    >
                      {ttsBusy === "stop" ? "Stopping…" : "Stop speaking"}
                    </button>
                  )}

                  {ttsModel?.state === "installed" && (
                    <button
                      type="button"
                      className="feature-secondary-button"
                      disabled={ttsBusy !== null}
                      onClick={() => void runTtsModel("remove")}
                    >
                      Remove voice
                    </button>
                  )}

                  <button
                    type="button"
                    className="feature-secondary-button"
                    disabled={ttsBusy !== null}
                    onClick={() => void onTtsRuntimeRefresh()}
                  >
                    Refresh
                  </button>
                </div>

                {ttsRuntime.lastText && (
                  <div className="voice-transcript-card">
                    <span>LAST SPOKEN RESPONSE</span>
                    <strong>{ttsRuntime.lastText}</strong>
                  </div>
                )}

                {ttsRuntime.lastError && (
                  <p className="voice-input-error">{ttsRuntime.lastError}</p>
                )}

                <p className="voice-license-note">
                  Voice model: MIT. Piper runtime: GPL-3.0, installed explicitly
                  inside AURA's private managed environment.
                </p>
              </div>

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
                title="Push-to-Talk"
                description="Hold anywhere in Windows to speak to AURA."
                trailing={<span className="settings-keys"><ShortcutKey>Ctrl</ShortcutKey><span>+</span><ShortcutKey>Shift</ShortcutKey><span>+</span><ShortcutKey>F8</ShortcutKey></span>}
              />
              <SettingRow
                title="Select Vision region"
                description="Press once on the first corner and again on the opposite corner."
                trailing={<span className="settings-keys"><ShortcutKey>Ctrl</ShortcutKey><span>+</span><ShortcutKey>Shift</ShortcutKey><span>+</span><ShortcutKey>F9</ShortcutKey></span>}
              />
              <SettingRow
                title="Custom shortcuts"
                description="Shortcut editing and conflict detection arrive later in M002."
                trailing={<Badge tone="planned">Planned</Badge>}
              />
            </Surface>
          </>
        )}

        {activeSection === "beta" && (
          <>
            <header className="settings-header">
              <span className="eyebrow">AURA PUBLIC BETA</span>
              <h2>Beta & Diagnostics</h2>
              <p>
                Local readiness information for troubleshooting the Beta without
                silently sending diagnostic data anywhere.
              </p>
            </header>

            <Surface className="settings-card">
              <SectionLabel trailing={<Badge tone="ready">Local only</Badge>}>
                Beta channel
              </SectionLabel>
              <SettingRow
                title="Release channel"
                description={(appStatus?.name ?? "AURA-2") + " " + (appStatus?.version ?? "") + " · " + (appStatus?.stage ?? "Public Beta")}
                trailing={<Badge tone="ready">Beta</Badge>}
              />
              <SettingRow
                title="Previous session"
                description={
                  betaStatus.previousSessionUnclean
                    ? "The previous AURA session did not record a clean exit. AURA started paused for safety and no report was uploaded."
                    : "The previous session ended cleanly or no recovery condition was detected."
                }
                trailing={
                  <Badge tone={betaStatus.previousSessionUnclean ? "warning" : "ready"}>
                    {betaStatus.previousSessionUnclean ? "Recovered" : "Clean"}
                  </Badge>
                }
              />
              <SettingRow
                title="Telemetry"
                description="No usage analytics are uploaded by this Beta build."
                trailing={<Badge tone="ready">Off</Badge>}
              />
              <SettingRow
                title="Automatic crash uploads"
                description="Crash and diagnostics data stay on this computer unless you manually export a diagnostics file."
                trailing={<Badge tone="ready">Off</Badge>}
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel
                trailing={
                  <Badge
                    tone={
                      betaDiagnostics
                        ? betaSelfCheckPassed
                          ? "ready"
                          : "warning"
                        : "planned"
                    }
                  >
                    {betaDiagnostics
                      ? betaSelfCheckPassed
                        ? "Passed"
                        : "Review"
                      : "Not run"}
                  </Badge>
                }
              >
                Beta self-check
              </SectionLabel>
              <SettingRow
                title="Privacy invariants"
                description="Telemetry must remain off, automatic crash uploads disabled and diagnostics local-only."
                trailing={
                  <Badge tone={betaPrivacyInvariant ? "ready" : "warning"}>
                    {betaPrivacyInvariant ? "Passed" : "Failed"}
                  </Badge>
                }
              />
              <SettingRow
                title="Snapshot identity"
                description="Validates diagnostics schema, product/version identity, Beta channel and the diagnostics telemetry flag."
                trailing={
                  <Badge
                    tone={
                      betaSnapshotIdentityValid === null
                        ? "planned"
                        : betaSnapshotIdentityValid
                          ? "ready"
                          : "warning"
                    }
                  >
                    {betaSnapshotIdentityValid === null
                      ? "Run diagnostics"
                      : betaSnapshotIdentityValid
                        ? "Passed"
                        : "Failed"}
                  </Badge>
                }
              />
              <SettingRow
                title="Runtime counters"
                description="Checks that active Agent runs and enabled Automations cannot exceed their recorded totals."
                trailing={
                  <Badge
                    tone={
                      betaCountersValid === null
                        ? "planned"
                        : betaCountersValid
                          ? "ready"
                          : "warning"
                    }
                  >
                    {betaCountersValid === null
                      ? "Run diagnostics"
                      : betaCountersValid
                        ? "Passed"
                        : "Failed"}
                  </Badge>
                }
              />
              <SettingRow
                title="Subsystem health"
                description="Checks local storage, session state, permission safety floors, model metadata and Agent/Automation stores independently."
                trailing={
                  <Badge
                    tone={
                      betaBackendHealthValid === null
                        ? "planned"
                        : betaBackendHealthValid
                          ? "ready"
                          : "critical"
                    }
                  >
                    {betaBackendHealthValid === null
                      ? "Run diagnostics"
                      : betaBackendHealthValid
                        ? "Passed"
                        : "Degraded"}
                  </Badge>
                }
              />
              <SettingRow
                title="Overall Beta integrity"
                description={
                  betaDiagnostics
                    ? betaSelfCheckPassed
                      ? "Critical local Beta invariants are internally consistent."
                      : "At least one critical Beta invariant needs review before release."
                    : "Generate a diagnostics snapshot to complete the self-check."
                }
                trailing={
                  <Badge
                    tone={
                      betaDiagnostics
                        ? betaSelfCheckPassed
                          ? "ready"
                          : "warning"
                        : "planned"
                    }
                  >
                    {betaDiagnostics
                      ? betaSelfCheckPassed
                        ? "Healthy"
                        : "Needs review"
                      : "Pending"}
                  </Badge>
                }
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Local diagnostics</SectionLabel>
              <SettingRow
                title="Generate diagnostics snapshot"
                description="Collects app/runtime state, installed model IDs and aggregate Agent/Automation counts. It does not include chat messages, prompts, screenshots, memory contents or passwords."
                trailing={
                  <button
                    type="button"
                    className="settings-action-button"
                    disabled={betaBusy !== null}
                    onClick={() => {
                      setBetaBusy("refresh");
                      void onBetaDiagnosticsRefresh().finally(() => setBetaBusy(null));
                    }}
                  >
                    {betaBusy === "refresh" ? "Checking…" : "Refresh"}
                  </button>
                }
              />

              {betaDiagnostics && (
                <div className="beta-diagnostics-grid">
                  <div><span>Platform</span><strong>{betaDiagnostics.platform} · {betaDiagnostics.architecture}</strong></div>
                  <div><span>Managed runtime</span><strong>{betaDiagnostics.managedRuntimeState}</strong></div>
                  <div><span>Create runtime</span><strong>{betaDiagnostics.createImageRuntimeState}</strong></div>
                  <div><span>Installed models</span><strong>{betaDiagnostics.installedModelIds.length}</strong></div>
                  <div><span>Agent runs</span><strong>{betaDiagnostics.activeAgentRuns} active · {betaDiagnostics.agentRunsTotal} recorded</strong></div>
                  <div><span>Saved Actions</span><strong>{betaDiagnostics.savedActions}</strong></div>
                  <div><span>Automations</span><strong>{betaDiagnostics.enabledAutomations}/{betaDiagnostics.automations} enabled</strong></div>
                  <div><span>Health</span><strong>{betaPassedHealthChecks}/{betaDiagnostics.healthChecks.length} checks passed</strong></div>
                </div>
              )}

              {betaDiagnostics && (
                <div className="beta-health-check-list">
                  {betaDiagnostics.healthChecks.map((check) => (
                    <div className="beta-health-check" key={check.id}>
                      <div>
                        <strong>{check.label}</strong>
                        <span>{check.detail}</span>
                      </div>
                      <Badge tone={check.status === "passed" ? "ready" : "critical"}>
                        {check.status === "passed" ? "Passed" : "Failed"}
                      </Badge>
                    </div>
                  ))}
                </div>
              )}

              <SettingRow
                title="Export diagnostics"
                description="Writes a size-bounded, allowlisted JSON diagnostics file to Downloads (or AURA Local Data as fallback). Unexpected fields are blocked by the privacy guard and nothing is uploaded automatically."
                trailing={
                  <button
                    type="button"
                    className="settings-action-button"
                    disabled={betaBusy !== null}
                    onClick={() => {
                      setBetaBusy("export");
                      setBetaExportPath(null);
                      void onBetaDiagnosticsExport()
                        .then((path) => setBetaExportPath(path))
                        .catch(() => setBetaExportPath(null))
                        .finally(() => setBetaBusy(null));
                    }}
                  >
                    {betaBusy === "export" ? "Exporting…" : "Export JSON"}
                  </button>
                }
              />
              {betaExportPath && <p className="beta-diagnostics-path">Saved locally to: {betaExportPath}</p>}
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Beta onboarding</SectionLabel>
              <SettingRow
                title="First-run guide"
                description="Show the Public Beta privacy, permissions and stability introduction again."
                trailing={
                  <button
                    type="button"
                    className="settings-action-button"
                    disabled={betaBusy !== null}
                    onClick={() => {
                      setBetaBusy("onboarding");
                      void onBetaPreferencesChange({ onboardingComplete: false })
                        .finally(() => setBetaBusy(null));
                    }}
                  >
                    Show again
                  </button>
                }
              />
              <SettingRow
                title="Permissions review"
                description="Review Read, Act, Modify, Sensitive and Destructive rules before running Agents."
                trailing={<button type="button" className="settings-action-button" onClick={() => onSectionChange("permissions")}>Review</button>}
              />
              <SettingRow
                title="Refresh Beta status"
                description="Re-read the local Beta session and recovery state."
                trailing={
                  <button
                    type="button"
                    className="settings-action-button"
                    disabled={betaBusy !== null}
                    onClick={() => {
                      setBetaBusy("status");
                      void onBetaRefresh().finally(() => setBetaBusy(null));
                    }}
                  >
                    Refresh
                  </button>
                }
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

              <DirectorPresets
                connected={obsConnection.connected}
                scenes={obsScenes}
                sources={obsSources}
                audio={obsAudio}
                presets={directorPresets}
                lastRun={directorLastRun}
                onRefresh={onDirectorPresetsRefresh}
                onSave={onDirectorPresetSave}
                onDelete={onDirectorPresetDelete}
                onRun={onDirectorPresetRun}
              />
            </Surface>

            <Surface className="settings-card">
              <SectionLabel>Available & planned</SectionLabel>
              <SettingRow title="Windows" description="Native app, window, input and system controls." trailing={<Badge tone="ready">M003</Badge>} />
              <SettingRow title="App Skills" description="Core-owned dynamic Skill Registry. Includes Browser, Notepad, Windows Terminal, Calculator and safe File Explorer skills." trailing={<Badge tone="ready">V3 Beta</Badge>} />
              <SettingRow title="File Context" description="V2 local file intake with opaque IDs, bounded inspection, fair multi-file Chat context, safe Explorer reveal and explicit Vision handoff." trailing={<Badge tone="ready">V2 Beta</Badge>} />
            </Surface>
          </>
        )}
      </div>
    </section>
  );
}

export type { SettingsSection };
