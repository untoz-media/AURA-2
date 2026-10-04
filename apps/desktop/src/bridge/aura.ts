import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
  RecentFilesSnapshot,
  ModelCatalog,
  ModelDownloadProgress,
  ModelRuntimeStatus,
  ManagedRuntimeStatus,
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
