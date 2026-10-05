import { FormEvent, useEffect, useRef, useState } from "react";
import { listenToOpenSettings } from "./bridge/aura";
import { useAuraBridge } from "./bridge/useAuraBridge";
import {
  AuraMark,
  NavItem,
  ShortcutKey,
  StatusPill,
} from "./design-system/components";
import ConfirmationCard from "./ConfirmationCard";
import Settings, { type SettingsSection } from "./Settings";
import Memory from "./Memory";
import Models from "./Models";
import Create from "./Create";
import Computer from "./Computer";
import Tasks from "./Tasks";
import DirectorMode from "./DirectorMode";
import Vision from "./Vision";
import Agents from "./Agents";
import BetaWelcome from "./BetaWelcome";
import DropTray from "./DropTray";
import "./beta.css";
import {
  applyAuraTheme,
  readAuraTheme,
  type AuraTheme,
} from "./theme";

type AppView =
  | "chat"
  | "memory"
  | "models"
  | "create"
  | "computer"
  | "vision"
  | "agents"
  | "tasks"
  | "director"
  | "settings";

function App() {
  const [command, setCommand] = useState("");
  const [attachedDropIds, setAttachedDropIds] = useState<string[]>([]);
  const [view, setView] = useState<AppView>("chat");
  const [settingsSection, setSettingsSection] =
    useState<SettingsSection>("general");
  const [theme, setTheme] = useState<AuraTheme>(() => readAuraTheme());
  const chatEndRef = useRef<HTMLDivElement | null>(null);

  const {
    status,
    activity,
    appStatus,
    betaStatus,
    betaDiagnostics,
    runtimeState,
    permissionPolicy,
    obsConnection,
    obsRuntime,
    obsScenes,
    obsSources,
    obsAudio,
    obsHealth,
    directorPresets,
    directorLastRun,
    memory,
    currentApp,
    appSkillCatalog,
    dropIntake,
    dropHover,
    recentFiles,
    routines,
    routineLastRun,
    projectMemory,
    audioInput,
    voiceCapture,
    speechRuntime,
    ttsRuntime,
    voicePreferences,
    visionRuntime,
    visionHistory,
    visionCapture,
    visionEvent,
    agentPlan,
    agentRuns,
    savedActions,
    automations,
    automationEvent,
    modelCatalog,
    modelRuntimeStatus,
    managedRuntimeStatus,
    imageRuntimeStatus,
    lastGeneratedImage,
    chatMessages,
    pendingConfirmation,
    bridgeError,
    submitCommand,
    refreshBetaStatus,
    updateBetaPreferences,
    refreshBetaDiagnostics,
    exportBetaDiagnosticsControl,
    setPaused,
    setBackgroundMode,
    setAutostart,
    setPermission,
    resetPermissions,
    connectObsControl,
    disconnectObsControl,
    refreshObsRuntime,
    refreshObsScenes,
    refreshObsSources,
    refreshObsAudio,
    refreshObsHealth,
    refreshDirectorPresets,
    refreshMemories,
    clearDropIntakeControl,
    revealDroppedFileControl,
    inspectDroppedFileControl,
    stageDroppedImageForVisionControl,
    refreshRecentFiles,
    refreshRoutines,
    saveRoutineControl,
    deleteRoutineControl,
    runRoutineControl,
    refreshProjectMemory,
    saveProjectMemoryControl,
    deleteProjectMemoryControl,
    setActiveProjectMemoryControl,
    updateVoicePreferences,
    planAgentGoal,
    clearAgentPlan,
    refreshAgentRuns,
    startAgentPlanControl,
    pauseAgentRunControl,
    cancelAgentRunControl,
    refreshSavedActions,
    saveAuraActionControl,
    deleteAuraActionControl,
    runAuraActionControl,
    refreshAutomations,
    saveAutomationControl,
    deleteAutomationControl,
    setAutomationEnabledControl,
    refreshVisionRuntime,
    updateVisionPreferences,
    clearVisionHistoryControl,
    captureVisionScreenControl,
    captureVisionActiveWindowControl,
    captureVisionRegionControl,
    analyzeVisionControl,
    clearVisionCaptureControl,
    stopSpeakingControl,
    refreshTtsRuntime,
    prepareTtsRuntimeControl,
    testTtsVoiceControl,
    refreshAudioInput,
    selectAudioInput,
    startAudioTest,
    stopAudioTest,
    refreshModels,
    refreshManagedRuntime,
    runManagedRuntimeAction,
    refreshImageRuntime,
    generateImageControl,
    clearConversationControl,
    runModelOperation,
    createMemoryControl,
    deleteMemoryControl,
    saveDirectorPresetControl,
    deleteDirectorPresetControl,
    runDirectorPresetControl,
    switchObsProgramScene,
    switchObsPreviewScene,
    controlObsRecording,
    controlObsStreaming,
    controlObsSourceVisibility,
    controlObsAudioMute,
    controlObsAudioVolume,
    approveConfirmation,
    cancelConfirmation,
  } = useAuraBridge();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listenToOpenSettings(() => {
      setSettingsSection("general");
      setView("settings");
    }).then((cleanup) => {
      if (cancelled) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [chatMessages.length, status]);

  useEffect(() => {
    const availableIds = new Set(dropIntake.items.map((item) => item.id));
    setAttachedDropIds((current) =>
      current.filter((dropId) => availableIds.has(dropId)),
    );
  }, [dropIntake.refreshedAtMs]);

  function changeTheme(nextTheme: AuraTheme) {
    setTheme(nextTheme);
    applyAuraTheme(nextTheme);
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const typedValue = command.trim();
    const dropIds = [...attachedDropIds];
    const value =
      typedValue
      || (dropIds.length > 0 ? "Analyze the attached local files." : "");

    if (!value) return;

    setCommand("");
    const ack = await submitCommand(value, "desktop", dropIds);
    if (ack) {
      setAttachedDropIds([]);
    }
  }

  async function runQuickCommand(value: string) {
    if (runtimeState.paused || pendingConfirmation || status === "Working") {
      return;
    }

    await submitCommand(value);
  }

  function openSettings(section: SettingsSection = "general") {
    setSettingsSection(section);
    setView("settings");
  }

  function startNewConversation() {
    setCommand("");
    setAttachedDropIds([]);
    setView("chat");
    void clearConversationControl();
  }

  const runtimeLabel = runtimeState.paused
    ? "AURA paused"
    : status === "Working"
      ? "AURA working"
      : "Computer ready";

  const attachedDropItems = dropIntake.items.filter((item) =>
    attachedDropIds.includes(item.id),
  );
  const attachedTextContextCount = attachedDropItems.filter(
    (item) => item.canPreviewText,
  ).length;
  const attachedImageMetadataCount = attachedDropItems.filter(
    (item) => !item.canPreviewText && item.kind === "image",
  ).length;
  const attachedMetadataOnlyCount =
    attachedDropItems.length
    - attachedTextContextCount
    - attachedImageMetadataCount;

  function toggleDropAttachment(dropId: string) {
    setAttachedDropIds((current) =>
      current.includes(dropId)
        ? current.filter((id) => id !== dropId)
        : [...current, dropId].slice(0, 8),
    );
    setView("chat");
  }

  function attachAllDroppedFiles() {
    setAttachedDropIds(dropIntake.items.map((item) => item.id).slice(0, 8));
    setView("chat");
  }

  return (
    <main className="app-shell aura1-evolved-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <AuraMark compact />
          <div className="brand-copy">
            <strong>AURA-2</strong>
            <span>by Untoz</span>
          </div>
        </div>

        <button
          type="button"
          className="new-conversation-button"
          onClick={startNewConversation}
          disabled={status === "Working" || Boolean(pendingConfirmation)}
        >
          <span aria-hidden="true">＋</span>
          New conversation
        </button>

        <nav className="primary-nav" aria-label="Primary navigation">
          <NavItem
            active={view === "chat"}
            icon="✦"
            onClick={() => setView("chat")}
          >
            Chat
          </NavItem>
          <NavItem
            active={view === "memory"}
            icon="◇"
            onClick={() => setView("memory")}
          >
            Memory
          </NavItem>
          <NavItem
            active={view === "models"}
            icon="◎"
            onClick={() => setView("models")}
          >
            Models
          </NavItem>
          <NavItem
            active={view === "create"}
            icon="◈"
            onClick={() => setView("create")}
          >
            Create
          </NavItem>
          <NavItem
            active={view === "computer"}
            icon="⌁"
            onClick={() => setView("computer")}
          >
            Computer
          </NavItem>
          <NavItem
            active={view === "vision"}
            icon="▣"
            onClick={() => setView("vision")}
          >
            Vision
          </NavItem>
          <NavItem
            active={view === "agents"}
            icon="✧"
            onClick={() => setView("agents")}
          >
            Agents
          </NavItem>
          <NavItem
            active={view === "tasks"}
            icon="↻"
            onClick={() => setView("tasks")}
          >
            Tasks
          </NavItem>
          <NavItem
            active={view === "director"}
            icon="◉"
            onClick={() => setView("director")}
          >
            Director Mode
          </NavItem>
        </nav>

        <div className="sidebar-status-card">
          <div className="sidebar-status-line">
            <span
              className={`sidebar-status-dot ${bridgeError ? "error" : runtimeState.paused ? "paused" : "ready"}`}
            />
            <strong>{runtimeLabel}</strong>
          </div>
          <div className="sidebar-status-details">
            <div>
              <span>Current app</span>
              <strong>{currentApp?.appName ?? "Detecting…"}</strong>
            </div>
            <div>
              <span>Active window</span>
              <strong title={currentApp?.windowTitle}>
                {currentApp?.windowTitle ?? "Unavailable"}
              </strong>
            </div>
            <div>
              <span>Mode</span>
              <strong>
                {appStatus?.localFirst === false ? "Hybrid" : "Local-first"}
              </strong>
            </div>
          </div>
        </div>

        <div className="sidebar-capabilities">
          <span>LOCAL CAPABILITIES</span>
          <div>
            <button type="button" onClick={() => setView("computer")}>
              Applications
            </button>
            <button type="button" onClick={() => setView("computer")}>
              System
            </button>
            <button type="button" onClick={() => setView("memory")}>
              Memory
            </button>
            <button type="button" onClick={() => setView("vision")}>
              Vision
            </button>
            <button type="button" onClick={() => setView("agents")}>
              Agents
            </button>
            <button type="button" onClick={() => setView("director")}>
              OBS
            </button>
            <button type="button" onClick={() => setView("create")}>
              Create
            </button>
          </div>
        </div>

        <div className="sidebar-bottom">
          <NavItem
            active={view === "settings"}
            icon="⚙"
            onClick={() => openSettings()}
          >
            Settings
          </NavItem>

          <p className="sidebar-privacy">
            <strong>AI that lives on your computer.</strong>
            <span>Local-first. Private by design.</span>
          </p>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar" data-tauri-drag-region>
          <div className="topbar-brand">
            <strong>AURA-2</strong>
            <span>Personal computer assistant</span>
          </div>
          <StatusPill status={status} />
        </header>

        {betaStatus.onboardingComplete &&
          betaDiagnostics &&
          (betaDiagnostics.buildLabel === "alpha-v1-testing-preview" ||
            betaDiagnostics.buildLabel === "beta-local-smoke") &&
          view === "chat" && (
            <div className="beta-recovery-banner beta-preview-banner" role="status">
              <div className="beta-recovery-copy">
                <span className="beta-recovery-kicker">
                  {betaDiagnostics.buildLabel === "alpha-v1-testing-preview"
                    ? "AURA-2 TESTING PREVIEW"
                    : "AURA-2 LOCAL TEST BUILD"}
                </span>
                <strong>
                  {betaDiagnostics.appVersion} ·{" "}
                  {betaDiagnostics.buildCommit.slice(0, 12)}
                </strong>
                <span>
                  This is a traceable pre-release build intended for testing.
                  Build source: {betaDiagnostics.buildSource}.
                </span>
              </div>
              <div className="beta-recovery-actions">
                <button
                  type="button"
                  className="feature-secondary-button"
                  onClick={() => openSettings("beta")}
                >
                  Build details
                </button>
              </div>
            </div>
          )}

        {betaStatus.onboardingComplete &&
          betaStatus.previousSessionUnclean &&
          runtimeState.paused &&
          view === "chat" && (
            <div className="beta-recovery-banner" role={betaStatus.crashLoopGuardActive ? "alert" : "status"}>
              <div className="beta-recovery-copy">
                <span className="beta-recovery-kicker">
                  {betaStatus.crashLoopGuardActive
                    ? "CRASH LOOP GUARD"
                    : "RECOVERY SAFE MODE"}
                </span>
                <strong>
                  {betaStatus.crashLoopGuardActive
                    ? "AURA detected repeated unclean sessions"
                    : "AURA recovered after an unclean previous session"}
                </strong>
                <span>
                  {betaStatus.crashLoopGuardActive
                    ? "AURA is paused and background startup has been suppressed. Review Beta & Diagnostics before resuming from Settings."
                    : "Agents and Automations are paused so nothing can run unexpectedly. Review diagnostics if needed, then resume AURA."}
                </span>
              </div>
              <div className="beta-recovery-actions">
                <button
                  type="button"
                  className="feature-secondary-button"
                  onClick={() => openSettings("beta")}
                >
                  Review diagnostics
                </button>
                {!betaStatus.crashLoopGuardActive && (
                  <button
                    type="button"
                    className="feature-primary-button"
                    onClick={() => void setPaused(false)}
                  >
                    Resume AURA
                  </button>
                )}
              </div>
            </div>
          )}

        {betaDiagnostics?.healthStatus === "degraded" && view === "chat" && (
          <div className="beta-recovery-banner beta-health-alert" role="alert">
            <div className="beta-recovery-copy">
              <span className="beta-recovery-kicker">BETA HEALTH CHECK</span>
              <strong>A local AURA subsystem needs attention</strong>
              <span>
                The automatic startup health check found at least one failed
                local check. Nothing was uploaded and unaffected features can
                continue to work.
              </span>
            </div>
            <div className="beta-recovery-actions">
              <button
                type="button"
                className="feature-primary-button"
                onClick={() => openSettings("beta")}
              >
                Review health report
              </button>
            </div>
          </div>
        )}

        <div className={`workspace-content ${view === "chat" ? "chat-workspace" : ""}`}>
          {view === "chat" && (
            <section className="chat-view">
              {chatMessages.length === 0 ? (
                <div className="chat-welcome">
                  <div className={`aura-presence ${status.toLowerCase()}`}>
                    <AuraMark />
                  </div>

                  <span className="chat-eyebrow">AURA-2 BY UNTOZ</span>
                  <h1>How can I help?</h1>
                  <p>AI that lives on your computer.</p>

                  <div className="chat-suggestions">
                    <button
                      type="button"
                      onClick={() => void runQuickCommand("What app am I using?")}
                    >
                      <strong>Computer status</strong>
                      <span>Understand what you are working in</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => void runQuickCommand("Look at my screen. What do you see?")}
                    >
                      <strong>Look at my screen</strong>
                      <span>Use local AURA Vision on an explicit screenshot</span>
                    </button>
                    <button type="button" onClick={() => setView("agents")}>
                      <strong>Plan a task</strong>
                      <span>Build and review a local multi-step Agent plan</span>
                    </button>
                    <button type="button" onClick={() => setView("create")}>
                      <strong>Create with AURA</strong>
                      <span>Images now, video-ready architecture</span>
                    </button>
                    <button type="button" onClick={() => setView("director")}>
                      <strong>Director Mode</strong>
                      <span>OBS production and saved presets</span>
                    </button>
                  </div>

                  <div className="chat-context-strip">
                    <div>
                      <span>Current app</span>
                      <strong>{currentApp?.appName ?? "Detecting…"}</strong>
                    </div>
                    <div>
                      <span>Active window</span>
                      <strong title={currentApp?.windowTitle}>
                        {currentApp?.windowTitle ?? "Unavailable"}
                      </strong>
                    </div>
                    <div>
                      <span>Local model</span>
                      <strong>
                        {modelCatalog.activeModelId
                          ? modelCatalog.models.find(
                              (model) => model.id === modelCatalog.activeModelId,
                            )?.name ?? modelCatalog.activeModelId
                          : "Not selected"}
                      </strong>
                    </div>
                    <div>
                      <span>Activity</span>
                      <strong>{activity}</strong>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="chat-thread">
                  <div className="chat-thread-inner">
                    {chatMessages.map((message) => (
                      <article
                        className={`chat-message ${message.role}`}
                        key={message.id}
                      >
                        <div className="chat-message-author">
                          {message.role === "assistant" ? (
                            <AuraMark compact />
                          ) : (
                            <span className="chat-user-mark">You</span>
                          )}
                        </div>
                        <div className="chat-message-body">
                          <span>
                            {message.role === "assistant" ? "AURA" : "You"}
                          </span>
                          {message.attachmentNames && message.attachmentNames.length > 0 && (
                            <div className="chat-message-attachments">
                              {message.attachmentNames.map((name, index) => (
                                <span key={`${message.id}:attachment:${index}`} title={name}>
                                  <span aria-hidden="true">▤</span>
                                  {name}
                                </span>
                              ))}
                            </div>
                          )}
                          <p>{message.content}</p>
                        </div>
                      </article>
                    ))}

                    {status === "Working" && (
                      <div className="chat-runtime-state">
                        <AuraMark compact />
                        <div>
                          <strong>
                            {modelRuntimeStatus.state === "loading"
                              ? `Loading ${modelCatalog.activeModelId === "aura-1" ? "AURA-1" : "local model"}…`
                              : modelRuntimeStatus.state === "generating"
                                ? "AURA is thinking locally…"
                                : "AURA is working…"}
                          </strong>
                          <span>
                            {modelRuntimeStatus.state === "loading"
                              ? "The first response can take longer while the model is loaded into memory."
                              : activity}
                          </span>
                        </div>
                      </div>
                    )}
                    <div ref={chatEndRef} />
                  </div>
                </div>
              )}

              <div className="chat-composer-area">
                {pendingConfirmation && (
                  <ConfirmationCard
                    confirmation={pendingConfirmation}
                    onAllow={(id) => void approveConfirmation(id)}
                    onCancel={(id) => void cancelConfirmation(id)}
                  />
                )}

                {attachedDropItems.length > 0 && (
                  <div className="composer-attachments" aria-label="Attached local files">
                    <div className="composer-attachments-heading">
                      <span>
                        {attachedDropItems.length} temporary file
                        {attachedDropItems.length === 1 ? "" : "s"} attached
                        {attachedTextContextCount > 0
                          ? ` · ${attachedTextContextCount} text context`
                          : ""}
                        {attachedImageMetadataCount > 0
                          ? ` · ${attachedImageMetadataCount} image metadata`
                          : ""}
                        {attachedMetadataOnlyCount > 0
                          ? ` · ${attachedMetadataOnlyCount} metadata only`
                          : ""}
                      </span>
                      <button
                        type="button"
                        onClick={() => setAttachedDropIds([])}
                        disabled={status === "Working"}
                      >
                        Clear
                      </button>
                    </div>
                    <div className="composer-attachment-chips">
                      {attachedDropItems.map((item) => (
                        <button
                          type="button"
                          className="composer-attachment-chip"
                          key={item.id}
                          title={`Remove ${item.name} from this message`}
                          onClick={() => toggleDropAttachment(item.id)}
                          disabled={status === "Working"}
                        >
                          <span aria-hidden="true">▤</span>
                          <strong>{item.name}</strong>
                          <span aria-hidden="true">×</span>
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                <form className="aura-composer" onSubmit={handleSubmit}>
                  <input
                    autoFocus
                    value={command}
                    onChange={(event) => setCommand(event.target.value)}
                    placeholder={
                      runtimeState.paused
                        ? "AURA is paused…"
                        : attachedDropItems.length > 0
                          ? `Ask AURA about ${attachedDropItems.length} attached file${attachedDropItems.length === 1 ? "" : "s"}…`
                          : "Message AURA…"
                    }
                    aria-label="Message AURA"
                    disabled={
                      status === "Working"
                      || runtimeState.paused
                      || Boolean(pendingConfirmation)
                    }
                  />
                  <button
                    type="submit"
                    aria-label="Send"
                    disabled={
                      (!command.trim() && attachedDropItems.length === 0)
                      || status === "Working"
                      || runtimeState.paused
                      || Boolean(pendingConfirmation)
                    }
                  >
                    ↑
                  </button>
                </form>

                <p className="composer-meta">
                  <span className="local-dot" />
                  Local-first · Private by design · {appStatus?.version ?? "AURA-2"}
                  {attachedDropItems.length > 0 && (
                    <> · Attachments are used for this turn only</>
                  )}
                  <span className="composer-shortcut">
                    <ShortcutKey>Ctrl</ShortcutKey> + <ShortcutKey>Shift</ShortcutKey> +{" "}
                    <ShortcutKey>Space</ShortcutKey>
                  </span>
                </p>
              </div>
            </section>
          )}

          {view === "memory" && (
            <Memory
              memory={memory}
              onRefresh={refreshMemories}
              onCreate={createMemoryControl}
              onDelete={deleteMemoryControl}
              projectMemory={projectMemory}
              routines={routines}
              onProjectRefresh={refreshProjectMemory}
              onProjectSave={saveProjectMemoryControl}
              onProjectDelete={deleteProjectMemoryControl}
              onProjectSetActive={setActiveProjectMemoryControl}
            />
          )}

          {view === "models" && (
            <Models
              catalog={modelCatalog}
              managedRuntime={managedRuntimeStatus}
              onRefresh={refreshModels}
              onOperation={runModelOperation}
              onRuntimeRefresh={refreshManagedRuntime}
              onRuntimeAction={runManagedRuntimeAction}
            />
          )}

          {view === "create" && (
            <Create
              catalog={modelCatalog}
              managedRuntime={managedRuntimeStatus}
              imageRuntime={imageRuntimeStatus}
              lastImage={lastGeneratedImage}
              onModelOperation={runModelOperation}
              onRuntimeAction={runManagedRuntimeAction}
              onRuntimeRefresh={refreshImageRuntime}
              onGenerateImage={generateImageControl}
            />
          )}

          {view === "computer" && (
            <Computer
              currentApp={currentApp}
              appSkillCatalog={appSkillCatalog}
              runtimeState={runtimeState}
              obsConnection={obsConnection}
              obsRuntime={obsRuntime}
              recentFiles={recentFiles}
              onRecentFilesRefresh={refreshRecentFiles}
              onCommand={async (value) => {
                const normalized = value.toLowerCase();
                const isPublishedAppSkill = appSkillCatalog.skills.some(
                  (skill) => skill.command.toLowerCase() === normalized,
                );

                if (
                  value === "Read clipboard" ||
                  normalized.startsWith("find file ") ||
                  normalized.startsWith("latest ") ||
                  normalized.startsWith("recent ") ||
                  normalized.startsWith("último ") ||
                  normalized.startsWith("ultimo ") ||
                  normalized.startsWith("última ") ||
                  normalized.startsWith("ultima ") ||
                  isPublishedAppSkill
                ) {
                  setView("chat");
                }
                await runQuickCommand(value);
              }}
            />
          )}

          {view === "vision" && (
            <Vision
              catalog={modelCatalog}
              managedRuntime={managedRuntimeStatus}
              runtime={visionRuntime}
              history={visionHistory}
              capture={visionCapture}
              event={visionEvent}
              readPermission={permissionPolicy.read}
              onModelOperation={runModelOperation}
              onCaptureScreen={captureVisionScreenControl}
              onCaptureActiveWindow={captureVisionActiveWindowControl}
              onCaptureRegion={captureVisionRegionControl}
              onAnalyze={analyzeVisionControl}
              onPreferencesChange={updateVisionPreferences}
              onHistoryClear={clearVisionHistoryControl}
              onCaptureClear={clearVisionCaptureControl}
              onRuntimeRefresh={refreshVisionRuntime}
            />
          )}

          {view === "agents" && (
            <Agents
              plan={agentPlan}
              runs={agentRuns}
              actions={savedActions}
              automations={automations}
              automationEvent={automationEvent}
              routines={routines}
              directorPresets={directorPresets}
              onPlan={planAgentGoal}
              onPlanClear={clearAgentPlan}
              onRunPlan={startAgentPlanControl}
              onPauseRun={pauseAgentRunControl}
              onCancelRun={cancelAgentRunControl}
              onActionSave={saveAuraActionControl}
              onActionDelete={deleteAuraActionControl}
              onActionRun={runAuraActionControl}
              onAutomationSave={saveAutomationControl}
              onAutomationDelete={deleteAutomationControl}
              onAutomationEnabled={setAutomationEnabledControl}
            />
          )}

          {view === "tasks" && (
            <Tasks
              routines={routines}
              lastRun={routineLastRun}
              onRefresh={refreshRoutines}
              onSave={saveRoutineControl}
              onDelete={deleteRoutineControl}
              onRun={runRoutineControl}
            />
          )}

          {view === "director" && (
            <DirectorMode
              connection={obsConnection}
              runtime={obsRuntime}
              health={obsHealth}
              presets={directorPresets}
              lastRun={directorLastRun}
              onRun={runDirectorPresetControl}
              onOpenSettings={() => openSettings("integrations")}
            />
          )}

          {view === "settings" && (
            <Settings
              activeSection={settingsSection}
              onSectionChange={setSettingsSection}
              appStatus={appStatus}
              runtimeState={runtimeState}
              onPausedChange={setPaused}
              onBackgroundChange={setBackgroundMode}
              onAutostartChange={setAutostart}
              permissionPolicy={permissionPolicy}
              onPermissionChange={setPermission}
              onResetPermissions={resetPermissions}
              obsConnection={obsConnection}
              obsRuntime={obsRuntime}
              obsScenes={obsScenes}
              obsSources={obsSources}
              obsAudio={obsAudio}
              obsHealth={obsHealth}
              directorPresets={directorPresets}
              directorLastRun={directorLastRun}
              onObsConnect={connectObsControl}
              onObsDisconnect={disconnectObsControl}
              onObsRefresh={refreshObsRuntime}
              onObsScenesRefresh={refreshObsScenes}
              onObsSourcesRefresh={refreshObsSources}
              onObsAudioRefresh={refreshObsAudio}
              onObsHealthRefresh={refreshObsHealth}
              onDirectorPresetsRefresh={refreshDirectorPresets}
              onDirectorPresetSave={saveDirectorPresetControl}
              onDirectorPresetDelete={deleteDirectorPresetControl}
              onDirectorPresetRun={runDirectorPresetControl}
              onObsProgramSceneChange={switchObsProgramScene}
              onObsPreviewSceneChange={switchObsPreviewScene}
              onObsRecordingAction={controlObsRecording}
              onObsStreamingAction={controlObsStreaming}
              onObsSourceVisibilityChange={controlObsSourceVisibility}
              onObsAudioMuteChange={controlObsAudioMute}
              onObsAudioVolumeChange={controlObsAudioVolume}
              theme={theme}
              onThemeChange={changeTheme}
              modelCatalog={modelCatalog}
              modelRuntimeStatus={modelRuntimeStatus}
              managedRuntimeStatus={managedRuntimeStatus}
              audioInput={audioInput}
              voiceCapture={voiceCapture}
              speechRuntime={speechRuntime}
              ttsRuntime={ttsRuntime}
              voicePreferences={voicePreferences}
              onVoicePreferencesChange={updateVoicePreferences}
              onStopSpeaking={stopSpeakingControl}
              onTtsRuntimeRefresh={refreshTtsRuntime}
              onTtsRuntimePrepare={prepareTtsRuntimeControl}
              onTtsVoiceTest={testTtsVoiceControl}
              onVoiceModelOperation={runModelOperation}
              onAudioRefresh={refreshAudioInput}
              onAudioSelect={selectAudioInput}
              onAudioTestStart={startAudioTest}
              onAudioTestStop={stopAudioTest}
              betaStatus={betaStatus}
              betaDiagnostics={betaDiagnostics}
              onBetaRefresh={refreshBetaStatus}
              onBetaPreferencesChange={updateBetaPreferences}
              onBetaDiagnosticsRefresh={refreshBetaDiagnostics}
              onBetaDiagnosticsExport={exportBetaDiagnosticsControl}
            />
          )}
        </div>
      </section>

      <DropTray
        snapshot={dropIntake}
        hovering={dropHover}
        paused={runtimeState.paused}
        onClear={clearDropIntakeControl}
        attachedIds={attachedDropIds}
        onToggleAttach={toggleDropAttachment}
        onAttachAll={attachAllDroppedFiles}
        onReveal={revealDroppedFileControl}
        onInspect={inspectDroppedFileControl}
        onAnalyze={async (dropIds) => {
          setView("chat");
          return submitCommand(
            "Analyze the attached local files. Summarize the available content, compare the files when useful, and clearly state when a file is metadata-only.",
            "desktop",
            dropIds,
          );
        }}
        onUseVision={async (dropId) => {
          await stageDroppedImageForVisionControl(dropId);
          setView("vision");
        }}
      />

      {betaStatus.refreshedAtMs > 0 && !betaStatus.onboardingComplete && (
        <BetaWelcome
          status={betaStatus}
          appStatus={appStatus}
          onComplete={async () => {
            await updateBetaPreferences({ onboardingComplete: true });
          }}
          onReviewPermissions={() => {
            void updateBetaPreferences({ onboardingComplete: true }).then(() =>
              openSettings("permissions"),
            );
          }}
        />
      )}
    </main>
  );
}

export default App;
