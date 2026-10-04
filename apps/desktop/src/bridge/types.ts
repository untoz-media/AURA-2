export type AuraStatus = "Idle" | "Listening" | "Thinking" | "Working" | "Waiting";

export type CoreEventKind =
  | "command.accepted"
  | "command.processing"
  | "command.completed"
  | "command.failed"
  | "command.awaiting_confirmation"
  | "command.confirmed"
  | "command.cancelled"
  | "command.unhandled";

export type CoreEvent = {
  id: string;
  kind: CoreEventKind;
  status: AuraStatus;
  message: string;
  command?: string;
  timestampMs: number;
};

export type CommandRequest = {
  text: string;
  source: "desktop" | "overlay" | "voice";
  approvalId?: string;
};

export type PendingConfirmation = {
  id: string;
  command: string;
  message: string;
};

export type CommandAck = {
  id: string;
  accepted: boolean;
  status: AuraStatus;
};

export type AppStatus = {
  name: string;
  version: string;
  stage: string;
  localFirst: boolean;
};

export type CoreError = {
  id?: string;
  code: string;
  message: string;
};

export type RuntimeState = {
  paused: boolean;
  backgroundEnabled: boolean;
  autostartEnabled: boolean;
};

export type LifecycleEventKind =
  | "foreground.entered"
  | "background.entered"
  | "background.enabled"
  | "background.disabled"
  | "autostart.enabled"
  | "autostart.disabled"
  | "startup.background"
  | "permissions.updated"
  | "permissions.reset";

export type LifecycleEvent = {
  kind: LifecycleEventKind;
  message: string;
  timestampMs: number;
};

export type PermissionClass =
  | "read"
  | "act"
  | "modify"
  | "destructive"
  | "sensitive";

export type PermissionDecision = "allow" | "ask" | "never";

export type PermissionPolicy = {
  read: PermissionDecision;
  act: PermissionDecision;
  modify: PermissionDecision;
  destructive: PermissionDecision;
  sensitive: PermissionDecision;
};


export type ObsConnectRequest = {
  host?: string;
  port?: number;
  password?: string;
};

export type ObsConnectionState = {
  connected: boolean;
  host: string;
  port: number;
  obsStudioVersion?: string;
  obsWebsocketVersion?: string;
  rpcVersion?: number;
  connectedAtMs?: number;
  lastError?: string;
};


export type ObsRuntimeState = {
  available: boolean;
  streaming: boolean;
  recording: boolean;
  recordingPaused: boolean;
  studioMode: boolean;
  currentProgramScene?: string;
  currentPreviewScene?: string;
  streamDurationMs: number;
  streamTimecode: string;
  refreshedAtMs: number;
  lastError?: string;
};


export type ObsSceneSummary = {
  name: string;
  uuid: string;
  index: number;
  isProgram: boolean;
  isPreview: boolean;
};

export type ObsSceneList = {
  scenes: ObsSceneSummary[];
  currentProgramScene?: string;
  currentPreviewScene?: string;
  refreshedAtMs: number;
  lastError?: string;
};


export type ObsSceneSwitchRequest = {
  sceneUuid: string;
};

export type ObsSceneSwitchResult = {
  target: "program" | "preview";
  sceneName: string;
  sceneUuid: string;
  changedAtMs: number;
};


export type ObsRecordingActionResult = {
  action: "start" | "stop" | "pause" | "resume";
  recording: boolean;
  paused: boolean;
  outputPath?: string;
  changedAtMs: number;
};


export type ObsStreamingActionResult = {
  action: "start" | "stop";
  streaming: boolean;
  changedAtMs: number;
};


export type ObsStreamDuration = {
  streaming: boolean;
  durationMs: number;
  timecode: string;
  refreshedAtMs: number;
};


export type ObsSourceItemSummary = {
  sceneName: string;
  itemId: number;
  index: number;
  sourceName: string;
  enabled: boolean;
  inputKind?: string;
  isGroup: boolean;
};

export type ObsSourceItemList = {
  sceneName: string;
  items: ObsSourceItemSummary[];
  refreshedAtMs: number;
  lastError?: string;
};

export type ObsSourceVisibilityRequest = {
  sceneName: string;
  itemId: number;
  enabled: boolean;
};

export type ObsSourceVisibilityResult = {
  sceneName: string;
  itemId: number;
  sourceName: string;
  enabled: boolean;
  changedAtMs: number;
};


export type ObsAudioInputSummary = {
  inputName: string;
  inputUuid: string;
  inputKind: string;
  muted: boolean;
  volumePercent: number;
  volumeMul: number;
  volumeDb: number;
};

export type ObsAudioInputList = {
  inputs: ObsAudioInputSummary[];
  refreshedAtMs: number;
  lastError?: string;
};

