import { useCallback, useEffect, useState } from "react";
import {
  connectObs,
  disconnectObs,
  getAppStatus,
  getObsConnectionState,
  getObsRuntimeState,
  getObsScenes,
  getObsSourceItems,
  getObsAudioInputs,
  getObsProductionHealth,
  getDirectorPresets,
  getMemories,
  getCurrentAppContext,
  getRecentFilesContext,
  getModelCatalog,
  getModelRuntimeStatus,
  getManagedRuntimeStatus,
  getPermissionPolicy,
  getRuntimeState,
  listenToAuraCore,
  listenToLifecycle,
  listenToModelDownloads,
  listenToManagedRuntime,
  listenToVoiceCapture,
  listenToVision,
  listenToAgent,
  listenToAutomation,
  createAgentPlan,
  getAgentRuns,
  startAgentPlan,
  pauseAgentRun,
  cancelAgentRun,
  getSavedAuraActions,
  saveAuraAction,
  deleteAuraAction,
  runAuraAction,
  getAuraAutomations,
  saveAuraAutomation,
  deleteAuraAutomation,
  setAuraAutomationEnabled,
  getVisionRuntimeStatus,
  getVisionHistory,
  setVisionPreferences,
  clearVisionHistory,
  getLastVisionCapture,
  clearLastVisionCapture,
  captureVisionScreen,
  captureVisionActiveWindow,
  captureVisionRegion,
  analyzeLastVisionCapture,
  resetPermissionPolicy,
  resolveConfirmation,
  setAutostartEnabled,
  setBackgroundEnabled,
  setPermissionDecision,
  setRuntimePaused,
  setObsProgramScene,
  setObsPreviewScene,
  startObsRecording,
  stopObsRecording,
  pauseObsRecording,
  resumeObsRecording,
  startObsStreaming,
  stopObsStreaming,
  setObsSourceVisibility,
  setObsAudioMuted,
  setObsAudioVolume,
  saveDirectorPreset,
  deleteDirectorPreset,
  runDirectorPreset,
  createMemory,
  deleteMemory,
  getUserRoutines,
  saveUserRoutine,
  deleteUserRoutine,
  runUserRoutine,
  getProjectMemory,
  saveProjectMemory,
  deleteProjectMemory,
  setActiveProjectMemory,
  getAudioInputState,
  getSpeechRuntimeStatus,
  getTtsRuntimeStatus,
  getVoicePreferences,
  setVoicePreferences,
  prepareTtsRuntime,
  stopTtsSpeaking,
  testTtsVoice,
  selectAudioInputDevice,
  startAudioInputTest,
  stopAudioInputTest,
  startModelDownload,
  pauseModelDownload,
  resumeModelDownload,
  cancelModelDownload,
  setActiveModel,
  removeModel,
  installManagedRuntime,
  repairManagedRuntime,
  removeManagedRuntime,
  clearModelConversation,
  submitAuraCommand,
} from "./aura";
import type {
  AppStatus,
  AuraStatus,
  CoreError,
  CoreEvent,
  LifecycleEvent,
  PermissionClass,
  PermissionDecision,
  PermissionPolicy,
  PendingConfirmation,
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
  SaveDirectorPresetRequest,
  DirectorPresetRunResult,
  MemorySnapshot,
  MemoryCreateResult,
  CurrentAppInfo,
  RecentFilesSnapshot,
  ModelCatalog,
  ModelDownloadProgress,
  ModelRuntimeStatus,
  ChatMessage,
  ManagedRuntimeStatus,
  UserRoutine,
  SaveRoutineRequest,
  RoutineRunResult,
  ProjectMemorySnapshot,
  SaveProjectRequest,
  ProjectMemory,
  AudioInputSnapshot,
  VoiceCaptureEvent,
  SpeechRuntimeStatus,
  TtsRuntimeStatus,
  VoicePreferences,
  VisionCapture,
  VisionEvent,
  VisionRuntimeStatus,
  VisionPreferences,
  VisionHistorySnapshot,
  VisionAnalysisPayload,
  VisionRegionRequest,
  AgentPlan,
  AgentRun,
  AgentSnapshot,
  AgentEvent,
  SavedAuraAction,
  SaveAuraActionRequest,
  AuraAutomation,
  SaveAutomationRequest,
  AutomationEvent,
} from "./types";

const DEFAULT_ACTIVITY =
  "Desktop foundation online. AURA Core and background runtime are ready.";

const DEFAULT_OBS_RUNTIME: ObsRuntimeState = {
  available: false,
  streaming: false,
  recording: false,
  recordingPaused: false,
  studioMode: false,
  streamDurationMs: 0,
  streamTimecode: "00:00:00.000",
  refreshedAtMs: 0,
};

const DEFAULT_OBS_SCENES: ObsSceneList = {
  scenes: [],
  refreshedAtMs: 0,
};

const DEFAULT_OBS_SOURCES: ObsSourceItemList = {
  sceneName: "",
  items: [],
  refreshedAtMs: 0,
};

const DEFAULT_OBS_AUDIO: ObsAudioInputList = {
  inputs: [],
  refreshedAtMs: 0,
};

const DEFAULT_MEMORY: MemorySnapshot = {
  records: [],
  refreshedAtMs: 0,
};

const DEFAULT_RECENT_FILES: RecentFilesSnapshot = {
  items: [],
  source: "windowsRecentItems",
  refreshedAtMs: 0,
};

const DEFAULT_PROJECT_MEMORY: ProjectMemorySnapshot = {
  projects: [],
  refreshedAtMs: 0,
};

const DEFAULT_AUDIO_INPUT: AudioInputSnapshot = {
  devices: [],
  testing: false,
  pushToTalk: false,
  level: 0,
  capturedSamples: 0,
  captureDurationMs: 0,
};

const DEFAULT_SPEECH_RUNTIME: SpeechRuntimeStatus = {
  state: "stopped",
  modelId: "voice-whisper-base",
  refreshedAtMs: 0,
};

const DEFAULT_TTS_RUNTIME: TtsRuntimeStatus = {
  state: "stopped",
  modelId: "voice-piper-ptpt",
  dependencyReady: false,
  voiceInstalled: false,
  refreshedAtMs: 0,
};

const DEFAULT_VOICE_PREFERENCES: VoicePreferences = {
  autoSpeak: true,
  ttsSpeed: 1,
  conversationMode: false,
  conversationTimeoutSeconds: 8,
  wakeWordEnabled: false,
  wakePhrase: "AURA",
  ttsVoiceId: "voice-piper-ptpt",
};

const DEFAULT_VISION_RUNTIME: VisionRuntimeStatus = {
  state: "stopped",
  modelId: "vision-smolvlm2-500m",
  refreshedAtMs: 0,
};

const DEFAULT_VISION_HISTORY: VisionHistorySnapshot = {
  preferences: {
    historyEnabled: false,
    retainImages: false,
    maxHistory: 20,
  },
  items: [],
  refreshedAtMs: 0,
};

const DEFAULT_AGENT_SNAPSHOT: AgentSnapshot = {
  runs: [],
  refreshedAtMs: 0,
};

