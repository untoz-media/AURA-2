import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  AppStatus,
  CommandAck,
  CommandRequest,
  CoreError,
  CoreEvent,
  LifecycleEvent,
  PermissionClass,
  PermissionDecision,
  PermissionPolicy,
  RuntimeState,
  ObsConnectRequest,
  ObsConnectionState,
  ObsRuntimeState,
  ObsSceneList,
  ObsSceneSwitchRequest,
  ObsSceneSwitchResult,
  ObsRecordingActionResult,
  ObsStreamingActionResult,
  ObsStreamDuration,
  ObsSourceItemList,
  ObsSourceVisibilityRequest,
  ObsSourceVisibilityResult,
  ObsAudioInputList,
  ObsAudioMuteRequest,
  ObsAudioVolumeRequest,
  ObsAudioControlResult,
  ObsProductionHealth,
  DirectorPreset,
  SaveDirectorPresetRequest,
  DirectorPresetRunResult,
  MemorySnapshot,
  CreateMemoryRequest,
  MemoryCreateResult,
  MemoryRecord,
  CurrentAppInfo,
  AppSkillCatalog,
  DropIntakeSnapshot,
  DroppedFileInspection,
  RecentFilesSnapshot,
  ModelCatalog,
  ModelDownloadProgress,
  ModelRuntimeStatus,
  ManagedRuntimeStatus,
  ImageRuntimeStatus,
  ImageGenerationRequest,
  ImageGenerationResult,
  UserRoutine,
  SaveRoutineRequest,
  RoutineRunResult,
  ProjectMemory,
  ProjectMemorySnapshot,
  SaveProjectRequest,
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
  BetaStatus,
  SetBetaPreferencesRequest,
  DiagnosticsSnapshot,
} from "./types";

export const AURA_EVENTS = {
  core: "aura:core-event",
  error: "aura:core-error",
  runtime: "aura:runtime-state",
  lifecycle: "aura:lifecycle-event",
  modelDownload: "aura:model-download",
  managedRuntime: "aura:managed-runtime",
  voiceCapture: "aura:voice-capture",
  vision: "aura:vision-event",
  agent: "aura:agent-event",
  automation: "aura:automation-event",
} as const;

export async function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}

export async function getRuntimeState(): Promise<RuntimeState> {
  return invoke<RuntimeState>("get_runtime_state");
}

export async function setRuntimePaused(paused: boolean): Promise<RuntimeState> {
  return invoke<RuntimeState>("set_runtime_paused", { paused });
}

export async function setBackgroundEnabled(
  backgroundEnabled: boolean,
): Promise<RuntimeState> {
  return invoke<RuntimeState>("set_background_enabled", { backgroundEnabled });
}

export async function setAutostartEnabled(
  autostartEnabled: boolean,
): Promise<RuntimeState> {
  return invoke<RuntimeState>("set_autostart_enabled", { autostartEnabled });
}

export async function getPermissionPolicy(): Promise<PermissionPolicy> {
  return invoke<PermissionPolicy>("get_permission_policy");
}

export async function setPermissionDecision(
  permissionClass: PermissionClass,
  decision: PermissionDecision,
): Promise<PermissionPolicy> {
  return invoke<PermissionPolicy>("set_permission_decision", {
    class: permissionClass,
    decision,
  });
}

export async function resetPermissionPolicy(): Promise<PermissionPolicy> {
  return invoke<PermissionPolicy>("reset_permission_policy");
}

export async function resolveConfirmation(
  id: string,
  approved: boolean,
): Promise<CommandAck | null> {
  return invoke<CommandAck | null>("resolve_confirmation", { id, approved });
}

export async function submitAuraCommand(
  request: CommandRequest,
): Promise<CommandAck> {
  return invoke<CommandAck>("process_user_command", { request });
}

export async function openMainWindow(): Promise<void> {
  return invoke<void>("open_main_window");
}

export async function hideOverlay(): Promise<void> {
  return invoke<void>("hide_overlay");
}

export async function listenToOpenSettings(
  handler: () => void,
): Promise<UnlistenFn> {
  return listen("aura:open-settings", () => handler());
}

export async function listenToLifecycle(
  handler: (event: LifecycleEvent) => void,
): Promise<UnlistenFn> {
  return listen<LifecycleEvent>(
    AURA_EVENTS.lifecycle,
    ({ payload }) => handler(payload),
  );
}