export type ObsAudioMuteRequest = {
  inputUuid: string;
  muted: boolean;
};

export type ObsAudioVolumeRequest = {
  inputUuid: string;
  percent: number;
};

export type ObsAudioControlResult = {
  inputName: string;
  inputUuid: string;
  muted: boolean;
  volumePercent: number;
  volumeMul: number;
  volumeDb: number;
  changedAtMs: number;
};


export type ObsProductionHealth = {
  status: "good" | "warning" | "critical";
  summary: string;
  issues: string[];
  cpuUsagePercent: number;
  memoryUsageMb: number;
  availableDiskSpaceMb: number;
  activeFps: number;
  averageFrameRenderTimeMs: number;
  renderSkippedFrames: number;
  renderTotalFrames: number;
  renderSkippedPercent: number;
  outputSkippedFrames: number;
  outputTotalFrames: number;
  outputSkippedPercent: number;
  streaming: boolean;
  streamReconnecting: boolean;
  streamCongestionPercent?: number;
  streamBitrateKbps?: number;
  streamOutputSkippedFrames?: number;
  streamOutputTotalFrames?: number;
  streamDroppedPercent?: number;
  checkedAtMs: number;
};


export type DirectorRecordingAction = "start" | "stop" | "pause" | "resume";
export type DirectorStreamingAction = "start" | "stop";

export type DirectorPresetAction =
  | { type: "programScene"; sceneName: string }
  | { type: "previewScene"; sceneName: string }
  | { type: "sourceVisibility"; sourceName: string; enabled: boolean }
  | { type: "audioMute"; inputName: string; muted: boolean }
  | { type: "audioVolume"; inputName: string; percent: number }
  | { type: "recording"; action: DirectorRecordingAction }
  | { type: "streaming"; action: DirectorStreamingAction }
  | { type: "wait"; milliseconds: number };

export type DirectorPreset = {
  id: string;
  name: string;
  description: string;
  aliases: string[];
  actions: DirectorPresetAction[];
  updatedAtMs: number;
};

export type SaveDirectorPresetRequest = {
  id?: string;
  name: string;
  description: string;
  aliases: string[];
  actions: DirectorPresetAction[];
};

export type DirectorPresetStepResult = {
  index: number;
  status: "applied" | "skipped" | "failed";
  label: string;
  message: string;
};

export type DirectorPresetRunResult = {
  success: boolean;
  presetId: string;
  presetName: string;
  completedSteps: number;
  totalSteps: number;
  failedStep?: number;
  error?: string;
  steps: DirectorPresetStepResult[];
  startedAtMs: number;
  completedAtMs: number;
};


export type MemoryRecord = {
  id: string;
  content: string;
  source: string;
  createdAtMs: number;
  updatedAtMs: number;
};

export type MemorySnapshot = {
  records: MemoryRecord[];
  refreshedAtMs: number;
};

export type CreateMemoryRequest = {
  content: string;
};

export type MemoryCreateResult = {
  record: MemoryRecord;
  created: boolean;
};


export type CurrentAppInfo = {
  appName: string;
  processName: string;
  processId: number;
  knownApp: boolean;
  windowTitle?: string | null;
  contextSource: "foreground" | "lastExternal";
  capturedAtMs: number;
};

export type RecentFileItem = {
  name: string;
  modifiedAtMs: number;
};

export type RecentFilesSnapshot = {
  items: RecentFileItem[];
  source: "windowsRecentItems";
  refreshedAtMs: number;
};


export type ModelInstallState =
  | "unavailable"
  | "notInstalled"
  | "downloading"
  | "paused"
  | "installed"
  | "failed";

export type ModelStatus = {
  id: string;
  name: string;
  subtitle: string;
  description: string;
  generation: string;
  role: "assistant" | "speechToText" | "textToSpeech" | "vision" | string;
  state: ModelInstallState;
  downloadAvailable: boolean;
  installed: boolean;
  active: boolean;
  sourceRepo?: string;
  sourceRevision?: string;
  license?: string;
  estimatedSizeBytes?: number;
  bytesDownloaded: number;
  totalBytes?: number;
  progressPercent: number;
  bytesPerSecond?: number;
  currentFile?: string;
  error?: string;
  availabilityMessage?: string;
  installPath?: string;
  installedAtMs?: number;
};

export type ModelCatalog = {
  models: ModelStatus[];
  activeModelId?: string;
  modelsRoot: string;
  refreshedAtMs: number;
};

export type ModelDownloadProgress = {
  modelId: string;
  state: ModelInstallState;
  bytesDownloaded: number;
  totalBytes?: number;
  progressPercent: number;
  bytesPerSecond?: number;
  currentFile?: string;
  error?: string;
  updatedAtMs: number;
};


