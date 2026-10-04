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
  const [view, setView] = useState<AppView>("chat");
  const [settingsSection, setSettingsSection] =
    useState<SettingsSection>("general");
  const [theme, setTheme] = useState<AuraTheme>(() => readAuraTheme());
  const chatEndRef = useRef<HTMLDivElement | null>(null);

  const {
    status,
    activity,
    appStatus,
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
    chatMessages,
    pendingConfirmation,
    bridgeError,
    submitCommand,
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

  function changeTheme(nextTheme: AuraTheme) {
    setTheme(nextTheme);
    applyAuraTheme(nextTheme);
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const value = command.trim();
    if (!value) return;

    setCommand("");
    await submitCommand(value);
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
    setView("chat");
    void clearConversationControl();
  }

  const activeAssistantModel = modelCatalog.activeModelId
    ? modelCatalog.models.find((model) => model.id === modelCatalog.activeModelId)
    : undefined;
  const managedRuntimeReady = managedRuntimeStatus.state === "ready";
  const localAssistantReady = Boolean(activeAssistantModel?.installed);
  const localSetupReady = managedRuntimeReady && localAssistantReady;

  const runtimeLabel = runtimeState.paused
    ? "AURA paused"
    : status === "Working"
      ? "AURA working"
      : "Computer ready";

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

                  {!localSetupReady && (
                    <div className="beta-setup-card">
                      <div className="beta-setup-heading">
                        <div>
                          <span className="chat-eyebrow">FIRST LOCAL SETUP</span>
                          <strong>Finish setting up AURA on this PC</strong>
                          <p>
                            Computer Control can work without a language model, but local Chat,
                            Agents and AI understanding need the AURA Runtime and an installed
                            assistant model.
                          </p>
                        </div>
                        <span className="beta-setup-progress">
                          {[managedRuntimeReady, localAssistantReady].filter(Boolean).length}/2 ready
                        </span>
                      </div>

                      <div className="beta-setup-steps">
                        <button type="button" onClick={() => setView("models")}>
                          <span className={managedRuntimeReady ? "complete" : ""}>
                            {managedRuntimeReady ? "✓" : "1"}
                          </span>
                          <div>
                            <strong>Install AURA Runtime</strong>
                            <small>
                              {managedRuntimeReady
                                ? `Ready · Python ${managedRuntimeStatus.pythonVersion ?? "installed"}`
                                : managedRuntimeStatus.state === "error"
                                  ? "Runtime needs attention"
                                  : "Private Python + local AI dependencies"}
                            </small>
                          </div>
                        </button>

                        <button type="button" onClick={() => setView("models")}>
                          <span className={localAssistantReady ? "complete" : ""}>
                            {localAssistantReady ? "✓" : "2"}
                          </span>
                          <div>
                            <strong>Install & select an assistant</strong>
                            <small>
                              {localAssistantReady
                                ? `${activeAssistantModel?.name ?? "Local model"} selected`
                                : "AURA-1 is currently the available assistant profile"}
                            </small>
                          </div>
                        </button>
                      </div>

                      <div className="beta-setup-actions">
                        <button type="button" onClick={() => setView("models")}>
                          Open Models
                        </button>
                        <button type="button" onClick={() => openSettings("permissions")}>
                          Review permissions
                        </button>
                        <button type="button" onClick={() => openSettings("diagnostics")}>
                          Diagnostics
                        </button>
                      </div>
                    </div>
                  )}

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
                        {activeAssistantModel?.name ?? "Not selected"}
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

                <form className="aura-composer" onSubmit={handleSubmit}>
                  <input
                    autoFocus
                    value={command}
                    onChange={(event) => setCommand(event.target.value)}
                    placeholder={
                      runtimeState.paused
                        ? "AURA is paused…"
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
                      !command.trim()
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

          {view === "create" && <Create />}

          {view === "computer" && (
            <Computer
              currentApp={currentApp}
              runtimeState={runtimeState}
              obsConnection={obsConnection}
              obsRuntime={obsRuntime}
              recentFiles={recentFiles}
              onRecentFilesRefresh={refreshRecentFiles}
              onCommand={runQuickCommand}
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
              visionRuntime={visionRuntime}
              agentRuns={agentRuns}
              automations={automations}
            />
          )}
        </div>
      </section>
    </main>
  );
}

export default App;