export async function listenToAuraCore(
  onEvent: (event: CoreEvent) => void,
  onError: (error: CoreError) => void,
  onRuntime: (state: RuntimeState) => void,
): Promise<UnlistenFn> {
  const unlistenCore = await listen<CoreEvent>(
    AURA_EVENTS.core,
    ({ payload }) => onEvent(payload),
  );

  const unlistenError = await listen<CoreError>(
    AURA_EVENTS.error,
    ({ payload }) => onError(payload),
  );

  const unlistenRuntime = await listen<RuntimeState>(
    AURA_EVENTS.runtime,
    ({ payload }) => onRuntime(payload),
  );

  return () => {
    unlistenCore();
    unlistenError();
    unlistenRuntime();
  };
}


export async function getObsConnectionState(): Promise<ObsConnectionState> {
  return invoke<ObsConnectionState>("get_obs_connection_state");
}

export async function connectObs(
  request: ObsConnectRequest,
): Promise<ObsConnectionState> {
  return invoke<ObsConnectionState>("connect_obs", { request });
}

export async function disconnectObs(): Promise<ObsConnectionState> {
  return invoke<ObsConnectionState>("disconnect_obs");
}


export async function getObsRuntimeState(): Promise<ObsRuntimeState> {
  return invoke<ObsRuntimeState>("get_obs_runtime_state");
}


export async function getObsScenes(): Promise<ObsSceneList> {
  return invoke<ObsSceneList>("get_obs_scenes");
}


export async function setObsProgramScene(
  request: ObsSceneSwitchRequest,
): Promise<ObsSceneSwitchResult> {
  return invoke<ObsSceneSwitchResult>("set_obs_program_scene", { request });
}

export async function setObsPreviewScene(
  request: ObsSceneSwitchRequest,
): Promise<ObsSceneSwitchResult> {
  return invoke<ObsSceneSwitchResult>("set_obs_preview_scene", { request });
}


export async function startObsRecording(): Promise<ObsRecordingActionResult> {
  return invoke<ObsRecordingActionResult>("start_obs_recording");
}

export async function stopObsRecording(): Promise<ObsRecordingActionResult> {
  return invoke<ObsRecordingActionResult>("stop_obs_recording");
}

export async function pauseObsRecording(): Promise<ObsRecordingActionResult> {
  return invoke<ObsRecordingActionResult>("pause_obs_recording");
}

export async function resumeObsRecording(): Promise<ObsRecordingActionResult> {
  return invoke<ObsRecordingActionResult>("resume_obs_recording");
}


export async function startObsStreaming(): Promise<ObsStreamingActionResult> {
  return invoke<ObsStreamingActionResult>("start_obs_streaming");
}

export async function stopObsStreaming(): Promise<ObsStreamingActionResult> {
  return invoke<ObsStreamingActionResult>("stop_obs_streaming");
}


export async function getObsStreamDuration(): Promise<ObsStreamDuration> {
  return invoke<ObsStreamDuration>("get_obs_stream_duration");
}


export async function getObsSourceItems(): Promise<ObsSourceItemList> {
  return invoke<ObsSourceItemList>("get_obs_source_items");
}

export async function setObsSourceVisibility(
  request: ObsSourceVisibilityRequest,
): Promise<ObsSourceVisibilityResult> {
  return invoke<ObsSourceVisibilityResult>("set_obs_source_visibility", { request });
}


export async function getObsAudioInputs(): Promise<ObsAudioInputList> {
  return invoke<ObsAudioInputList>("get_obs_audio_inputs");
}

export async function setObsAudioMuted(
  request: ObsAudioMuteRequest,
): Promise<ObsAudioControlResult> {
  return invoke<ObsAudioControlResult>("set_obs_audio_muted", { request });
}

export async function setObsAudioVolume(
  request: ObsAudioVolumeRequest,
): Promise<ObsAudioControlResult> {
  return invoke<ObsAudioControlResult>("set_obs_audio_volume", { request });
}


export async function getObsProductionHealth(): Promise<ObsProductionHealth> {
  return invoke<ObsProductionHealth>("get_obs_production_health");
}


export async function getDirectorPresets(): Promise<DirectorPreset[]> {
  return invoke<DirectorPreset[]>("get_director_presets");
}

export async function saveDirectorPreset(
  request: SaveDirectorPresetRequest,
): Promise<DirectorPreset> {
  return invoke<DirectorPreset>("save_director_preset_command", { request });
}

export async function deleteDirectorPreset(presetId: string): Promise<void> {
  return invoke<void>("delete_director_preset_command", { presetId });
}