export type ModelRuntimeStatus = {
  state: "stopped" | "loading" | "ready" | "generating" | "error";
  loadedModelId?: string;
  pythonExecutable?: string;
  device?: string;
  cuda?: boolean;
  lastError?: string;
  refreshedAtMs: number;
};

export type ChatMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestampMs: number;
};


export type ManagedRuntimeStatus = {
  state:
    | "notInstalled"
    | "preparing"
    | "downloadingPython"
    | "verifyingInstaller"
    | "installingPython"
    | "preparingPackages"
    | "installingPackages"
    | "verifying"
    | "ready"
    | "needsRepair"
    | "error";
  progressPercent: number;
  message: string;
  pythonPath?: string;
  pythonVersion?: string;
  torchVersion?: string;
  transformersVersion?: string;
  accelerateVersion?: string;
  bitsandbytesVersion?: string;
  cudaAvailable?: boolean;
  cudaDeviceName?: string;
  lastError?: string;
  updatedAtMs: number;
};


export type RoutineStep =
  | { type: "launchApp"; app: string }
  | { type: "switchToApp"; app: string }
  | { type: "directorPreset"; preset: string }
  | { type: "wait"; milliseconds: number };

export type UserRoutine = {
  id: string;
  name: string;
  description: string;
  aliases: string[];
  steps: RoutineStep[];
  updatedAtMs: number;
};

export type SaveRoutineRequest = {
  id?: string;
  name: string;
  description?: string;
  aliases?: string[];
  steps: RoutineStep[];
};

export type RoutineStepResult = {
  index: number;
  status: string;
  label: string;
  message: string;
};

export type RoutineRunResult = {
  success: boolean;
  routineId: string;
  routineName: string;
  completedSteps: number;
  totalSteps: number;
  failedStep?: number | null;
  error?: string | null;
  steps: RoutineStepResult[];
  startedAtMs: number;
  completedAtMs: number;
};


export type ProjectMemory = {
  id: string;
  name: string;
  description: string;
  aliases: string[];
  notes: string[];
  routineIds: string[];
  createdAtMs: number;
  updatedAtMs: number;
};

export type ProjectMemorySnapshot = {
  projects: ProjectMemory[];
  activeProjectId?: string | null;
  refreshedAtMs: number;
};

export type SaveProjectRequest = {
  id?: string;
  name: string;
  description?: string;
  aliases?: string[];
  notes?: string[];
  routineIds?: string[];
};


export type AudioInputDevice = {
  name: string;
  isDefault: boolean;
};

export type AudioInputSnapshot = {
  devices: AudioInputDevice[];
  selectedDevice?: string | null;
  testing: boolean;
  pushToTalk: boolean;
  level: number;
  capturedSamples: number;
  captureDurationMs: number;
  sampleRate?: number | null;
  channels?: number | null;
  sampleFormat?: string | null;
  lastError?: string | null;
};


export type VoiceCaptureEvent = {
  phase:
    | "listening"
    | "captured"
    | "transcribing"
    | "transcribed"
    | "submitted"
    | "conversationListening"
    | "conversationTimeout"
    | "wakeDetected"
    | "error";
  shortcut: string;
  sampleCount: number;
  durationMs: number;
  sampleRate?: number | null;
  channels?: number | null;
  message: string;
  text?: string | null;
  timestampMs: number;
};


export type SpeechRuntimeStatus = {
  state: "stopped" | "loading" | "ready" | "transcribing" | "error" | string;
  modelId: string;
  device?: string | null;
  cuda?: boolean | null;
  lastText?: string | null;
  lastError?: string | null;
  refreshedAtMs: number;
};


export type TtsRuntimeStatus = {
  state:
    | "stopped"
    | "installingDependency"
    | "loading"
    | "ready"
    | "speaking"
    | "error"
    | string;
  modelId: string;
  dependencyReady: boolean;
  voiceInstalled: boolean;
  sampleRate?: number | null;
  lastText?: string | null;
  lastError?: string | null;
  refreshedAtMs: number;
};


export type VoicePreferences = {
  autoSpeak: boolean;
  ttsSpeed: number;
  conversationMode: boolean;
  conversationTimeoutSeconds: number;
  wakeWordEnabled: boolean;
  wakePhrase: string;
  ttsVoiceId: "voice-piper-ptpt" | "voice-piper-engb-alan" | string;
};


export type CaptureRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type VisionCapture = {
  id: string;
  kind: "screen" | "activeWindow" | "region" | string;
  path: string;
  rect: CaptureRect;
  windowTitle?: string | null;
  createdAtMs: number;
};

export type VisionEvent = {
  phase:
    | "capturing"
    | "captured"
    | "regionSelecting"
    | "analyzing"
    | "completed"
    | "historyError"
    | "error"
    | string;
  message: string;
  capture?: VisionCapture | null;
  analysis?: string | null;
  timestampMs: number;
};

