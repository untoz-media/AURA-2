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
  ModelCatalog,
  ModelDownloadProgress,
  ModelRuntimeStatus,
} from "./types";

export const AURA_EVENTS = {
  core: "aura:core-event",
  error: "aura:core-error",
  runtime: "aura:runtime-state",
  lifecycle: "aura:lifecycle-event",
  modelDownload: "aura:model-download",
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