export async function runDirectorPreset(
  presetId: string,
): Promise<DirectorPresetRunResult> {
  return invoke<DirectorPresetRunResult>("run_director_preset_command", { presetId });
}


export async function getMemories(): Promise<MemorySnapshot> {
  return invoke<MemorySnapshot>("get_memories");
}

export async function createMemory(
  request: CreateMemoryRequest,
): Promise<MemoryCreateResult> {
  return invoke<MemoryCreateResult>("create_memory_command", { request });
}

export async function deleteMemory(memoryId: string): Promise<MemoryRecord> {
  return invoke<MemoryRecord>("delete_memory_command", { memoryId });
}


export async function getCurrentAppContext(): Promise<CurrentAppInfo> {
  return invoke<CurrentAppInfo>("get_current_app_context");
}

export async function getAppSkillCatalog(): Promise<AppSkillCatalog> {
  return invoke<AppSkillCatalog>("get_app_skill_catalog");
}

export async function getDropIntake(): Promise<DropIntakeSnapshot> {
  return invoke<DropIntakeSnapshot>("get_drop_intake");
}

export async function ingestDroppedFiles(
  paths: string[],
): Promise<DropIntakeSnapshot> {
  return invoke<DropIntakeSnapshot>("ingest_dropped_files", { paths });
}

export async function clearDropIntake(): Promise<DropIntakeSnapshot> {
  return invoke<DropIntakeSnapshot>("clear_drop_intake");
}

export async function revealDroppedFile(dropId: string): Promise<string> {
  return invoke<string>("reveal_dropped_file", { dropId });
}

export async function inspectDroppedFile(
  dropId: string,
): Promise<DroppedFileInspection> {
  return invoke<DroppedFileInspection>("inspect_dropped_file", { dropId });
}

export async function stageDroppedImageForVision(
  dropId: string,
): Promise<VisionCapture> {
  return invoke<VisionCapture>("stage_dropped_image_for_vision", { dropId });
}

export async function listenToFileDrop(
  handler: (paths: string[]) => void,
  hover?: (active: boolean) => void,
): Promise<UnlistenFn> {
  return getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === "enter" || event.payload.type === "over") {
      hover?.(true);
      return;
    }

    if (event.payload.type === "drop") {
      hover?.(false);
      handler(event.payload.paths);
      return;
    }

    hover?.(false);
  });
}

export async function getRecentFilesContext(): Promise<RecentFilesSnapshot> {
  return invoke<RecentFilesSnapshot>("get_recent_files_context");
}


export async function getModelCatalog(): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("get_model_catalog");
}

export async function startModelDownload(modelId: string): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("start_model_download", { modelId });
}

export async function pauseModelDownload(modelId: string): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("pause_model_download", { modelId });
}

export async function resumeModelDownload(modelId: string): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("resume_model_download", { modelId });
}

export async function cancelModelDownload(modelId: string): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("cancel_model_download", { modelId });
}

export async function setActiveModel(modelId: string): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("set_active_model", { modelId });
}

export async function removeModel(modelId: string): Promise<ModelCatalog> {
  return invoke<ModelCatalog>("remove_model", { modelId });
}