const DEFAULT_MODEL_CATALOG: ModelCatalog = {
  models: [],
  modelsRoot: "",
  refreshedAtMs: 0,
};

const DEFAULT_MODEL_RUNTIME: ModelRuntimeStatus = {
  state: "stopped",
  refreshedAtMs: 0,
};

const DEFAULT_MANAGED_RUNTIME: ManagedRuntimeStatus = {
  state: "notInstalled",
  progressPercent: 0,
  message: "Managed AI runtime is not installed.",
  updatedAtMs: 0,
};

export function useAuraBridge() {
  const [status, setStatus] = useState<AuraStatus>("Idle");
  const [activity, setActivity] = useState(DEFAULT_ACTIVITY);
  const [appStatus, setAppStatus] = useState<AppStatus | null>(null);
  const [runtimeState, setRuntimeState] = useState<RuntimeState>({
    paused: false,
    backgroundEnabled: true,
    autostartEnabled: false,
  });
  const [obsConnection, setObsConnection] = useState<ObsConnectionState>({
    connected: false,
    host: "127.0.0.1",
    port: 4455,
  });
  const [obsRuntime, setObsRuntime] = useState<ObsRuntimeState>(DEFAULT_OBS_RUNTIME);
  const [obsScenes, setObsScenes] = useState<ObsSceneList>(DEFAULT_OBS_SCENES);
  const [obsSources, setObsSources] = useState<ObsSourceItemList>(DEFAULT_OBS_SOURCES);
  const [obsAudio, setObsAudio] = useState<ObsAudioInputList>(DEFAULT_OBS_AUDIO);
  const [obsHealth, setObsHealth] = useState<ObsProductionHealth | null>(null);
  const [directorPresets, setDirectorPresets] = useState<DirectorPreset[]>([]);
  const [directorLastRun, setDirectorLastRun] =
    useState<DirectorPresetRunResult | null>(null);
  const [memory, setMemory] = useState<MemorySnapshot>(DEFAULT_MEMORY);
  const [currentApp, setCurrentApp] = useState<CurrentAppInfo | null>(null);
  const [recentFiles, setRecentFiles] =
    useState<RecentFilesSnapshot>(DEFAULT_RECENT_FILES);
  const [modelCatalog, setModelCatalog] =
    useState<ModelCatalog>(DEFAULT_MODEL_CATALOG);
  const [modelRuntimeStatus, setModelRuntimeStatus] =
    useState<ModelRuntimeStatus>(DEFAULT_MODEL_RUNTIME);
  const [managedRuntimeStatus, setManagedRuntimeStatus] =
    useState<ManagedRuntimeStatus>(DEFAULT_MANAGED_RUNTIME);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [routines, setRoutines] = useState<UserRoutine[]>([]);
  const [routineLastRun, setRoutineLastRun] =
    useState<RoutineRunResult | null>(null);
  const [projectMemory, setProjectMemory] =
    useState<ProjectMemorySnapshot>(DEFAULT_PROJECT_MEMORY);
  const [audioInput, setAudioInput] =
    useState<AudioInputSnapshot>(DEFAULT_AUDIO_INPUT);
  const [voiceCapture, setVoiceCapture] =
    useState<VoiceCaptureEvent | null>(null);
  const [speechRuntime, setSpeechRuntime] =
    useState<SpeechRuntimeStatus>(DEFAULT_SPEECH_RUNTIME);
  const [ttsRuntime, setTtsRuntime] =
    useState<TtsRuntimeStatus>(DEFAULT_TTS_RUNTIME);
  const [voicePreferences, setVoicePreferencesState] =
    useState<VoicePreferences>(DEFAULT_VOICE_PREFERENCES);
  const [visionRuntime, setVisionRuntime] =
    useState<VisionRuntimeStatus>(DEFAULT_VISION_RUNTIME);
  const [visionHistory, setVisionHistory] =
    useState<VisionHistorySnapshot>(DEFAULT_VISION_HISTORY);
  const [visionCapture, setVisionCapture] =
    useState<VisionCapture | null>(null);
  const [visionEvent, setVisionEvent] =
    useState<VisionEvent | null>(null);
  const [agentPlan, setAgentPlan] = useState<AgentPlan | null>(null);
  const [agentRuns, setAgentRuns] =
    useState<AgentSnapshot>(DEFAULT_AGENT_SNAPSHOT);
  const [savedActions, setSavedActions] = useState<SavedAuraAction[]>([]);
  const [automations, setAutomations] = useState<AuraAutomation[]>([]);
  const [automationEvent, setAutomationEvent] =
    useState<AutomationEvent | null>(null);
  const [permissionPolicy, setPermissionPolicyState] = useState<PermissionPolicy>({
    read: "allow",
    act: "allow",
    modify: "ask",
    destructive: "ask",
    sensitive: "ask",
  });
  const [pendingConfirmation, setPendingConfirmation] =
    useState<PendingConfirmation | null>(null);
  const [bridgeError, setBridgeError] = useState<CoreError | null>(null);

  useEffect(() => {
    let cancelled = false;
    let cleanupCore: (() => void) | undefined;
    let cleanupLifecycle: (() => void) | undefined;
    let cleanupModels: (() => void) | undefined;
    let cleanupManagedRuntime: (() => void) | undefined;
    let cleanupVoiceCapture: (() => void) | undefined;
    let cleanupVision: (() => void) | undefined;
    let cleanupAgent: (() => void) | undefined;
    let cleanupAutomation: (() => void) | undefined;

    Promise.all([
      getAppStatus(),
      getRuntimeState(),
      getPermissionPolicy(),
      getObsConnectionState(),
      getDirectorPresets(),
    ])
      .then(([app, runtime, permissions, obs, presets]) => {
        if (cancelled) return;
        setAppStatus(app);
        setRuntimeState(runtime);
        setPermissionPolicyState(permissions);
        setObsConnection(obs);
        setDirectorPresets(presets);

        if (runtime.paused) {
          setActivity("AURA is paused. Resume it from the system tray or settings.");
        } else if (runtime.backgroundEnabled) {
          setActivity("AURA is ready and can remain available in the background.");
        } else {
          setActivity("Background mode is disabled. Closing AURA will quit the app.");
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setBridgeError({
            code: "bridge.status_failed",
            message: String(error),
          });
        }
      });

    getRecentFilesContext()
      .then((snapshot) => {
        if (!cancelled) setRecentFiles(snapshot);
      })
      .catch(() => {
        // Windows Recent Items can legitimately be unavailable or empty.
      });

    getMemories()
      .then((snapshot) => {
        if (!cancelled) setMemory(snapshot);
      })
      .catch((error) => {
        if (!cancelled) {
          const message = String(error);
          setBridgeError({
            code: "memory.load_failed",
            message,
          });
          setActivity(message);
        }
      });


    getSpeechRuntimeStatus()
      .then((runtime) => {
        if (!cancelled) setSpeechRuntime(runtime);
      })
      .catch(() => {
        // Speech runtime remains stopped until Voice STT is used.
      });

    getTtsRuntimeStatus()
      .then((runtime) => {
        if (!cancelled) setTtsRuntime(runtime);
      })
      .catch(() => {
        // TTS remains unavailable until explicitly prepared.
      });

    getVoicePreferences()
      .then((preferences) => {
        if (!cancelled) setVoicePreferencesState(preferences);
      })
      .catch(() => {
        // Defaults remain active until preferences can be read.
      });

    getVisionRuntimeStatus()
      .then((runtime) => {
        if (!cancelled) setVisionRuntime(runtime);
      })
      .catch(() => {
        // Vision remains stopped until its local model is used.
      });

    getVisionHistory()
      .then((snapshot) => {
        if (!cancelled) setVisionHistory(snapshot);
      })
      .catch(() => {
        // Vision history is optional and disabled by default.
      });

    getLastVisionCapture()
      .then((capture) => {
        if (!cancelled) setVisionCapture(capture);
      })
      .catch(() => {
        // No current screenshot is a supported state.
      });

    getAgentRuns()
      .then((snapshot) => {
        if (!cancelled) setAgentRuns(snapshot);
      })
      .catch(() => {
        // Agent history starts empty.
      });

    getSavedAuraActions()
      .then((items) => {
        if (!cancelled) setSavedActions(items);
      })
      .catch(() => {
        // Saved Actions are optional.
      });

    getAuraAutomations()
      .then((items) => {
        if (!cancelled) setAutomations(items);
      })
      .catch(() => {
        // Automations are optional.
      });

    getAudioInputState()
      .then((snapshot) => {
        if (!cancelled) setAudioInput(snapshot);
      })
      .catch(() => {
        // Microphone access can be unavailable or blocked by Windows privacy settings.
      });

    getProjectMemory()
      .then((snapshot) => {
        if (!cancelled) setProjectMemory(snapshot);
      })
      .catch(() => {
        // No project memory is a supported state.
      });

    getUserRoutines()
      .then((items) => {
        if (!cancelled) setRoutines(items);
      })
      .catch(() => {
        // Empty/unavailable routines are a supported state.
      });

    getModelRuntimeStatus()
      .then((runtime) => {
        if (!cancelled) setModelRuntimeStatus(runtime);
      })
      .catch(() => {
        // Runtime remains stopped until a selected model is used.
      });

    getManagedRuntimeStatus()
      .then((runtime) => {
        if (!cancelled) setManagedRuntimeStatus(runtime);
      })
      .catch(() => {
        // A missing managed runtime is a supported Alpha state.
      });

    getModelCatalog()
      .then((catalog) => {
        if (!cancelled) setModelCatalog(catalog);
      })
      .catch((error) => {
        if (!cancelled) {
          const message = String(error);
          setBridgeError({
            code: "models.load_failed",
            message,
          });
          setActivity(message);
        }
      });

    listenToModelDownloads((progress: ModelDownloadProgress) => {
      if (cancelled) return;

      setModelCatalog((current) => ({
        ...current,
        refreshedAtMs: progress.updatedAtMs,
        models: current.models.map((model) =>
          model.id === progress.modelId
            ? {
                ...model,
                state: progress.state,
                bytesDownloaded: progress.bytesDownloaded,
                totalBytes: progress.totalBytes ?? model.totalBytes,
                progressPercent: progress.progressPercent,
                bytesPerSecond: progress.bytesPerSecond,
                currentFile: progress.currentFile,
                error: progress.error,
                installed: progress.state === "installed" ? true : model.installed,
              }
            : model,
        ),
      }));

      if (progress.state === "installed") {
        setActivity(`${progress.modelId === "aura-1" ? "AURA-1" : progress.modelId} installed and verified.`);
      } else if (progress.state === "failed") {
        setActivity(progress.error ?? "Model download failed.");
      } else if (progress.state === "notInstalled") {
        setActivity("Model download cancelled.");
      }

      if (
        progress.state === "installed"
        || progress.state === "notInstalled"
        || progress.state === "failed"
      ) {
        void getModelCatalog()
          .then((catalog) => {
            if (!cancelled) setModelCatalog(catalog);
          })
          .catch(() => {
            // The event already carries the terminal state.
          });
      }
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanupModels = unlisten;
        }
      })
      .catch(() => {
        // Model download events are supplementary to explicit catalog refreshes.
      });

    listenToVoiceCapture((event: VoiceCaptureEvent) => {
      if (cancelled) return;
      setVoiceCapture(event);

      if (
        event.phase === "listening" ||
        event.phase === "conversationListening"
      ) {
        setStatus("Listening");
      } else if (
        event.phase === "transcribing" ||
        event.phase === "transcribed" ||
        event.phase === "wakeDetected"
      ) {
        setStatus("Thinking");
      } else if (event.phase === "submitted") {
        // AURA Core owns status from this point onward.
      } else {
        setStatus("Idle");
      }

      setActivity(event.message);

      if (event.phase === "submitted" && event.text) {
        const messageId = `voice:${event.timestampMs}:user`;
        setChatMessages((current) =>
          current.some((message) => message.id === messageId)
            ? current
            : [
                ...current,
                {
                  id: messageId,
                  role: "user",
                  content: event.text!,
                  timestampMs: event.timestampMs,
                },
              ],
        );
      }

      if (
        event.phase === "transcribed" ||
        event.phase === "submitted" ||
        event.phase === "error"
      ) {
        void getSpeechRuntimeStatus()
          .then((runtime) => {
            if (!cancelled) setSpeechRuntime(runtime);
          })
          .catch(() => undefined);
      }

      void getAudioInputState()
        .then((snapshot) => {
          if (!cancelled) setAudioInput(snapshot);
        })
        .catch(() => undefined);
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else cleanupVoiceCapture = unlisten;
      })
      .catch(() => {
        // Voice capture events are supplementary to the microphone state.
      });

    listenToVision((event: VisionEvent) => {
      if (cancelled) return;

      setVisionEvent(event);
      setActivity(event.message);

      if (
        event.capture &&
        !["completed", "historyError", "error"].includes(event.phase)
      ) {
        setVisionCapture(event.capture);
      }

      if (event.phase === "completed") {
        setVisionCapture(null);
      }

      if (event.phase === "capturing" || event.phase === "analyzing") {
        setStatus("Working");
      } else if (
        event.phase === "completed" ||
        event.phase === "error" ||
        event.phase === "historyError"
      ) {
        setStatus("Idle");
      }

      if (
        event.phase === "completed" ||
        event.phase === "historyError" ||
        event.phase === "error"
      ) {
        void getVisionRuntimeStatus()
          .then((runtime) => {
            if (!cancelled) setVisionRuntime(runtime);
          })
          .catch(() => undefined);

        void getVisionHistory()
          .then((snapshot) => {
            if (!cancelled) setVisionHistory(snapshot);
          })
          .catch(() => undefined);
      }
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else cleanupVision = unlisten;
      })
      .catch(() => {
        // Vision events supplement explicit capture/analysis results.
      });

    listenToAgent((event: AgentEvent) => {
      if (cancelled) return;
      setActivity(event.message);
      setAgentRuns((current) => {
        const runs = [
          event.run,
          ...current.runs.filter((run) => run.id !== event.run.id),
        ].sort((left, right) => right.startedAtMs - left.startedAtMs);
        return {
          runs: runs.slice(0, 50),
          refreshedAtMs: event.timestampMs,
        };
      });

      if (["queued", "running"].includes(event.run.state)) {
        setStatus("Working");
      } else if (event.run.state === "paused") {
        setStatus("Waiting");
      } else {
        setStatus("Idle");
      }
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else cleanupAgent = unlisten;
      })
      .catch(() => {
        // Agent events supplement explicit snapshots.
      });

    listenToAutomation((event: AutomationEvent) => {
      if (cancelled) return;
      setAutomationEvent(event);
      setActivity(`${event.automationName}: ${event.message}`);
      void getAuraAutomations()
        .then((items) => {
          if (!cancelled) setAutomations(items);
        })
        .catch(() => undefined);
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else cleanupAutomation = unlisten;
      })
      .catch(() => {
        // Automation events supplement explicit snapshots.
      });

    listenToManagedRuntime((runtime: ManagedRuntimeStatus) => {
      if (cancelled) return;
      setManagedRuntimeStatus(runtime);
      setActivity(runtime.message);

      if (runtime.state === "ready") {
        void getModelRuntimeStatus()
          .then((modelRuntime) => {
            if (!cancelled) setModelRuntimeStatus(modelRuntime);
          })
          .catch(() => {
            // Model runtime will refresh again when local inference starts.
          });
      }
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanupManagedRuntime = unlisten;
        }
      })
      .catch(() => {
        // Managed runtime events are supplementary to explicit status reads.
      });

    listenToAuraCore(
      (event: CoreEvent) => {
        if (cancelled) return;
        setStatus(event.status);
        setActivity(event.message);
        setBridgeError(null);

        if (
          event.kind === "command.completed" ||
          event.kind === "command.failed"
        ) {
          window.setTimeout(() => {
            void getTtsRuntimeStatus()
              .then((runtime) => {
                if (!cancelled) setTtsRuntime(runtime);
              })
              .catch(() => undefined);
          }, 120);
        }

        if (
          ["command.completed", "command.failed", "command.cancelled"].includes(
            event.kind,
          )
        ) {
          const messageId = `${event.id}:assistant`;
          setChatMessages((current) =>
            current.some((message) => message.id === messageId)
              ? current
              : [
                  ...current,
                  {
                    id: messageId,
                    role: "assistant",
                    content: event.message,
                    timestampMs: event.timestampMs,
                  },
                ],
          );
        }

        if (event.kind === "command.completed") {
          void getMemories()
            .then((snapshot) => {
              if (!cancelled) setMemory(snapshot);
            })
            .catch(() => {
              // Memory refresh is supplementary to the command result.
            });
        }

        if (
          event.kind === "command.awaiting_confirmation"
          && event.command
        ) {
          setPendingConfirmation({
            id: event.id,
            command: event.command,
            message: event.message,
          });
        } else if (
          [
            "command.confirmed",
            "command.cancelled",
            "command.completed",
            "command.failed",
          ].includes(event.kind)
        ) {
          setPendingConfirmation((current) =>
            current?.id === event.id ? null : current,
          );
        }
      },
      (error: CoreError) => {
        if (cancelled) return;
        setStatus("Idle");
        setBridgeError(error);
        setActivity(error.message);
      },
      (runtime: RuntimeState) => {
        if (cancelled) return;
        setRuntimeState(runtime);
        setStatus("Idle");
        setBridgeError(null);

        if (runtime.paused) {
          setActivity(
            "AURA paused. New commands and future background actions are disabled.",
          );
        } else if (runtime.backgroundEnabled) {
          setActivity("AURA is ready and background mode is enabled.");
        } else {
          setActivity("Background mode is disabled. Closing AURA will quit the app.");
        }
      },
    )
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanupCore = unlisten;
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setBridgeError({
            code: "bridge.listen_failed",
            message: String(error),
          });
        }
      });

    listenToLifecycle((event: LifecycleEvent) => {
      if (cancelled) return;
      setActivity(event.message);
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanupLifecycle = unlisten;
        }
      })
      .catch(() => {
        // Lifecycle messaging is supplementary; the main Core bridge remains authoritative.
      });

    return () => {
      cancelled = true;
      cleanupCore?.();
      cleanupLifecycle?.();
      cleanupModels?.();
      cleanupManagedRuntime?.();
      cleanupVoiceCapture?.();
      cleanupVision?.();
      cleanupAgent?.();
      cleanupAutomation?.();
    };
  }, []);

  useEffect(() => {
    if (ttsRuntime.state !== "speaking") return;

    let cancelled = false;
    const interval = window.setInterval(() => {
      void getTtsRuntimeStatus()
        .then((runtime) => {
          if (!cancelled) setTtsRuntime(runtime);
        })
        .catch(() => undefined);
    }, 350);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [ttsRuntime.state]);

  useEffect(() => {
    if (!audioInput.testing && !audioInput.pushToTalk) return;

    let cancelled = false;

    const updateAudioLevel = async () => {
      try {
        const snapshot = await getAudioInputState();
        if (!cancelled) setAudioInput(snapshot);
      } catch {
        // Keep the most recent microphone state; stream errors are reflected in the snapshot.
      }
    };

    const interval = window.setInterval(() => void updateAudioLevel(), 120);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [audioInput.testing, audioInput.pushToTalk]);

  useEffect(() => {
    if (runtimeState.paused) {
      return;
    }

    let cancelled = false;

    const updateCurrentApp = async () => {
      try {
        const context = await getCurrentAppContext();
        if (!cancelled) {
          setCurrentApp(context);
        }
      } catch {
        // Current-app awareness is contextual; keep the last valid snapshot.
      }
    };

    void updateCurrentApp();
    const interval = window.setInterval(() => void updateCurrentApp(), 1000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [runtimeState.paused]);

  const refreshObsRuntime = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsRuntime(DEFAULT_OBS_RUNTIME);
      return DEFAULT_OBS_RUNTIME;
    }

    const state = await getObsRuntimeState();
    setObsRuntime(state);

    if (!state.available) {
      setObsConnection((current) => ({
        ...current,
        connected: false,
        lastError: state.lastError,
      }));
    }

    return state;
  }, [obsConnection.connected]);


  const refreshObsScenes = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsScenes(DEFAULT_OBS_SCENES);
      return DEFAULT_OBS_SCENES;
    }

    const scenes = await getObsScenes();
    setObsScenes(scenes);
    return scenes;
  }, [obsConnection.connected]);


  const refreshObsSources = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsSources(DEFAULT_OBS_SOURCES);
      return DEFAULT_OBS_SOURCES;
    }

    const sources = await getObsSourceItems();
    setObsSources(sources);
    return sources;
  }, [obsConnection.connected]);


  const refreshObsAudio = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsAudio(DEFAULT_OBS_AUDIO);
      return DEFAULT_OBS_AUDIO;
    }

    const audio = await getObsAudioInputs();
    setObsAudio(audio);
    return audio;
  }, [obsConnection.connected]);


  const refreshObsHealth = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsHealth(null);
      return null;
    }

    const health = await getObsProductionHealth();
    setObsHealth(health);
    return health;
  }, [obsConnection.connected]);


  const refreshModelRuntime = useCallback(async () => {
    const runtime = await getModelRuntimeStatus();
    setModelRuntimeStatus(runtime);
    return runtime;
  }, []);

  const clearConversationControl = useCallback(async () => {
    setChatMessages([]);

    try {
      const runtime = await clearModelConversation();
      setModelRuntimeStatus(runtime);
      setActivity("Started a new local conversation.");
      return runtime;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "models.conversation_reset_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const refreshManagedRuntime = useCallback(async () => {
    const runtime = await getManagedRuntimeStatus();
    setManagedRuntimeStatus(runtime);
    return runtime;
  }, []);

  const runManagedRuntimeAction = useCallback(async (
    action: "install" | "repair" | "remove",
  ): Promise<ManagedRuntimeStatus> => {
    try {
      setBridgeError(null);

      const runtime =
        action === "install"
          ? await installManagedRuntime()
          : action === "repair"
            ? await repairManagedRuntime()
            : await removeManagedRuntime();

      setManagedRuntimeStatus(runtime);
      setActivity(runtime.message);

      if (action === "repair" || action === "remove") {
        const modelRuntime = await getModelRuntimeStatus().catch(
          () => DEFAULT_MODEL_RUNTIME,
        );
        setModelRuntimeStatus(modelRuntime);
      }

      return runtime;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: `runtime.${action}_failed`,
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const refreshModels = useCallback(async () => {
    const catalog = await getModelCatalog();
    setModelCatalog(catalog);
    return catalog;
  }, []);

  const runModelOperation = useCallback(async (
    operation:
      | "download"
      | "pause"
      | "resume"
      | "cancel"
      | "activate"
      | "remove",
    modelId: string,
  ): Promise<ModelCatalog> => {
    try {
      setBridgeError(null);

      const catalog = await (
        operation === "download"
          ? startModelDownload(modelId)
          : operation === "pause"
            ? pauseModelDownload(modelId)
            : operation === "resume"
              ? resumeModelDownload(modelId)
              : operation === "cancel"
                ? cancelModelDownload(modelId)
                : operation === "activate"
                  ? setActiveModel(modelId)
                  : removeModel(modelId)
      );

      setModelCatalog(catalog);

      if (operation === "activate" || operation === "remove") {
        const runtime = await getModelRuntimeStatus().catch(() => DEFAULT_MODEL_RUNTIME);
        setModelRuntimeStatus(runtime);
      }

      const model = catalog.models.find((item) => item.id === modelId);
      const name = model?.name ?? modelId;

      setActivity(
        operation === "download"
          ? `Downloading ${name}…`
          : operation === "pause"
            ? `${name} download paused.`
            : operation === "resume"
              ? `${name} download resumed.`
              : operation === "cancel"
                ? `Cancelling ${name} download…`
                : operation === "activate"
                  ? `${name} selected as the active local model.`
                  : `${name} removed from this computer.`,
      );

      return catalog;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: `models.${operation}_failed`,
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const refreshCurrentApp = useCallback(async () => {
    const context = await getCurrentAppContext();
    setCurrentApp(context);
    return context;
  }, []);

  const refreshRecentFiles = useCallback(async () => {
    const snapshot = await getRecentFilesContext();
    setRecentFiles(snapshot);
    return snapshot;
  }, []);

  const refreshRoutines = useCallback(async () => {
    const items = await getUserRoutines();
    setRoutines(items);
    return items;
  }, []);

  const refreshProjectMemory = useCallback(async () => {
    const snapshot = await getProjectMemory();
    setProjectMemory(snapshot);
    return snapshot;
  }, []);

  const updateVoicePreferences = useCallback(async (
    preferences: VoicePreferences,
  ) => {
    const saved = await setVoicePreferences(preferences);
    setVoicePreferencesState(saved);
    return saved;
  }, []);

  const stopSpeakingControl = useCallback(async () => {
    const runtime = await stopTtsSpeaking();
    setTtsRuntime(runtime);
    return runtime;
  }, []);

  const planAgentGoal = useCallback(async (goal: string) => {
    const plan = await createAgentPlan(goal);
    setAgentPlan(plan);
    return plan;
  }, []);

  const clearAgentPlan = useCallback(() => {
    setAgentPlan(null);
  }, []);

  const refreshAgentRuns = useCallback(async () => {
    const snapshot = await getAgentRuns();
    setAgentRuns(snapshot);
    return snapshot;
  }, []);

  const startAgentPlanControl = useCallback(async (
    plan: AgentPlan,
    approved: boolean,
  ) => {
    const run = await startAgentPlan(plan, approved);
    setAgentRuns((current) => ({
      runs: [run, ...current.runs.filter((item) => item.id !== run.id)],
      refreshedAtMs: Date.now(),
    }));
    return run;
  }, []);

  const pauseAgentRunControl = useCallback(async (
    runId: string,
    paused: boolean,
  ) => {
    const run = await pauseAgentRun(runId, paused);
    setAgentRuns((current) => ({
      ...current,
      runs: current.runs.map((item) => (item.id === run.id ? run : item)),
      refreshedAtMs: Date.now(),
    }));
    return run;
  }, []);

  const cancelAgentRunControl = useCallback(async (runId: string) => {
    const run = await cancelAgentRun(runId);
    return run;
  }, []);

  const refreshSavedActions = useCallback(async () => {
    const items = await getSavedAuraActions();
    setSavedActions(items);
    return items;
  }, []);

  const saveAuraActionControl = useCallback(async (
    request: SaveAuraActionRequest,
  ) => {
    const saved = await saveAuraAction(request);
    const items = await getSavedAuraActions();
    setSavedActions(items);
    return saved;
  }, []);

  const deleteAuraActionControl = useCallback(async (actionId: string) => {
    await deleteAuraAction(actionId);
    const items = await getSavedAuraActions();
    setSavedActions(items);
  }, []);

  const runAuraActionControl = useCallback(async (actionId: string) => {
    const result = await runAuraAction(actionId, true);
    setActivity(result);
    return result;
  }, []);

  const refreshAutomations = useCallback(async () => {
    const items = await getAuraAutomations();
    setAutomations(items);
    return items;
  }, []);

  const saveAutomationControl = useCallback(async (
    request: SaveAutomationRequest,
  ) => {
    const saved = await saveAuraAutomation(request);
    const items = await getAuraAutomations();
    setAutomations(items);
    return saved;
  }, []);

  const deleteAutomationControl = useCallback(async (automationId: string) => {
    await deleteAuraAutomation(automationId);
    const items = await getAuraAutomations();
    setAutomations(items);
  }, []);

  const setAutomationEnabledControl = useCallback(async (
    automationId: string,
    enabled: boolean,
  ) => {
    const saved = await setAuraAutomationEnabled(automationId, enabled);
    setAutomations((current) =>
      current.map((item) => (item.id === saved.id ? saved : item)),
    );
    return saved;
  }, []);

  const refreshVisionRuntime = useCallback(async () => {
    const runtime = await getVisionRuntimeStatus();
    setVisionRuntime(runtime);
    return runtime;
  }, []);

  const refreshVisionHistory = useCallback(async () => {
    const snapshot = await getVisionHistory();
    setVisionHistory(snapshot);
    return snapshot;
  }, []);

  const updateVisionPreferences = useCallback(async (
    preferences: VisionPreferences,
  ) => {
    const snapshot = await setVisionPreferences(preferences);
    setVisionHistory(snapshot);
    return snapshot;
  }, []);

  const clearVisionHistoryControl = useCallback(async () => {
    const snapshot = await clearVisionHistory();
    setVisionHistory(snapshot);
    return snapshot;
  }, []);

  const captureVisionScreenControl = useCallback(async () => {
    const capture = await captureVisionScreen();
    setVisionCapture(capture);
    return capture;
  }, []);

  const captureVisionActiveWindowControl = useCallback(async () => {
    const capture = await captureVisionActiveWindow();
    setVisionCapture(capture);
    return capture;
  }, []);

  const captureVisionRegionControl = useCallback(async (
    request: VisionRegionRequest,
  ) => {
    const capture = await captureVisionRegion(request);
    setVisionCapture(capture);
    return capture;
  }, []);

  const analyzeVisionControl = useCallback(async (prompt: string) => {
    const result = await analyzeLastVisionCapture(prompt);
    setVisionCapture(null);
    setVisionHistory(result.history);
    setVisionRuntime((current) => ({
      ...current,
      state: "ready",
      lastPrompt: result.prompt,
      lastAnalysis: result.analysis,
      lastError: null,
      refreshedAtMs: result.completedAtMs,
    }));
    return result;
  }, []);

  const clearVisionCaptureControl = useCallback(async () => {
    await clearLastVisionCapture();
    setVisionCapture(null);
    setVisionEvent(null);
  }, []);

  const refreshTtsRuntime = useCallback(async () => {
    const runtime = await getTtsRuntimeStatus();
    setTtsRuntime(runtime);
    return runtime;
  }, []);

  const prepareTtsRuntimeControl = useCallback(async () => {
    const runtime = await prepareTtsRuntime();
    setTtsRuntime(runtime);
    return runtime;
  }, []);

  const testTtsVoiceControl = useCallback(async (text?: string) => {
    const runtime = await testTtsVoice(text);
    setTtsRuntime(runtime);
    return runtime;
  }, []);

  const refreshAudioInput = useCallback(async () => {
    const snapshot = await getAudioInputState();
    setAudioInput(snapshot);
    return snapshot;
  }, []);

  const selectAudioInput = useCallback(async (deviceName?: string) => {
    const snapshot = await selectAudioInputDevice(deviceName);
    setAudioInput(snapshot);
    return snapshot;
  }, []);

  const startAudioTest = useCallback(async () => {
    const snapshot = await startAudioInputTest();
    setAudioInput(snapshot);
    return snapshot;
  }, []);

  const stopAudioTest = useCallback(async () => {
    const snapshot = await stopAudioInputTest();
    setAudioInput(snapshot);
    return snapshot;
  }, []);

  const saveProjectMemoryControl = useCallback(async (request: SaveProjectRequest) => {
    const project = await saveProjectMemory(request);
    const snapshot = await getProjectMemory();
    setProjectMemory(snapshot);
    setActivity(`Saved project ${project.name}.`);
    return project;
  }, []);

  const deleteProjectMemoryControl = useCallback(async (projectId: string) => {
    await deleteProjectMemory(projectId);
    const snapshot = await getProjectMemory();
    setProjectMemory(snapshot);
    setActivity("Project memory deleted.");
  }, []);

  const setActiveProjectMemoryControl = useCallback(async (projectId?: string) => {
    const snapshot = await setActiveProjectMemory(projectId);
    setProjectMemory(snapshot);
    const active = snapshot.projects.find((project) => project.id === snapshot.activeProjectId);
    setActivity(active ? `Active project: ${active.name}.` : "No active project.");
    return snapshot;
  }, []);

  const saveRoutineControl = useCallback(async (request: SaveRoutineRequest) => {
    try {
      setBridgeError(null);
      const routine = await saveUserRoutine(request);
      const items = await getUserRoutines();
      setRoutines(items);
      setActivity(`Saved routine ${routine.name}.`);
      return routine;
    } catch (error) {
      const message = String(error);
      setBridgeError({ code: "routine.save_failed", message });
      setActivity(message);
      throw error;
    }
  }, []);

  const deleteRoutineControl = useCallback(async (routineId: string) => {
    try {
      setBridgeError(null);
      await deleteUserRoutine(routineId);
      const items = await getUserRoutines();
      setRoutines(items);
      setActivity("Routine deleted.");
    } catch (error) {
      const message = String(error);
      setBridgeError({ code: "routine.delete_failed", message });
      setActivity(message);
      throw error;
    }
  }, []);

  const runRoutineControl = useCallback(async (routineId: string) => {
    try {
      setBridgeError(null);
      const result = await runUserRoutine(routineId);
      setRoutineLastRun(result);
      setActivity(
        result.success
          ? `Routine ${result.routineName} completed.`
          : result.error ?? `Routine ${result.routineName} failed.`,
      );
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({ code: "routine.run_failed", message });
      setActivity(message);
      throw error;
    }
  }, []);

  const refreshMemories = useCallback(async () => {
    const snapshot = await getMemories();
    setMemory(snapshot);
    return snapshot;
  }, []);

  const createMemoryControl = useCallback(async (
    content: string,
  ): Promise<MemoryCreateResult> => {
    try {
      setBridgeError(null);
      const result = await createMemory({ content });
      const snapshot = await getMemories();
      setMemory(snapshot);
      setActivity(
        result.created
          ? `Remembered: ${result.record.content}`
          : `Memory already existed and was refreshed: ${result.record.content}`,
      );
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "memory.create_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const deleteMemoryControl = useCallback(async (
    memoryId: string,
  ) => {
    try {
      setBridgeError(null);
      const removed = await deleteMemory(memoryId);
      const snapshot = await getMemories();
      setMemory(snapshot);
      setActivity(`Forgot: ${removed.content}`);
      return removed;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "memory.delete_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const refreshDirectorPresets = useCallback(async () => {
    const presets = await getDirectorPresets();
    setDirectorPresets(presets);
    return presets;
  }, []);

  const saveDirectorPresetControl = useCallback(async (
    request: SaveDirectorPresetRequest,
  ): Promise<DirectorPreset> => {
    try {
      setBridgeError(null);
      const preset = await saveDirectorPreset(request);
      const presets = await getDirectorPresets();
      setDirectorPresets(presets);
      setActivity(`Director Mode preset ${preset.name} saved.`);
      return preset;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "director.preset_save_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const deleteDirectorPresetControl = useCallback(async (
    presetId: string,
  ): Promise<void> => {
    try {
      setBridgeError(null);
      await deleteDirectorPreset(presetId);
      const presets = await getDirectorPresets();
      setDirectorPresets(presets);
      setDirectorLastRun((current) =>
        current?.presetId === presetId ? null : current,
      );
      setActivity("Director Mode preset deleted.");
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "director.preset_delete_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const runDirectorPresetControl = useCallback(async (
    presetId: string,
  ): Promise<DirectorPresetRunResult> => {
    try {
      setBridgeError(null);
      const result = await runDirectorPreset(presetId);
      setDirectorLastRun(result);

      const refreshes = await Promise.allSettled([
        getObsRuntimeState(),
        getObsScenes(),
        getObsSourceItems(),
        getObsAudioInputs(),
        getObsProductionHealth(),
      ]);

      const [runtime, scenes, sources, audio, health] = refreshes;
      if (runtime.status === "fulfilled") setObsRuntime(runtime.value);
      if (scenes.status === "fulfilled") setObsScenes(scenes.value);
      if (sources.status === "fulfilled") setObsSources(sources.value);
      if (audio.status === "fulfilled") setObsAudio(audio.value);
      if (health.status === "fulfilled") setObsHealth(health.value);

      if (result.success) {
        setActivity(
          `Director Mode preset ${result.presetName} completed: ${result.completedSteps}/${result.totalSteps} steps.`,
        );
      } else {
        const message = result.error ?? "Director Mode preset failed.";
        setBridgeError({
          code: "director.preset_failed",
          message,
        });
        setActivity(message);
      }

      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "director.preset_run_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const switchObsProgramScene = useCallback(async (
    sceneUuid: string,
  ): Promise<ObsSceneSwitchResult> => {
    try {
      setBridgeError(null);
      const result = await setObsProgramScene({ sceneUuid });
      const [runtime, scenes, sources, audio] = await Promise.all([
        getObsRuntimeState(),
        getObsScenes(),
        getObsSourceItems(),
        getObsAudioInputs(),
      ]);
      setObsRuntime(runtime);
      setObsScenes(scenes);
      setObsSources(sources);
      setObsAudio(audio);
      setActivity(`OBS Program switched to ${result.sceneName}.`);
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.scene_switch_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const switchObsPreviewScene = useCallback(async (
    sceneUuid: string,
  ): Promise<ObsSceneSwitchResult> => {
    try {
      setBridgeError(null);
      const result = await setObsPreviewScene({ sceneUuid });
      const [runtime, scenes, sources] = await Promise.all([
        getObsRuntimeState(),
        getObsScenes(),
        getObsSourceItems(),
      ]);
      setObsRuntime(runtime);
      setObsScenes(scenes);
      setObsSources(sources);
      setActivity(`OBS Preview switched to ${result.sceneName}.`);
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.preview_switch_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const controlObsRecording = useCallback(async (
    action: ObsRecordingActionResult["action"],
  ): Promise<ObsRecordingActionResult> => {
    try {
      setBridgeError(null);

      const result = await (
        action === "start"
          ? startObsRecording()
          : action === "stop"
            ? stopObsRecording()
            : action === "pause"
              ? pauseObsRecording()
              : resumeObsRecording()
      );

      const runtime = await getObsRuntimeState();
      setObsRuntime(runtime);

      const message = action === "start"
        ? "OBS recording started."
        : action === "pause"
          ? "OBS recording paused."
          : action === "resume"
            ? "OBS recording resumed."
            : result.outputPath
              ? `OBS recording stopped. Saved to ${result.outputPath}.`
              : "OBS recording stopped.";

      setActivity(message);
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.recording_control_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const controlObsStreaming = useCallback(async (
    action: ObsStreamingActionResult["action"],
  ): Promise<ObsStreamingActionResult> => {
    try {
      setBridgeError(null);

      const result = await (
        action === "start"
          ? startObsStreaming()
          : stopObsStreaming()
      );

      const runtime = await getObsRuntimeState();
      setObsRuntime(runtime);

      setActivity(
        action === "start"
          ? "OBS stream is live."
          : "OBS stream stopped.",
      );

      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.streaming_control_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const controlObsSourceVisibility = useCallback(async (
    sceneName: string,
    itemId: number,
    enabled: boolean,
  ): Promise<ObsSourceVisibilityResult> => {
    try {
      setBridgeError(null);

      const result = await setObsSourceVisibility({
        sceneName,
        itemId,
        enabled,
      });

      const sources = await getObsSourceItems();
      setObsSources(sources);
      setActivity(
        `OBS source ${result.sourceName} is now ${result.enabled ? "visible" : "hidden"} in ${result.sceneName}.`,
      );

      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.source_visibility_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const controlObsAudioMute = useCallback(async (
    inputUuid: string,
    muted: boolean,
  ): Promise<ObsAudioControlResult> => {
    try {
      setBridgeError(null);

      const result = await setObsAudioMuted({ inputUuid, muted });
      const audio = await getObsAudioInputs();
      setObsAudio(audio);
      setActivity(
        `OBS input ${result.inputName} is now ${result.muted ? "muted" : "unmuted"}.`,
      );

      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.audio_mute_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const controlObsAudioVolume = useCallback(async (
    inputUuid: string,
    percent: number,
  ): Promise<ObsAudioControlResult> => {
    try {
      setBridgeError(null);

      const result = await setObsAudioVolume({ inputUuid, percent });
      const audio = await getObsAudioInputs();
      setObsAudio(audio);
      setActivity(
        `OBS input ${result.inputName} volume set to ${result.volumePercent}%.`,
      );

      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.audio_volume_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    const updateRuntime = async () => {
      try {
        const runtime = await getModelRuntimeStatus();
        if (!cancelled) setModelRuntimeStatus(runtime);
      } catch {
        // Runtime status is supplementary to Core command events.
      }
    };

    void updateRuntime();

    if (status !== "Working") {
      return () => {
        cancelled = true;
      };
    }

    const interval = window.setInterval(() => void updateRuntime(), 600);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [status]);

  useEffect(() => {
    if (!obsConnection.connected) {
      setObsRuntime(DEFAULT_OBS_RUNTIME);
      return;
    }

    let cancelled = false;

    const update = async () => {
      try {
        const state = await getObsRuntimeState();
        if (cancelled) return;

        setObsRuntime(state);

        if (!state.available) {
          setObsConnection((current) => ({
            ...current,
            connected: false,
            lastError: state.lastError,
          }));
        }
      } catch (error) {
        if (cancelled) return;

        const message = String(error);
        setObsRuntime({
          ...DEFAULT_OBS_RUNTIME,
          refreshedAtMs: Date.now(),
          lastError: message,
        });
        setObsConnection((current) => ({
          ...current,
          connected: false,
          lastError: message,
        }));
      }
    };

    void update();
    const interval = window.setInterval(() => void update(), 2000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected]);

  useEffect(() => {
    if (!obsConnection.connected) {
      setObsScenes(DEFAULT_OBS_SCENES);
      return;
    }

    let cancelled = false;

    const updateScenes = async () => {
      try {
        const scenes = await getObsScenes();
        if (!cancelled) {
          setObsScenes(scenes);
        }
      } catch (error) {
        if (!cancelled) {
          setObsScenes({
            ...DEFAULT_OBS_SCENES,
            refreshedAtMs: Date.now(),
            lastError: String(error),
          });
        }
      }
    };

    void updateScenes();
    const interval = window.setInterval(() => void updateScenes(), 5000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected]);

  useEffect(() => {
    if (!obsConnection.connected || !obsRuntime.currentProgramScene) {
      setObsSources(DEFAULT_OBS_SOURCES);
      return;
    }

    let cancelled = false;

    const updateSources = async () => {
      try {
        const sources = await getObsSourceItems();
        if (!cancelled) {
          setObsSources(sources);
        }
      } catch (error) {
        if (!cancelled) {
          setObsSources({
            ...DEFAULT_OBS_SOURCES,
            sceneName: obsRuntime.currentProgramScene ?? "",
            refreshedAtMs: Date.now(),
            lastError: String(error),
          });
        }
      }
    };

    void updateSources();
    const interval = window.setInterval(() => void updateSources(), 5000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected, obsRuntime.currentProgramScene]);

  useEffect(() => {
    if (!obsConnection.connected) {
      setObsAudio(DEFAULT_OBS_AUDIO);
      return;
    }

    let cancelled = false;

    const updateAudio = async () => {
      try {
        const audio = await getObsAudioInputs();
        if (!cancelled) {
          setObsAudio(audio);
        }
      } catch (error) {
        if (!cancelled) {
          setObsAudio({
            ...DEFAULT_OBS_AUDIO,
            refreshedAtMs: Date.now(),
            lastError: String(error),
          });
        }
      }
    };

    void updateAudio();
    const interval = window.setInterval(() => void updateAudio(), 5000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected]);

  useEffect(() => {
    if (!obsConnection.connected) {
      setObsHealth(null);
      return;
    }

    let cancelled = false;

    const updateHealth = async () => {
      try {
        const health = await getObsProductionHealth();
        if (!cancelled) {
          setObsHealth(health);
        }
      } catch {
        if (!cancelled) {
          setObsHealth(null);
        }
      }
    };

    void updateHealth();
    const interval = window.setInterval(() => void updateHealth(), 5000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected]);

  const submitCommand = useCallback(async (
    text: string,
    source: "desktop" | "overlay" | "voice" = "desktop",
  ) => {
    const trimmed = text.trim();
    if (!trimmed) return null;

    const clientMessageId = `user-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    setChatMessages((current) => [
      ...current,
      {
        id: clientMessageId,
        role: "user",
        content: trimmed,
        timestampMs: Date.now(),
      },
    ]);

    try {
      setBridgeError(null);
      const ack = await submitAuraCommand({
        text: trimmed,
        source,
      });
      setStatus(ack.status);
      return ack;
    } catch (error) {
      const coreError: CoreError = {
        code: "bridge.command_failed",
        message: String(error),
      };
      setBridgeError(coreError);
      setStatus("Idle");
      setActivity(coreError.message);
      setChatMessages((current) => [
        ...current,
        {
          id: `${clientMessageId}:error`,
          role: "assistant",
          content: coreError.message,
          timestampMs: Date.now(),
        },
      ]);
      return null;
    }
  }, []);

  const setPaused = useCallback(async (paused: boolean) => {
    const state = await setRuntimePaused(paused);
    setRuntimeState(state);
    return state;
  }, []);

  const setBackgroundMode = useCallback(async (backgroundEnabled: boolean) => {
    const state = await setBackgroundEnabled(backgroundEnabled);
    setRuntimeState(state);
    return state;
  }, []);

  const setAutostart = useCallback(async (autostartEnabled: boolean) => {
    const state = await setAutostartEnabled(autostartEnabled);
    setRuntimeState(state);
    return state;
  }, []);

  const setPermission = useCallback(async (
    permissionClass: PermissionClass,
    decision: PermissionDecision,
  ) => {
    const policy = await setPermissionDecision(permissionClass, decision);
    setPermissionPolicyState(policy);
    return policy;
  }, []);

  const resetPermissions = useCallback(async () => {
    const policy = await resetPermissionPolicy();
    setPermissionPolicyState(policy);
    return policy;
  }, []);

  const connectObsControl = useCallback(async (request: ObsConnectRequest) => {
    try {
      setBridgeError(null);
      const state = await connectObs(request);
      setObsConnection(state);
      const [runtime, scenes] = await Promise.all([
        getObsRuntimeState(),
        getObsScenes(),
      ]);
      setObsRuntime(runtime);
      setObsScenes(scenes);
      setActivity(
        `OBS Studio ${state.obsStudioVersion ?? ""} connected at ${state.host}:${state.port}.`,
      );
      return state;
    } catch (error) {
      const refreshed = await getObsConnectionState().catch(() => ({
        connected: false,
        host: request.host || "127.0.0.1",
        port: request.port || 4455,
        lastError: String(error),
      } satisfies ObsConnectionState));

      setObsConnection(refreshed);
      setBridgeError({
        code: "obs.connection_failed",
        message: String(error),
      });
      setActivity(String(error));
      throw error;
    }
  }, []);

  const disconnectObsControl = useCallback(async () => {
    const state = await disconnectObs();
    setObsConnection(state);
    setObsRuntime(DEFAULT_OBS_RUNTIME);
    setObsScenes(DEFAULT_OBS_SCENES);
    setObsSources(DEFAULT_OBS_SOURCES);
    setObsAudio(DEFAULT_OBS_AUDIO);
    setObsHealth(null);
    setBridgeError(null);
    setActivity("OBS Studio disconnected from AURA.");
    return state;
  }, []);

  const approveConfirmation = useCallback(async (id: string) => {
    try {
      setBridgeError(null);
      const ack = await resolveConfirmation(id, true);
      if (ack) {
        setStatus(ack.status);
      }
      return ack;
    } catch (error) {
      const coreError: CoreError = {
        id,
        code: "bridge.confirmation_failed",
        message: String(error),
      };
      setPendingConfirmation(null);
      setBridgeError(coreError);
      setStatus("Idle");
      setActivity(coreError.message);
      return null;
    }
  }, []);

  const cancelConfirmation = useCallback(async (id: string) => {
    try {
      setBridgeError(null);
      await resolveConfirmation(id, false);
    } catch (error) {
      const coreError: CoreError = {
        id,
        code: "bridge.confirmation_failed",
        message: String(error),
      };
      setPendingConfirmation(null);
      setBridgeError(coreError);
      setStatus("Idle");
      setActivity(coreError.message);
    }
  }, []);

  return {
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
    refreshCurrentApp,
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
    refreshVisionHistory,
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
    refreshModelRuntime,
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
  };
}