export type VisionRuntimeStatus = {
  state: "stopped" | "loading" | "ready" | "analyzing" | "error" | string;
  modelId: string;
  device?: string | null;
  cuda?: boolean | null;
  lastPrompt?: string | null;
  lastAnalysis?: string | null;
  lastError?: string | null;
  refreshedAtMs: number;
};

export type VisionPreferences = {
  historyEnabled: boolean;
  retainImages: boolean;
  maxHistory: number;
};

export type VisionHistoryItem = {
  id: string;
  captureKind: string;
  prompt: string;
  analysis: string;
  windowTitle?: string | null;
  imagePath?: string | null;
  createdAtMs: number;
};

export type VisionHistorySnapshot = {
  preferences: VisionPreferences;
  items: VisionHistoryItem[];
  refreshedAtMs: number;
};

export type VisionAnalysisPayload = {
  capture: VisionCapture;
  prompt: string;
  analysis: string;
  completedAtMs: number;
  history: VisionHistorySnapshot;
};

export type VisionRegionRequest = {
  x: number;
  y: number;
  width: number;
  height: number;
};


export type AgentStep =
  | { type: "launchApp"; app: string }
  | { type: "switchToApp"; app: string }
  | { type: "runRoutine"; routine: string }
  | { type: "directorPreset"; preset: string }
  | { type: "wait"; milliseconds: number }
  | { type: "savedAction"; action: string };

export type AgentPlan = {
  id: string;
  goal: string;
  summary: string;
  steps: AgentStep[];
  requiresConfirmation: boolean;
  blockedByPolicy: boolean;
  highestPermission: PermissionClass;
  createdAtMs: number;
};

export type AgentStepResult = {
  index: number;
  label: string;
  status: "completed" | "failed" | string;
  attempts: number;
  message: string;
  startedAtMs: number;
  completedAtMs: number;
};

export type AgentRun = {
  id: string;
  planId: string;
  goal: string;
  state:
    | "queued"
    | "running"
    | "paused"
    | "completed"
    | "failed"
    | "cancelled"
    | "cancelling"
    | "interrupted"
    | string;
  currentStep?: number | null;
  completedSteps: number;
  totalSteps: number;
  steps: AgentStepResult[];
  startedAtMs: number;
  completedAtMs?: number | null;
  error?: string | null;
};

export type AgentSnapshot = {
  runs: AgentRun[];
  refreshedAtMs: number;
};

export type AgentEvent = {
  run: AgentRun;
  message: string;
  timestampMs: number;
};

export type SavedAuraAction = {
  id: string;
  name: string;
  description: string;
  aliases: string[];
  step: AgentStep;
  updatedAtMs: number;
};

export type SaveAuraActionRequest = {
  id?: string;
  name: string;
  description?: string;
  aliases?: string[];
  step: AgentStep;
};

export type AutomationTrigger =
  | { type: "startup" }
  | { type: "interval"; everyMinutes: number }
  | { type: "atTime"; runAtMs: number; repeatDaily: boolean }
  | { type: "appFocused"; app: string };

export type AuraAutomation = {
  id: string;
  name: string;
  enabled: boolean;
  actionId: string;
  trigger: AutomationTrigger;
  lastRunAtMs?: number | null;
  nextRunAtMs?: number | null;
  lastResult?: string | null;
  updatedAtMs: number;
};

export type SaveAutomationRequest = {
  id?: string;
  name: string;
  enabled: boolean;
  actionId: string;
  trigger: AutomationTrigger;
};

export type AutomationEvent = {
  automationId: string;
  automationName: string;
  status: "completed" | "failed" | "error" | string;
  message: string;
  timestampMs: number;
};


export type BetaStatus = {
  channel: "beta" | string;
  onboardingComplete: boolean;
  previousSessionUnclean: boolean;
  telemetryEnabled: boolean;
  automaticCrashUploads: boolean;
  localDiagnosticsOnly: boolean;
  refreshedAtMs: number;
};

export type SetBetaPreferencesRequest = {
  onboardingComplete: boolean;
};

export type DiagnosticsSnapshot = {
  schemaVersion: number;
  appName: string;
  appVersion: string;
  channel: string;
  platform: string;
  architecture: string;
  paused: boolean;
  backgroundEnabled: boolean;
  autostartEnabled: boolean;
  activeModelId?: string | null;
  installedModelIds: string[];
  managedRuntimeState: string;
  agentRunsTotal: number;
  activeAgentRuns: number;
  savedActions: number;
  automations: number;
  enabledAutomations: number;
  telemetryEnabled: boolean;
  generatedAtMs: number;
};