export async function listenToModelDownloads(
  handler: (progress: ModelDownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<ModelDownloadProgress>(
    AURA_EVENTS.modelDownload,
    ({ payload }) => handler(payload),
  );
}


export async function getModelRuntimeStatus(): Promise<ModelRuntimeStatus> {
  return invoke<ModelRuntimeStatus>("get_model_runtime_status");
}

export async function clearModelConversation(): Promise<ModelRuntimeStatus> {
  return invoke<ModelRuntimeStatus>("clear_model_conversation");
}


export async function getManagedRuntimeStatus(): Promise<ManagedRuntimeStatus> {
  return invoke<ManagedRuntimeStatus>("get_managed_runtime_status");
}

export async function installManagedRuntime(): Promise<ManagedRuntimeStatus> {
  return invoke<ManagedRuntimeStatus>("install_managed_runtime");
}

export async function repairManagedRuntime(): Promise<ManagedRuntimeStatus> {
  return invoke<ManagedRuntimeStatus>("repair_managed_runtime");
}

export async function removeManagedRuntime(): Promise<ManagedRuntimeStatus> {
  return invoke<ManagedRuntimeStatus>("remove_managed_runtime");
}

export async function listenToManagedRuntime(
  handler: (status: ManagedRuntimeStatus) => void,
): Promise<UnlistenFn> {
  return listen<ManagedRuntimeStatus>(
    AURA_EVENTS.managedRuntime,
    ({ payload }) => handler(payload),
  );
}


export async function getUserRoutines(): Promise<UserRoutine[]> {
  return invoke<UserRoutine[]>("get_user_routines");
}

export async function saveUserRoutine(
  request: SaveRoutineRequest,
): Promise<UserRoutine> {
  return invoke<UserRoutine>("save_user_routine", { request });
}

export async function deleteUserRoutine(routineId: string): Promise<void> {
  return invoke<void>("delete_user_routine", { routineId });
}

export async function runUserRoutine(routineId: string): Promise<RoutineRunResult> {
  return invoke<RoutineRunResult>("run_user_routine", { routineId });
}


export async function getProjectMemory(): Promise<ProjectMemorySnapshot> {
  return invoke<ProjectMemorySnapshot>("get_project_memory");
}

export async function saveProjectMemory(
  request: SaveProjectRequest,
): Promise<ProjectMemory> {
  return invoke<ProjectMemory>("save_project_memory", { request });
}

export async function deleteProjectMemory(projectId: string): Promise<void> {
  return invoke<void>("delete_project_memory", { projectId });
}

export async function setActiveProjectMemory(
  projectId?: string,
): Promise<ProjectMemorySnapshot> {
  return invoke<ProjectMemorySnapshot>("set_active_project_memory", {
    projectId: projectId ?? null,
  });
}


export async function getAudioInputState(): Promise<AudioInputSnapshot> {
  return invoke<AudioInputSnapshot>("get_audio_input_state");
}

export async function selectAudioInputDevice(
  deviceName?: string,
): Promise<AudioInputSnapshot> {
  return invoke<AudioInputSnapshot>("select_audio_input_device", {
    deviceName: deviceName ?? null,
  });
}

export async function startAudioInputTest(): Promise<AudioInputSnapshot> {
  return invoke<AudioInputSnapshot>("start_audio_input_test");
}

export async function stopAudioInputTest(): Promise<AudioInputSnapshot> {
  return invoke<AudioInputSnapshot>("stop_audio_input_test");
}


export async function listenToVoiceCapture(
  handler: (event: VoiceCaptureEvent) => void,
): Promise<UnlistenFn> {
  return listen<VoiceCaptureEvent>(
    AURA_EVENTS.voiceCapture,
    ({ payload }) => handler(payload),
  );
}


export async function getSpeechRuntimeStatus(): Promise<SpeechRuntimeStatus> {
  return invoke<SpeechRuntimeStatus>("get_speech_runtime_status");
}


export async function getTtsRuntimeStatus(): Promise<TtsRuntimeStatus> {
  return invoke<TtsRuntimeStatus>("get_tts_runtime_status");
}

export async function prepareTtsRuntime(): Promise<TtsRuntimeStatus> {
  return invoke<TtsRuntimeStatus>("prepare_tts_runtime");
}

export async function testTtsVoice(text?: string): Promise<TtsRuntimeStatus> {
  return invoke<TtsRuntimeStatus>("test_tts_voice", {
    text: text ?? null,
  });
}


export async function getVoicePreferences(): Promise<VoicePreferences> {
  return invoke<VoicePreferences>("get_voice_preferences");
}

export async function setVoicePreferences(
  preferences: VoicePreferences,
): Promise<VoicePreferences> {
  return invoke<VoicePreferences>("set_voice_preferences", { preferences });
}

export async function stopTtsSpeaking(): Promise<TtsRuntimeStatus> {
  return invoke<TtsRuntimeStatus>("stop_tts_speaking");
}


export async function getCreateImageRuntimeStatus(): Promise<ImageRuntimeStatus> {
  return invoke<ImageRuntimeStatus>("get_create_image_runtime_status");
}

export async function generateCreateImage(
  request: ImageGenerationRequest,
): Promise<ImageGenerationResult> {
  return invoke<ImageGenerationResult>("generate_create_image", { request });
}


export async function getVisionRuntimeStatus(): Promise<VisionRuntimeStatus> {
  return invoke<VisionRuntimeStatus>("get_vision_runtime_status");
}

export async function getVisionHistory(): Promise<VisionHistorySnapshot> {
  return invoke<VisionHistorySnapshot>("get_vision_history");
}

export async function setVisionPreferences(
  preferences: VisionPreferences,
): Promise<VisionHistorySnapshot> {
  return invoke<VisionHistorySnapshot>("set_vision_preferences", { preferences });
}

export async function clearVisionHistory(): Promise<VisionHistorySnapshot> {
  return invoke<VisionHistorySnapshot>("clear_vision_history");
}

export async function getLastVisionCapture(): Promise<VisionCapture | null> {
  return invoke<VisionCapture | null>("get_last_vision_capture");
}

export async function clearLastVisionCapture(): Promise<void> {
  return invoke<void>("clear_last_vision_capture");
}

export async function captureVisionScreen(): Promise<VisionCapture> {
  return invoke<VisionCapture>("capture_vision_screen");
}

export async function captureVisionActiveWindow(): Promise<VisionCapture> {
  return invoke<VisionCapture>("capture_vision_active_window");
}

export async function captureVisionRegion(
  request: VisionRegionRequest,
): Promise<VisionCapture> {
  return invoke<VisionCapture>("capture_vision_region", { request });
}

export async function analyzeLastVisionCapture(
  prompt: string,
): Promise<VisionAnalysisPayload> {
  return invoke<VisionAnalysisPayload>("analyze_last_vision_capture", { prompt });
}

export async function listenToVision(
  handler: (event: VisionEvent) => void,
): Promise<UnlistenFn> {
  return listen<VisionEvent>(
    AURA_EVENTS.vision,
    ({ payload }) => handler(payload),
  );
}


export async function createAgentPlan(goal: string): Promise<AgentPlan> {
  return invoke<AgentPlan>("create_agent_plan", { goal });
}

export async function getAgentRuns(): Promise<AgentSnapshot> {
  return invoke<AgentSnapshot>("get_agent_runs");
}

export async function startAgentPlan(
  plan: AgentPlan,
  approved: boolean,
): Promise<AgentRun> {
  return invoke<AgentRun>("start_agent_plan", { plan, approved });
}

export async function pauseAgentRun(
  runId: string,
  paused: boolean,
): Promise<AgentRun> {
  return invoke<AgentRun>("pause_agent_run", { runId, paused });
}

export async function cancelAgentRun(runId: string): Promise<AgentRun> {
  return invoke<AgentRun>("cancel_agent_run", { runId });
}

export async function getSavedAuraActions(): Promise<SavedAuraAction[]> {
  return invoke<SavedAuraAction[]>("get_saved_aura_actions");
}

export async function saveAuraAction(
  request: SaveAuraActionRequest,
): Promise<SavedAuraAction> {
  return invoke<SavedAuraAction>("save_aura_action", { request });
}

export async function deleteAuraAction(actionId: string): Promise<void> {
  return invoke<void>("delete_aura_action", { actionId });
}

export async function runAuraAction(
  actionId: string,
  approved: boolean,
): Promise<string> {
  return invoke<string>("run_aura_action", { actionId, approved });
}

export async function getAuraAutomations(): Promise<AuraAutomation[]> {
  return invoke<AuraAutomation[]>("get_aura_automations");
}

export async function saveAuraAutomation(
  request: SaveAutomationRequest,
): Promise<AuraAutomation> {
  return invoke<AuraAutomation>("save_aura_automation", { request });
}

export async function deleteAuraAutomation(automationId: string): Promise<void> {
  return invoke<void>("delete_aura_automation", { automationId });
}

export async function setAuraAutomationEnabled(
  automationId: string,
  enabled: boolean,
): Promise<AuraAutomation> {
  return invoke<AuraAutomation>("set_aura_automation_enabled", {
    automationId,
    enabled,
  });
}

export async function listenToAgent(
  handler: (event: AgentEvent) => void,
): Promise<UnlistenFn> {
  return listen<AgentEvent>(AURA_EVENTS.agent, ({ payload }) => handler(payload));
}

export async function listenToAutomation(
  handler: (event: AutomationEvent) => void,
): Promise<UnlistenFn> {
  return listen<AutomationEvent>(
    AURA_EVENTS.automation,
    ({ payload }) => handler(payload),
  );
}


export async function getBetaStatus(): Promise<BetaStatus> {
  return invoke<BetaStatus>("get_beta_status");
}

export async function setBetaPreferences(
  request: SetBetaPreferencesRequest,
): Promise<BetaStatus> {
  return invoke<BetaStatus>("set_beta_preferences", { request });
}

export async function getBetaDiagnostics(): Promise<DiagnosticsSnapshot> {
  return invoke<DiagnosticsSnapshot>("get_beta_diagnostics");
}

export async function exportBetaDiagnostics(): Promise<string> {
  return invoke<string>("export_beta_diagnostics");
}
